//! Simple File System for Genesis OS
//!
//! Provides basic file operations for agent data persistence.
//! Initially in-memory, will be extended to persistent storage.

use alloc::string::String;
use alloc::vec::Vec;
use alloc::collections::BTreeMap;
use spin::Mutex;
use lazy_static::lazy_static;

/// File system entry
#[derive(Debug, Clone)]
pub enum FileSystemEntry {
    File { content: Vec<u8> },
    Directory { entries: BTreeMap<String, FileSystemEntry> },
}

/// Simple in-memory file system
pub struct FileSystem {
    root: BTreeMap<String, FileSystemEntry>,
}

impl FileSystem {
    /// Create a new file system
    pub fn new() -> Self {
        let mut fs = FileSystem {
            root: BTreeMap::new(),
        };
        
        // Create standard directory structure
        fs.create_directory("/storage").unwrap();
        fs.create_directory("/storage/agents").unwrap();
        fs.create_directory("/storage/agents/archimedes").unwrap();
        fs.create_directory("/storage/agents/archimedes/daily_ambitions").unwrap();
        fs.create_directory("/storage/agents/archimedes/workspace_layouts").unwrap();
        fs.create_directory("/storage/agents/scout").unwrap();
        fs.create_directory("/storage/agents/scout/resources").unwrap();
        fs.create_directory("/storage/agents/sentinel").unwrap();
        fs.create_directory("/storage/agents/sentinel/security_logs").unwrap();
        fs.create_directory("/storage/agents/typewrite").unwrap();
        fs.create_directory("/storage/agents/thomas").unwrap();
        fs.create_directory("/workspaces").unwrap();
        fs.create_directory("/workspaces/today").unwrap();
        fs.create_directory("/workspaces/today/focus").unwrap();
        fs.create_directory("/workspaces/today/resources").unwrap();
        fs.create_directory("/workspaces/today/output").unwrap();
        
        fs
    }
    
    /// Read a file
    pub fn read_file(&self, path: &str) -> Result<Vec<u8>, FileSystemError> {
        let parts: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
        let mut current = &self.root;
        
        // Navigate to parent directory
        for part in parts.iter().take(parts.len().saturating_sub(1)) {
            match current.get(*part) {
                Some(FileSystemEntry::Directory { entries }) => {
                    current = entries;
                }
                _ => return Err(FileSystemError::NotFound),
            }
        }
        
        // Get file
        let filename = parts.last().ok_or(FileSystemError::InvalidPath)?;
        match current.get(*filename) {
            Some(FileSystemEntry::File { content }) => Ok(content.clone()),
            _ => Err(FileSystemError::NotFound),
        }
    }
    
    /// Read file as string
    pub fn read_file_string(&self, path: &str) -> Result<String, FileSystemError> {
        let bytes = self.read_file(path)?;
        String::from_utf8(bytes).map_err(|_| FileSystemError::InvalidEncoding)
    }
    
    /// Write a file
    pub fn write_file(&mut self, path: &str, data: &[u8]) -> Result<(), FileSystemError> {
        let parts: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
        
        if parts.is_empty() {
            return Err(FileSystemError::InvalidPath);
        }
        
        // Ensure parent directories exist
        let mut current = &mut self.root;
        for part in parts.iter().take(parts.len().saturating_sub(1)) {
            if !current.contains_key(*part) {
                current.insert(String::from(*part), FileSystemEntry::Directory {
                    entries: BTreeMap::new(),
                });
            }
            
            match current.get_mut(*part) {
                Some(FileSystemEntry::Directory { entries }) => {
                    current = entries;
                }
                _ => return Err(FileSystemError::InvalidPath),
            }
        }
        
        // Write file
        let filename = parts.last().ok_or(FileSystemError::InvalidPath)?;
        current.insert(String::from(*filename), FileSystemEntry::File {
            content: data.to_vec(),
        });
        
        Ok(())
    }
    
    /// Write file from string
    pub fn write_file_string(&mut self, path: &str, content: &str) -> Result<(), FileSystemError> {
        self.write_file(path, content.as_bytes())
    }
    
    /// List directory
    pub fn list_dir(&self, path: &str) -> Result<Vec<String>, FileSystemError> {
        let parts: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
        let mut current = &self.root;
        
        // Navigate to directory
        for part in parts {
            match current.get(part) {
                Some(FileSystemEntry::Directory { entries }) => {
                    current = entries;
                }
                _ => return Err(FileSystemError::NotFound),
            }
        }
        
        Ok(current.keys().cloned().collect())
    }
    
    /// Create directory
    pub fn create_directory(&mut self, path: &str) -> Result<(), FileSystemError> {
        let parts: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
        
        if parts.is_empty() {
            return Ok(()); // Root already exists
        }
        
        let mut current = &mut self.root;
        for part in parts {
            if !current.contains_key(part) {
                current.insert(String::from(part), FileSystemEntry::Directory {
                    entries: BTreeMap::new(),
                });
            }
            
            match current.get_mut(part) {
                Some(FileSystemEntry::Directory { entries }) => {
                    current = entries;
                }
                _ => return Err(FileSystemError::InvalidPath),
            }
        }
        
        Ok(())
    }
    
    /// Check if file exists
    pub fn file_exists(&self, path: &str) -> bool {
        self.read_file(path).is_ok()
    }
    
    /// Check if directory exists
    pub fn dir_exists(&self, path: &str) -> bool {
        self.list_dir(path).is_ok()
    }
}

/// File system errors
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileSystemError {
    NotFound,
    InvalidPath,
    InvalidEncoding,
    AlreadyExists,
}

/// Global file system instance
lazy_static! {
    pub static ref FILESYSTEM: Mutex<FileSystem> = Mutex::new(FileSystem::new());
}

/// Read file from global file system
pub fn read_file(path: &str) -> Result<Vec<u8>, FileSystemError> {
    FILESYSTEM.lock().read_file(path)
}

/// Read file as string from global file system
pub fn read_file_string(path: &str) -> Result<String, FileSystemError> {
    FILESYSTEM.lock().read_file_string(path)
}

/// Write file to global file system
pub fn write_file(path: &str, data: &[u8]) -> Result<(), FileSystemError> {
    FILESYSTEM.lock().write_file(path, data)
}

/// Write file as string to global file system
pub fn write_file_string(path: &str, content: &str) -> Result<(), FileSystemError> {
    FILESYSTEM.lock().write_file_string(path, content)
}

/// List directory in global file system
pub fn list_dir(path: &str) -> Result<Vec<String>, FileSystemError> {
    FILESYSTEM.lock().list_dir(path)
}

/// Create directory in global file system
pub fn create_dir(path: &str) -> Result<(), FileSystemError> {
    FILESYSTEM.lock().create_directory(path)
}

/// Check if file exists
pub fn file_exists(path: &str) -> bool {
    FILESYSTEM.lock().file_exists(path)
}

/// Check if directory exists
pub fn dir_exists(path: &str) -> bool {
    FILESYSTEM.lock().dir_exists(path)
}



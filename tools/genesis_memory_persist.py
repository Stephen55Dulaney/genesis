#!/usr/bin/env python3
"""
Genesis Memory Persistence System
Two-layer backup: Local disk + GitHub disaster recovery
"""

import json
import os
import subprocess
import time
from datetime import datetime
from pathlib import Path
from typing import Dict, Any, Optional
import hashlib

class GenesisMemory:
    """
    Persistent memory system for Genesis OS
    
    Layer 1: Local persistence (survives reboots)
    Layer 2: GitHub backup (survives hardware failure)
    """
    
    def __init__(self, base_path: str = None):
        if base_path is None:
            base_path = os.path.expanduser("~/genesis")
        
        self.base_path = Path(base_path)
        self.memory_dir = self.base_path / "memory"
        self.checkpoint_dir = self.memory_dir / "checkpoints"
        self.state_dir = self.memory_dir / "state"
        self.secrets_dir = self.memory_dir / "secrets"
        
        # Create directory structure
        for d in [self.memory_dir, self.checkpoint_dir, self.state_dir, self.secrets_dir]:
            d.mkdir(parents=True, exist_ok=True)
        
        # Ensure secrets are gitignored
        gitignore = self.memory_dir / ".gitignore"
        if not gitignore.exists():
            gitignore.write_text("secrets/\n*.key\n*.secret\n")
    
    def critical(self, key: str, value: Any, encrypt: bool = False) -> bool:
        """
        Store critical data that must persist immediately
        
        Args:
            key: Unique identifier
            value: Data to store (will be JSON serialized)
            encrypt: Whether to store in secrets (gitignored)
        
        Returns:
            True if successful
        """
        target_dir = self.secrets_dir if encrypt else self.state_dir
        filepath = target_dir / f"{key}.json"
        
        data = {
            "key": key,
            "value": value,
            "timestamp": datetime.now().isoformat(),
            "encrypted": encrypt
        }
        
        try:
            with open(filepath, 'w') as f:
                json.dump(data, f, indent=2)
            return True
        except Exception as e:
            print(f"❌ Failed to store {key}: {e}")
            return False
    
    def recall(self, key: str) -> Optional[Any]:
        """
        Retrieve critical data by key
        
        Returns:
            The stored value, or None if not found
        """
        # Check state first, then secrets
        for directory in [self.state_dir, self.secrets_dir]:
            filepath = directory / f"{key}.json"
            if filepath.exists():
                try:
                    with open(filepath, 'r') as f:
                        data = json.load(f)
                    return data['value']
                except Exception as e:
                    print(f"⚠️ Failed to read {key}: {e}")
        
        return None
    
    def checkpoint(self, label: str, metadata: Dict[str, Any] = None) -> str:
        """
        Create a complete system checkpoint
        
        Args:
            label: Human-readable checkpoint name
            metadata: Additional context to store
        
        Returns:
            Checkpoint ID
        """
        timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
        checkpoint_id = f"{timestamp}_{label}"
        checkpoint_path = self.checkpoint_dir / checkpoint_id
        checkpoint_path.mkdir(exist_ok=True)
        
        # Collect all state files
        state_files = list(self.state_dir.glob("*.json"))
        
        checkpoint_data = {
            "id": checkpoint_id,
            "label": label,
            "timestamp": datetime.now().isoformat(),
            "metadata": metadata or {},
            "state_count": len(state_files),
            "files": []
        }
        
        # Copy state files to checkpoint
        for state_file in state_files:
            dest = checkpoint_path / state_file.name
            dest.write_bytes(state_file.read_bytes())
            checkpoint_data["files"].append(state_file.name)
        
        # Write checkpoint manifest
        manifest = checkpoint_path / "manifest.json"
        with open(manifest, 'w') as f:
            json.dump(checkpoint_data, f, indent=2)
        
        print(f"✅ Checkpoint created: {checkpoint_id}")
        print(f"   Saved {len(state_files)} state files")
        
        return checkpoint_id
    
    def restore(self, checkpoint_id: str) -> bool:
        """
        Restore from a checkpoint
        
        Args:
            checkpoint_id: The checkpoint to restore
        
        Returns:
            True if successful
        """
        checkpoint_path = self.checkpoint_dir / checkpoint_id
        manifest_path = checkpoint_path / "manifest.json"
        
        if not manifest_path.exists():
            print(f"❌ Checkpoint not found: {checkpoint_id}")
            return False
        
        try:
            with open(manifest_path, 'r') as f:
                manifest = json.load(f)
            
            # Restore all files
            restored = 0
            for filename in manifest["files"]:
                src = checkpoint_path / filename
                dest = self.state_dir / filename
                if src.exists():
                    dest.write_bytes(src.read_bytes())
                    restored += 1
            
            print(f"✅ Restored from checkpoint: {checkpoint_id}")
            print(f"   Restored {restored}/{len(manifest['files'])} files")
            return True
            
        except Exception as e:
            print(f"❌ Restore failed: {e}")
            return False
    
    def list_checkpoints(self):
        """List all available checkpoints"""
        checkpoints = []
        for cp_dir in sorted(self.checkpoint_dir.iterdir(), reverse=True):
            if cp_dir.is_dir():
                manifest = cp_dir / "manifest.json"
                if manifest.exists():
                    with open(manifest, 'r') as f:
                        data = json.load(f)
                    checkpoints.append({
                        "id": data["id"],
                        "label": data["label"],
                        "timestamp": data["timestamp"],
                        "files": data["state_count"]
                    })
        return checkpoints
    
    def archive_session(self, session_name: str = None) -> str:
        """
        Archive entire session before shutdown
        
        Args:
            session_name: Optional name for the session
        
        Returns:
            Checkpoint ID
        """
        if session_name is None:
            session_name = "auto_archive"
        
        metadata = {
            "type": "session_archive",
            "reason": "pre_shutdown",
            "working_dir": os.getcwd()
        }
        
        return self.checkpoint(session_name, metadata)
    
    def git_backup(self, commit_message: str = None) -> bool:
        """
        Backup memory state to GitHub
        
        Args:
            commit_message: Custom commit message
        
        Returns:
            True if successful
        """
        if commit_message is None:
            commit_message = f"Memory backup: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}"
        
        try:
            # Add memory directory (excluding secrets via .gitignore)
            subprocess.run(
                ["git", "add", "memory/"],
                cwd=self.base_path,
                check=True,
                capture_output=True
            )
            
            # Check if there are changes
            result = subprocess.run(
                ["git", "diff", "--cached", "--quiet"],
                cwd=self.base_path,
                capture_output=True
            )
            
            if result.returncode == 0:
                print("ℹ️ No changes to backup")
                return True
            
            # Commit
            subprocess.run(
                ["git", "commit", "-m", commit_message],
                cwd=self.base_path,
                check=True,
                capture_output=True
            )
            
            # Push to GitHub
            subprocess.run(
                ["git", "push"],
                cwd=self.base_path,
                check=True,
                capture_output=True
            )
            
            print(f"✅ Backed up to GitHub: {commit_message}")
            return True
            
        except subprocess.CalledProcessError as e:
            print(f"⚠️ Git backup failed: {e}")
            return False
    
    def auto_backup(self, interval_seconds: int = 300):
        """
        Automatic backup daemon (runs in background)
        
        Args:
            interval_seconds: How often to backup (default 5 minutes)
        """
        print(f"🔄 Auto-backup started (every {interval_seconds}s)")
        
        last_hash = None
        
        while True:
            time.sleep(interval_seconds)
            
            # Calculate hash of current state
            state_files = sorted(self.state_dir.glob("*.json"))
            current_hash = hashlib.sha256()
            for f in state_files:
                current_hash.update(f.read_bytes())
            current_hash = current_hash.hexdigest()
            
            # Only backup if changed
            if current_hash != last_hash:
                self.git_backup("Auto-backup: state changed")
                last_hash = current_hash
            else:
                print("ℹ️ No changes since last backup")


class GenesisState:
    """High-level state management for Genesis agents"""
    
    def __init__(self):
        self.memory = GenesisMemory()
    
    def set_ambition(self, ambition: str):
        """Store daily ambition"""
        return self.memory.critical("daily_ambition", {
            "text": ambition,
            "set_at": datetime.now().isoformat()
        })
    
    def get_ambition(self) -> Optional[str]:
        """Retrieve current ambition"""
        data = self.memory.recall("daily_ambition")
        return data["text"] if data else None
    
    def set_api_key(self, service: str, key: str):
        """Store API key (encrypted/gitignored)"""
        return self.memory.critical(f"api_key_{service}", key, encrypt=True)
    
    def get_api_key(self, service: str) -> Optional[str]:
        """Retrieve API key"""
        return self.memory.recall(f"api_key_{service}")
    
    def log_event(self, event_type: str, data: Dict[str, Any]):
        """Log important system event"""
        timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
        key = f"event_{event_type}_{timestamp}"
        return self.memory.critical(key, data)


# Command-line interface
if __name__ == "__main__":
    import sys
    
    memory = GenesisMemory()
    
    if len(sys.argv) < 2:
        print("Genesis Memory Persistence System")
        print("\nUsage:")
        print("  python genesis_memory_persist.py checkpoint <label>")
        print("  python genesis_memory_persist.py restore <checkpoint_id>")
        print("  python genesis_memory_persist.py list")
        print("  python genesis_memory_persist.py backup [message]")
        print("  python genesis_memory_persist.py auto-backup")
        sys.exit(1)
    
    command = sys.argv[1]
    
    if command == "checkpoint":
        label = sys.argv[2] if len(sys.argv) > 2 else "manual"
        checkpoint_id = memory.checkpoint(label)
        print(f"Checkpoint ID: {checkpoint_id}")
    
    elif command == "restore":
        if len(sys.argv) < 3:
            print("❌ Usage: restore <checkpoint_id>")
            sys.exit(1)
        checkpoint_id = sys.argv[2]
        memory.restore(checkpoint_id)
    
    elif command == "list":
        checkpoints = memory.list_checkpoints()
        print(f"\n📦 {len(checkpoints)} checkpoints available:\n")
        for cp in checkpoints:
            print(f"  {cp['id']}")
            print(f"    Label: {cp['label']}")
            print(f"    Time: {cp['timestamp']}")
            print(f"    Files: {cp['files']}")
            print()
    
    elif command == "backup":
        message = " ".join(sys.argv[2:]) if len(sys.argv) > 2 else None
        memory.git_backup(message)
    
    elif command == "auto-backup":
        memory.auto_backup()
    
    else:
        print(f"❌ Unknown command: {command}")
        sys.exit(1)

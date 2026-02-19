#!/usr/bin/env python3
"""
Genesis OS - Gemini Vision Integration
Gives Genesis the ability to see and understand images.
"""

import os
import base64
import json
import requests
from pathlib import Path
from typing import Dict, Optional, List

class GeminiVision:
    """Vision system powered by Gemini 3 Flash API."""
    
    def __init__(self, api_key: Optional[str] = None):
        """Initialize with API key from env or parameter."""
        self.api_key = api_key or os.getenv('GEMINI_API_KEY')
        if not self.api_key:
            raise ValueError("GEMINI_API_KEY not found in environment or parameter")
        
        # Using Gemini 3 Flash (latest)
        self.model = "gemini-3-flash-preview"
        self.base_url = "https://generativelanguage.googleapis.com/v1beta"
        
    def encode_image(self, image_path: str) -> str:
        """Encode image to base64 string."""
        with open(image_path, 'rb') as f:
            return base64.b64encode(f.read()).decode('utf-8')
    
    def encode_image_from_url(self, url: str) -> tuple[str, str]:
        """Download and encode image from URL."""
        response = requests.get(url, timeout=10)
        response.raise_for_status()
        
        # Detect mime type
        content_type = response.headers.get('content-type', 'image/jpeg')
        
        encoded = base64.b64encode(response.content).decode('utf-8')
        return encoded, content_type
    
    def see(self, image_source: str, prompt: str = "Describe what you see in this image in detail.") -> Dict:
        """
        Analyze an image using Gemini Vision.
        
        Args:
            image_source: File path or URL to image
            prompt: Question or instruction about the image
            
        Returns:
            Dict with 'description', 'raw_response', and metadata
        """
        
        # Determine if source is URL or file path
        is_url = image_source.startswith('http://') or image_source.startswith('https://')
        
        try:
            if is_url:
                image_data, mime_type = self.encode_image_from_url(image_source)
            else:
                if not os.path.exists(image_source):
                    return {
                        'error': f"Image file not found: {image_source}",
                        'success': False
                    }
                image_data = self.encode_image(image_source)
                # Detect mime from extension
                ext = Path(image_source).suffix.lower()
                mime_map = {'.jpg': 'image/jpeg', '.jpeg': 'image/jpeg', 
                           '.png': 'image/png', '.gif': 'image/gif', '.webp': 'image/webp'}
                mime_type = mime_map.get(ext, 'image/jpeg')
            
            # Build request payload
            payload = {
                "contents": [{
                    "parts": [
                        {"text": prompt},
                        {
                            "inline_data": {
                                "mime_type": mime_type,
                                "data": image_data
                            }
                        }
                    ]
                }]
            }
            
            # Call Gemini API with correct model
            url = f"{self.base_url}/models/{self.model}:generateContent?key={self.api_key}"
            
            response = requests.post(url, json=payload, timeout=30)
            
            # Debug output for troubleshooting
            if response.status_code != 200:
                return {
                    'error': f"API returned {response.status_code}: {response.text[:300]}",
                    'success': False,
                    'status_code': response.status_code
                }
            
            result = response.json()
            
            # Extract text from response
            if 'candidates' in result and len(result['candidates']) > 0:
                candidate = result['candidates'][0]
                if 'content' in candidate and 'parts' in candidate['content']:
                    text = candidate['content']['parts'][0].get('text', '')
                    
                    return {
                        'success': True,
                        'description': text,
                        'raw_response': result,
                        'source': image_source,
                        'prompt': prompt,
                        'model': self.model
                    }
            
            return {
                'error': 'Unexpected response format',
                'success': False,
                'raw_response': result
            }
            
        except requests.exceptions.RequestException as e:
            return {
                'error': f"Network error: {str(e)}",
                'success': False
            }
        except Exception as e:
            return {
                'error': f"Vision error: {str(e)}",
                'success': False
            }
    
    def analyze_scene(self, image_source: str) -> Dict:
        """Get detailed scene analysis."""
        return self.see(image_source, 
                       "Analyze this image in detail. Describe: 1) Main subjects, "
                       "2) Environment/setting, 3) Colors and lighting, 4) Any text visible, "
                       "5) Notable details or context.")
    
    def quick_look(self, image_source: str) -> str:
        """Quick one-sentence description."""
        result = self.see(image_source, "Describe this image in one clear sentence.")
        if result.get('success'):
            return result.get('description', 'No description')
        else:
            return f"ERROR: {result.get('error', 'Unknown error')}"
    
    def find_objects(self, image_source: str) -> Dict:
        """Detect and list objects in the image."""
        return self.see(image_source,
                       "List all distinct objects you can identify in this image. "
                       "Format as a numbered list.")
    
    def read_text(self, image_source: str) -> Dict:
        """Extract any text visible in the image (OCR)."""
        return self.see(image_source,
                       "Extract and transcribe any text visible in this image. "
                       "If no text is present, say 'No text detected'.")
    
    def answer_question(self, image_source: str, question: str) -> Dict:
        """Ask a specific question about the image."""
        return self.see(image_source, question)


def test_vision():
    """Test the vision system with the frog image."""
    print("🔧 Genesis Vision System - Gemini 3 Flash")
    print("=" * 60)
    
    # Load API key from environment
    api_key = os.getenv("GEMINI_API_KEY")
    if not api_key:
        print("Set GEMINI_API_KEY environment variable first")
        return
    
    vision = GeminiVision(api_key=api_key)
    print(f"✓ Initialized with model: {vision.model}\n")
    
    # Test with the frog
    frog_url = "https://cdn.britannica.com/72/45872-050-10F6A603/Green-frog.jpg"
    
    print(f"📸 Image: {frog_url}")
    print("⏳ Processing...\n")
    
    result = vision.see(frog_url, "Describe this image in one clear sentence.")
    
    if result.get('success'):
        print("=" * 60)
        print("👁️  GENESIS CAN SEE")
        print("=" * 60)
        print(f"\n{result['description']}\n")
        print("-" * 60)
        
        # Get detailed analysis
        print("\n🔍 Detailed Analysis:")
        detailed = vision.analyze_scene(frog_url)
        if detailed.get('success'):
            print(detailed['description'])
        
        print("\n" + "=" * 60)
        print("✅ Vision system fully operational.")
        print("=" * 60)
    else:
        print(f"❌ ERROR: {result.get('error')}")
        print("\nDebugging info:")
        print(result)


if __name__ == '__main__':
    test_vision()

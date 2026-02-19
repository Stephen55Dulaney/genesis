#!/usr/bin/env python3
"""
Genesis Vision System
Built for robot embodiment - processes camera feeds and provides scene understanding
"""

import base64
import json
from pathlib import Path
from typing import Dict, List, Optional

class VisionProcessor:
    """
    Core vision processing for Genesis agents.
    Designed to work with Claude Sonnet's native vision capabilities.
    """
    
    def __init__(self):
        self.last_frame = None
        self.frame_history = []
        self.max_history = 10
    
    def encode_image(self, image_path: str) -> str:
        """Encode image to base64 for processing."""
        with open(image_path, 'rb') as f:
            return base64.b64encode(f.read()).decode('utf-8')
    
    def process_frame(self, image_path: str, context: str = "") -> Dict:
        """
        Process a single frame from robot camera.
        Returns structured observation for agent memory.
        """
        self.last_frame = image_path
        self.frame_history.append(image_path)
        if len(self.frame_history) > self.max_history:
            self.frame_history.pop(0)
        
        return {
            'type': 'vision_frame',
            'path': image_path,
            'context': context,
            'timestamp': self._get_timestamp(),
            'encoded': self.encode_image(image_path)
        }
    
    def compare_frames(self, frame1_path: str, frame2_path: str) -> Dict:
        """Compare two frames to detect changes."""
        return {
            'type': 'frame_comparison',
            'frame1': frame1_path,
            'frame2': frame2_path,
            'timestamp': self._get_timestamp()
        }
    
    def _get_timestamp(self) -> int:
        """Get current timestamp in ms."""
        import time
        return int(time.time() * 1000)

class RobotEyes:
    """
    Interface between robot camera hardware and Genesis vision system.
    Abstracts different robot platforms (Vector, Pi-based, etc.)
    """
    
    def __init__(self, robot_type: str = "generic"):
        self.robot_type = robot_type
        self.vision = VisionProcessor()
        self.capture_dir = Path("/tmp/genesis_captures")
        self.capture_dir.mkdir(exist_ok=True)
    
    def capture_frame(self, save_name: Optional[str] = None) -> str:
        """
        Capture a frame from robot camera.
        Returns path to saved image.
        """
        if save_name is None:
            save_name = f"frame_{self.vision._get_timestamp()}.jpg"
        
        output_path = self.capture_dir / save_name
        
        if self.robot_type == "pi_camera":
            return self._capture_pi_camera(output_path)
        elif self.robot_type == "usb_camera":
            return self._capture_usb_camera(output_path)
        elif self.robot_type == "vector":
            return self._capture_vector_camera(output_path)
        else:
            return self._capture_generic(output_path)
    
    def _capture_pi_camera(self, output_path: Path) -> str:
        """Capture from Raspberry Pi camera module."""
        try:
            from picamera2 import Picamera2
            camera = Picamera2()
            camera.start()
            camera.capture_file(str(output_path))
            camera.stop()
            return str(output_path)
        except ImportError:
            # Fallback if picamera2 not available
            import subprocess
            subprocess.run(['raspistill', '-o', str(output_path), '-t', '1'])
            return str(output_path)
    
    def _capture_usb_camera(self, output_path: Path) -> str:
        """Capture from USB webcam."""
        try:
            import cv2
            cap = cv2.VideoCapture(0)
            ret, frame = cap.read()
            if ret:
                cv2.imwrite(str(output_path), frame)
            cap.release()
            return str(output_path)
        except ImportError:
            raise RuntimeError("OpenCV required for USB camera capture")
    
    def _capture_vector_camera(self, output_path: Path) -> str:
        """Capture from Anki Vector camera."""
        try:
            import anki_vector
            with anki_vector.Robot() as robot:
                image = robot.camera.latest_image
                image.save(str(output_path))
            return str(output_path)
        except ImportError:
            raise RuntimeError("anki_vector SDK required for Vector camera")
    
    def _capture_generic(self, output_path: Path) -> str:
        """Generic capture - try multiple methods."""
        methods = [
            self._capture_usb_camera,
            self._capture_pi_camera,
        ]
        
        for method in methods:
            try:
                return method(output_path)
            except:
                continue
        
        raise RuntimeError("No camera capture method succeeded")

class VisionCommands:
    """
    High-level vision commands that Stephen can issue to Genesis.
    Maps natural language intent to vision processing.
    """
    
    def __init__(self, eyes: RobotEyes):
        self.eyes = eyes
        self.vision = eyes.vision
    
    def look(self) -> Dict:
        """Capture and process current view."""
        frame_path = self.eyes.capture_frame()
        return self.vision.process_frame(frame_path, context="look command")
    
    def look_for(self, target: str) -> Dict:
        """Look for specific object/person/thing."""
        frame_path = self.eyes.capture_frame()
        return self.vision.process_frame(
            frame_path, 
            context=f"searching for: {target}"
        )
    
    def watch(self, duration_seconds: int = 5) -> List[Dict]:
        """Capture multiple frames over time."""
        import time
        frames = []
        interval = 1.0  # 1 second between frames
        
        for i in range(duration_seconds):
            frame_path = self.eyes.capture_frame(f"watch_{i}.jpg")
            frames.append(self.vision.process_frame(
                frame_path,
                context=f"watch sequence {i}/{duration_seconds}"
            ))
            if i < duration_seconds - 1:
                time.sleep(interval)
        
        return frames
    
    def compare_now_and_then(self, then_image_path: str) -> Dict:
        """Compare current view to a previous image."""
        now_path = self.eyes.capture_frame("now.jpg")
        return self.vision.compare_frames(then_image_path, now_path)


# Example usage and test functions
def test_vision_system():
    """Test the vision system with available hardware."""
    print("Genesis Vision System Test")
    print("=" * 50)
    
    # Try to detect available camera
    eyes = RobotEyes(robot_type="generic")
    commands = VisionCommands(eyes)
    
    print(f"Capture directory: {eyes.capture_dir}")
    print("\nAttempting to capture frame...")
    
    try:
        result = commands.look()
        print(f"✓ Frame captured: {result['path']}")
        print(f"  Timestamp: {result['timestamp']}")
        return True
    except Exception as e:
        print(f"✗ Capture failed: {e}")
        print("\nThis is expected if no camera is connected.")
        print("Connect robot hardware to test live capture.")
        return False

if __name__ == "__main__":
    test_vision_system()

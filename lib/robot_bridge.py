#!/usr/bin/env python3
"""
Genesis Robot Bridge
Network bridge between Genesis OS and robot hardware
"""

import asyncio
import json
import base64
from typing import Dict, Optional, Callable
from dataclasses import dataclass, asdict
from enum import Enum

class MessageType(Enum):
    """Types of messages between Genesis and robot."""
    VISION_FRAME = "vision_frame"
    COMMAND = "command"
    SENSOR_DATA = "sensor_data"
    STATUS = "status"
    ERROR = "error"
    ACK = "ack"

@dataclass
class RobotStatus:
    """Current robot state."""
    battery_percent: float
    connected: bool
    camera_active: bool
    motors_enabled: bool
    position: Optional[Dict[str, float]] = None  # {x, y, heading}
    sensors: Optional[Dict[str, float]] = None
    
    def to_dict(self) -> Dict:
        return asdict(self)

@dataclass
class VisionFrame:
    """Vision data from robot."""
    timestamp: int
    image_b64: str
    resolution: tuple[int, int]
    context: str = ""
    detected_objects: list = None
    
    def to_dict(self) -> Dict:
        return asdict(self)

class RobotBridgeServer:
    """
    Server running on robot hardware.
    Exposes camera and control to Genesis.
    """
    
    def __init__(self, host: str = "0.0.0.0", port: int = 8765):
        self.host = host
        self.port = port
        self.clients = set()
        self.status = RobotStatus(
            battery_percent=100.0,
            connected=False,
            camera_active=False,
            motors_enabled=False
        )
        self.command_handlers = {}
        self._setup_default_handlers()
    
    def _setup_default_handlers(self):
        """Register default command handlers."""
        self.register_command("capture_frame", self._handle_capture)
        self.register_command("move_forward", self._handle_move_forward)
        self.register_command("move_backward", self._handle_move_backward)
        self.register_command("turn_left", self._handle_turn_left)
        self.register_command("turn_right", self._handle_turn_right)
        self.register_command("stop", self._handle_stop)
        self.register_command("get_status", self._handle_get_status)
    
    def register_command(self, command: str, handler: Callable):
        """Register a command handler function."""
        self.command_handlers[command] = handler
    
    async def _handle_capture(self, params: Dict) -> Dict:
        """Capture frame from camera."""
        # This would integrate with actual camera hardware
        # For now, return mock response
        return {
            "type": MessageType.VISION_FRAME.value,
            "timestamp": self._get_timestamp(),
            "image_b64": "",  # Would contain actual image data
            "resolution": params.get("resolution", [640, 480]),
            "context": params.get("context", "")
        }
    
    async def _handle_move_forward(self, params: Dict) -> Dict:
        """Move robot forward."""
        distance = params.get("distance", 10)  # cm
        # Motor control code would go here
        return {
            "type": MessageType.ACK.value,
            "command": "move_forward",
            "distance_moved": distance,
            "status": "completed"
        }
    
    async def _handle_move_backward(self, params: Dict) -> Dict:
        """Move robot backward."""
        distance = params.get("distance", 10)
        return {
            "type": MessageType.ACK.value,
            "command": "move_backward",
            "distance_moved": distance,
            "status": "completed"
        }
    
    async def _handle_turn_left(self, params: Dict) -> Dict:
        """Turn robot left."""
        degrees = params.get("degrees", 90)
        return {
            "type": MessageType.ACK.value,
            "command": "turn_left",
            "degrees_turned": degrees,
            "status": "completed"
        }
    
    async def _handle_turn_right(self, params: Dict) -> Dict:
        """Turn robot right."""
        degrees = params.get("degrees", 90)
        return {
            "type": MessageType.ACK.value,
            "command": "turn_right",
            "degrees_turned": degrees,
            "status": "completed"
        }
    
    async def _handle_stop(self, params: Dict) -> Dict:
        """Emergency stop."""
        # Stop all motors immediately
        return {
            "type": MessageType.ACK.value,
            "command": "stop",
            "status": "stopped"
        }
    
    async def _handle_get_status(self, params: Dict) -> Dict:
        """Return current robot status."""
        return {
            "type": MessageType.STATUS.value,
            "status": self.status.to_dict()
        }
    
    async def handle_message(self, message: Dict) -> Dict:
        """Process incoming message from Genesis."""
        msg_type = message.get("type")
        
        if msg_type == MessageType.COMMAND.value:
            command = message.get("command")
            params = message.get("params", {})
            
            if command in self.command_handlers:
                try:
                    return await self.command_handlers[command](params)
                except Exception as e:
                    return {
                        "type": MessageType.ERROR.value,
                        "error": str(e),
                        "command": command
                    }
            else:
                return {
                    "type": MessageType.ERROR.value,
                    "error": f"Unknown command: {command}"
                }
        
        return {
            "type": MessageType.ERROR.value,
            "error": f"Unknown message type: {msg_type}"
        }
    
    def _get_timestamp(self) -> int:
        """Get current timestamp in ms."""
        import time
        return int(time.time() * 1000)
    
    async def start(self):
        """Start the bridge server."""
        # This would use websockets or HTTP server
        # Simplified version for now
        print(f"Robot bridge server starting on {self.host}:{self.port}")
        print("Waiting for Genesis connection...")
        # In real implementation: await websockets.serve(self.handler, self.host, self.port)

class GenesisBridgeClient:
    """
    Client running on Genesis OS.
    Connects to robot and issues commands.
    """
    
    def __init__(self, robot_host: str, robot_port: int = 8765):
        self.robot_host = robot_host
        self.robot_port = robot_port
        self.connected = False
        self.websocket = None
    
    async def connect(self):
        """Establish connection to robot."""
        # Would use websockets.connect() in real implementation
        print(f"Connecting to robot at {self.robot_host}:{self.robot_port}")
        self.connected = True
        return True
    
    async def send_command(self, command: str, params: Dict = None) -> Dict:
        """Send command to robot and wait for response."""
        if not self.connected:
            raise RuntimeError("Not connected to robot")
        
        message = {
            "type": MessageType.COMMAND.value,
            "command": command,
            "params": params or {},
            "timestamp": self._get_timestamp()
        }
        
        # In real implementation: await self.websocket.send(json.dumps(message))
        # For now, return mock response
        return {
            "type": MessageType.ACK.value,
            "command": command,
            "status": "completed"
        }
    
    async def capture_frame(self, context: str = "") -> VisionFrame:
        """Request camera frame from robot."""
        response = await self.send_command("capture_frame", {
            "resolution": [640, 480],
            "context": context
        })
        
        return VisionFrame(
            timestamp=response.get("timestamp", 0),
            image_b64=response.get("image_b64", ""),
            resolution=tuple(response.get("resolution", [640, 480])),
            context=context
        )
    
    async def move(self, direction: str, amount: float):
        """Move robot in specified direction."""
        if direction in ["forward", "backward"]:
            return await self.send_command(f"move_{direction}", {"distance": amount})
        elif direction in ["left", "right"]:
            return await self.send_command(f"turn_{direction}", {"degrees": amount})
        else:
            raise ValueError(f"Unknown direction: {direction}")
    
    async def stop(self):
        """Emergency stop."""
        return await self.send_command("stop")
    
    async def get_status(self) -> RobotStatus:
        """Get current robot status."""
        response = await self.send_command("get_status")
        status_data = response.get("status", {})
        return RobotStatus(**status_data)
    
    def _get_timestamp(self) -> int:
        import time
        return int(time.time() * 1000)
    
    async def disconnect(self):
        """Close connection to robot."""
        if self.websocket:
            await self.websocket.close()
        self.connected = False

class GenesisRobotController:
    """
    High-level interface for Genesis agents to control robot.
    Abstracts network communication and provides semantic commands.
    """
    
    def __init__(self, robot_host: str):
        self.client = GenesisBridgeClient(robot_host)
        self.last_frame = None
        self.autonomous_mode = False
    
    async def initialize(self):
        """Connect to robot and verify status."""
        await self.client.connect()
        status = await self.client.get_status()
        print(f"Robot connected. Battery: {status.battery_percent}%")
        return status
    
    async def look(self, context: str = "general observation") -> VisionFrame:
        """Capture and return current view."""
        frame = await self.client.capture_frame(context)
        self.last_frame = frame
        return frame
    
    async def look_for(self, target: str) -> VisionFrame:
        """Look for specific target."""
        return await self.look(context=f"searching for: {target}")
    
    async def explore(self, duration: int = 30):
        """Autonomous exploration mode."""
        print(f"Starting exploration for {duration} seconds...")
        self.autonomous_mode = True
        
        # Simple exploration: move forward, look, turn, repeat
        import time
        start_time = time.time()
        
        while time.time() - start_time < duration and self.autonomous_mode:
            # Look around
            frame = await self.look("exploration")
            
            # Move forward a bit
            await self.client.move("forward", 20)
            await asyncio.sleep(1)
            
            # Turn to scan
            await self.client.move("left", 45)
            await asyncio.sleep(0.5)
        
        await self.client.stop()
        self.autonomous_mode = False
        print("Exploration complete")
    
    async def navigate_to_target(self, target: str):
        """Navigate toward target object."""
        # This would use vision processing to guide movement
        # Simplified version:
        for _ in range(5):
            frame = await self.look_for(target)
            # Would analyze frame.image_b64 to determine direction
            # For now, just move forward
            await self.client.move("forward", 10)
            await asyncio.sleep(1)
    
    async def emergency_stop(self):
        """Stop all movement immediately."""
        self.autonomous_mode = False
        await self.client.stop()
    
    async def shutdown(self):
        """Clean shutdown."""
        await self.emergency_stop()
        await self.client.disconnect()

# Example usage and testing
async def test_bridge():
    """Test the bridge system."""
    print("Genesis Robot Bridge Test")
    print("=" * 50)
    
    # In real usage, this would be robot's IP address
    controller = GenesisRobotController("192.168.1.100")
    
    try:
        # Connect
        status = await controller.initialize()
        print(f"✓ Connected to robot")
        print(f"  Battery: {status.battery_percent}%")
        
        # Test vision
        print("\nTesting vision...")
        frame = await controller.look("test capture")
        print(f"✓ Frame captured at {frame.timestamp}")
        
        # Test movement
        print("\nTesting movement...")
        await controller.client.move("forward", 10)
        print("✓ Moved forward")
        
        await controller.client.move("left", 90)
        print("✓ Turned left")
        
        # Test stop
        print("\nTesting emergency stop...")
        await controller.emergency_stop()
        print("✓ Stopped")
        
    except Exception as e:
        print(f"✗ Error: {e}")
    finally:
        await controller.shutdown()
        print("\n✓ Test complete")

if __name__ == "__main__":
    # Run test
    print("Note: This test uses mock connections.")
    print("Real robot hardware required for actual operation.\n")
    asyncio.run(test_bridge())

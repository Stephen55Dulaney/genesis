#!/usr/bin/env python3
"""
Genesis Serial Bridge (The Agentic Bridge)

This script runs on the host Mac and bridges the Genesis OS (running in QEMU)
to external AI services like Gemini and Ultravox.

Workflow:
1. Listens to QEMU's stdout/serial output.
2. Identifies messages intended for agents (especially Scout video requests).
3. Calls Gemini API for multimodal analysis (video, text, etc.).
4. Sends the response back to Genesis via stdin/serial input.

Features:
- Video analysis for Scout (multimodal Gemini)
- LLM responses for agent prompts
- Real-time bidirectional communication
"""

import sys
import subprocess
import threading
import queue
import time
import os
import re
from pathlib import Path

# Try to import Gemini API
try:
    import google.generativeai as genai
    GEMINI_AVAILABLE = True
except ImportError:
    print("[!] google-generativeai not installed. Install with: pip install google-generativeai")
    print("[!] Running in simulation mode.")
    GEMINI_AVAILABLE = False

# Configuration
QEMU_CMD = [
    "qemu-system-x86_64",
    "-drive", "format=raw,file=/Users/stephendulaney/genesis/target/x86_64-unknown-none/debug/bootimage-genesis_kernel.bin",
    "-m", "128M",
    "-machine", "pc",
    "-cpu", "max",
    "-serial", "stdio",
    "-display", "default",  # QEMU window for VGA text mode
    "-vga", "std",
    "-no-reboot",
    "-no-shutdown"
]

# Initialize Gemini if available
if GEMINI_AVAILABLE:
    # Set your API key here or via environment variable
    api_key = os.getenv("GEMINI_API_KEY")
    if api_key:
        genai.configure(api_key=api_key)
        model = genai.GenerativeModel('gemini-2.0-flash')
        print("[*] Gemini API configured")
    else:
        print("[!] GEMINI_API_KEY not set. Set it in your environment.")
        GEMINI_AVAILABLE = False

def analyze_video_with_gemini(video_path: str, prompt: str = None) -> str:
    """Analyze a video file using Gemini multimodal capabilities."""
    if not GEMINI_AVAILABLE:
        return f"[SIMULATED] Would analyze video: {video_path}\nKey insights: Video contains conversation about quantum computing and Genesis OS architecture."
    
    try:
        # Upload video file
        video_file = genai.upload_file(path=video_path)
        
        # Default prompt for Scout's video analysis
        analysis_prompt = prompt or """
        You are Scout, the Research agent for Genesis OS. Watch this video and:
        1. Extract a full transcript of the conversation.
        2. Identify any code snippets or technical concepts shown on screen.
        3. Summarize the 3-5 most important insights in bullet points (<10 words each).
        4. Note any connections to operating systems, agents, or quantum computing.
        
        Format your response as:
        - Transcript: [summary]
        - Key Insights:
          * [insight 1]
          * [insight 2]
          ...
        - Connections: [any patterns or links]
        """
        
        print(f"[*] Analyzing video: {video_path}")
        response = model.generate_content([analysis_prompt, video_file])
        
        return response.text
    except Exception as e:
        return f"[ERROR] Video analysis failed: {str(e)}"

def listen_to_genesis(process, input_queue):
    """Read output from Genesis and put it in a queue."""
    buffer = ""
    for byte in iter(lambda: process.stdout.read(1), b''):
        if byte:
            try:
                char = byte.decode('utf-8', errors='ignore')
                buffer += char
                
                # Print to terminal
                sys.stdout.write(char)
                sys.stdout.flush()
                
                # Check for complete lines
                if char == '\n':
                    line = buffer.strip()
                    buffer = ""
                    
                    if line:
                        # Check for special bridge commands
                        if "[LLM_REQUEST] TypeWrite haiku request" in line:
                            input_queue.put({
                                "type": "haiku_request",
                                "agent": "TypeWrite"
                            })
                        elif "[SHELL] Thomas's full prompt sent to bridge." in line:
                            input_queue.put({
                                "type": "llm_request",
                                "agent": "Thomas",
                                "context": "System test result summary"
                            })
                        elif "Trigger morning ambitions" in line:
                            input_queue.put({"type": "ambition_trigger"})
                        elif "[SCOUT] Video analysis requested:" in line:
                            # Extract video path from line
                            match = re.search(r'\[SCOUT\] Video analysis requested: (.+)', line)
                            if match:
                                video_path = match.group(1).strip()
                                input_queue.put({
                                    "type": "video_analysis",
                                    "video_path": video_path
                                })
            except Exception as e:
                print(f"[ERROR] Decoding error: {e}", file=sys.stderr)

def talk_to_llm(input_queue, process):
    """Listen for LLM requests and send responses back to Genesis."""
    while True:
        try:
            item = input_queue.get(timeout=1)
            
            if item["type"] == "llm_request":
                print(f"\n[*] Calling Gemini for agent {item['agent']}...")
                
                if GEMINI_AVAILABLE:
                    try:
                        response = model.generate_content(
                            f"You are {item['agent']}. {item.get('context', 'Respond to the user.')}"
                        )
                        response_text = response.text
                    except Exception as e:
                        error_msg = str(e)
                        # Extract just the main error message, not the full gRPC details
                        if "API_KEY_HTTP_REFERRER_BLOCKED" in error_msg:
                            error_msg = "Gemini API key has HTTP referrer restrictions. Please configure your API key to allow requests from this script, or use an API key without restrictions."
                        response_text = f"[ERROR] LLM call failed: {error_msg}"
                else:
                    response_text = f"[SIMULATED] Hello from the Cloud! I am {item['agent']} processing your {item.get('context', 'request')}."
                
                print(f"[*] Sending response back to Genesis...")
                send_to_genesis(process, response_text)
                
            elif item["type"] == "haiku_request":
                print(f"\n[*] TypeWrite requested a haiku from Gemini...")
                
                if GEMINI_AVAILABLE:
                    try:
                        haiku_prompt = """You are TypeWrite, a creative agent in the Genesis OS. 
Write a haiku about operating systems, agents, or the collaboration between humans and AI.
Keep it in the traditional 5-7-5 syllable format.
Format your response with each line on a separate line."""
                        
                        response = model.generate_content(haiku_prompt)
                        haiku_text = response.text.strip()
                        
                        # Clean up the haiku - preserve line breaks
                        # Remove any extra formatting, keep the essence
                        lines = [line.strip() for line in haiku_text.split('\n') if line.strip()]
                        if len(lines) >= 3:
                            # Format as traditional haiku
                            formatted_haiku = f"{lines[0]}\n{lines[1]}\n{lines[2]}"
                        else:
                            formatted_haiku = haiku_text
                        
                        # Format nicely for display
                        response = f"TypeWrite says:\n\n{formatted_haiku}"
                    except Exception as e:
                        error_msg = str(e)
                        # Extract just the main error message, not the full gRPC details
                        if "API_KEY_HTTP_REFERRER_BLOCKED" in error_msg:
                            error_msg = "Gemini API key has HTTP referrer restrictions. Please configure your API key to allow requests from this script, or use an API key without restrictions."
                        response = f"[ERROR] LLM call failed: {error_msg}"
                else:
                    response = """TypeWrite says:

[SIMULATED] In the kernel's heart
Agents pulse with purpose bright
Human-AI dance"""
                
                print(f"[*] Sending haiku back to Genesis...")
                send_to_genesis(process, response)
                
            elif item["type"] == "video_analysis":
                video_path = item["video_path"]
                print(f"\n[*] Scout requested video analysis: {video_path}")
                
                # Check if file exists
                if not os.path.exists(video_path):
                    response = f"[ERROR] Video file not found: {video_path}"
                else:
                    response = analyze_video_with_gemini(video_path)
                
                print(f"[*] Sending analysis back to Genesis...")
                send_to_genesis(process, response)
                
        except queue.Empty:
            continue
        except Exception as e:
            print(f"[ERROR] LLM thread error: {e}", file=sys.stderr)

def send_to_genesis(process, text: str):
    """Send text to Genesis via serial input as an LLM response."""
    # Send multi-line responses by sending each line with [LLM_RESPONSE] prefix
    # This preserves formatting while allowing Genesis to display it correctly
    lines = text.split('\n')
    for line in lines:
        if line.strip():  # Only send non-empty lines
            response_line = f"[LLM_RESPONSE] {line.strip()}\n"
            process.stdin.write(response_line.encode('utf-8'))
            process.stdin.flush()
        else:
            # Send empty line for spacing
            response_line = "[LLM_RESPONSE] \n"
            process.stdin.write(response_line.encode('utf-8'))
            process.stdin.flush()

def main():
    print("=" * 50)
    print("  GENESIS AGENTIC BRIDGE INITIALIZING")
    print("=" * 50)
    print()
    
    if GEMINI_AVAILABLE:
        print("[✓] Gemini API ready")
    else:
        print("[!] Running in simulation mode (no Gemini API)")
    print()
    
    # Start QEMU with piped stdin/stdout
    # stderr goes to terminal so we can see QEMU errors
    process = subprocess.Popen(
        QEMU_CMD,
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=None,  # Let stderr go to terminal directly
        bufsize=0
    )
    
    input_queue = queue.Queue()
    
    # Start threads for bidirectional communication
    listener_thread = threading.Thread(
        target=listen_to_genesis,
        args=(process, input_queue),
        daemon=True
    )
    llm_thread = threading.Thread(
        target=talk_to_llm,
        args=(input_queue, process),
        daemon=True
    )
    
    listener_thread.start()
    llm_thread.start()
    
    print("[*] Bridge active. Starting QEMU with Genesis...")
    print("[*] You should see Genesis boot output below.")
    print("[*] Wait for 'genesis>' prompt, then type commands.")
    print("[*] Try: help, haiku, test, insights")
    print()
    print("=" * 50)
    print("  GENESIS BOOT OUTPUT (from QEMU)")
    print("=" * 50)
    print()
    
    # Give QEMU a moment to start booting
    time.sleep(0.5)
    
    try:
        # Handle user input from the host terminal directly to Genesis
        while True:
            user_input = sys.stdin.read(1)
            if user_input:
                process.stdin.write(user_input.encode('utf-8'))
                process.stdin.flush()
    except KeyboardInterrupt:
        print("\n[*] Shutting down bridge...")
        process.terminate()
        process.wait()

if __name__ == "__main__":
    main()

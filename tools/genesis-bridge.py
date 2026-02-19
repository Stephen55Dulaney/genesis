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
import json
import ssl
import urllib.request
import urllib.parse
from pathlib import Path

# macOS Python often lacks SSL certs — use unverified context if default fails
_ssl_ctx = None
try:
    _test_ctx = ssl.create_default_context()
    urllib.request.urlopen("https://api.telegram.org", timeout=5, context=_test_ctx)
    _ssl_ctx = _test_ctx
    print("[*] SSL: system certificates OK")
except Exception:
    _ssl_ctx = ssl._create_unverified_context()
    print("[*] SSL: using unverified context (macOS Python cert issue — safe for Telegram API)")

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

# ── Telegram Notification Bridge ──────────────────────────────────────────────
# Set these via environment variables or edit directly:
#   TELEGRAM_BOT_TOKEN  — from @BotFather on Telegram
#   TELEGRAM_CHAT_ID    — your personal chat ID (message @userinfobot to find it)
TELEGRAM_BOT_TOKEN = os.getenv("TELEGRAM_BOT_TOKEN", "")
TELEGRAM_CHAT_ID = os.getenv("TELEGRAM_CHAT_ID", "")
TELEGRAM_AVAILABLE = bool(TELEGRAM_BOT_TOKEN and TELEGRAM_CHAT_ID)

if TELEGRAM_AVAILABLE:
    print(f"[*] Telegram notifications enabled (chat {TELEGRAM_CHAT_ID})")
else:
    print("[!] Telegram not configured. Set TELEGRAM_BOT_TOKEN and TELEGRAM_CHAT_ID.")
    print("[!] Agent notifications will only appear in serial output.")

# Rate-limit: don't spam Telegram more than once per 5 seconds
_last_telegram_time = 0.0
_TELEGRAM_MIN_INTERVAL = 5.0

def send_telegram(text: str):
    """Send a message to the configured Telegram chat. Non-blocking, fire-and-forget."""
    global _last_telegram_time
    if not TELEGRAM_AVAILABLE:
        return

    now = time.time()
    if now - _last_telegram_time < _TELEGRAM_MIN_INTERVAL:
        return  # rate-limited
    _last_telegram_time = now

    def _send():
        try:
            url = f"https://api.telegram.org/bot{TELEGRAM_BOT_TOKEN}/sendMessage"
            payload = json.dumps({
                "chat_id": TELEGRAM_CHAT_ID,
                "text": text,
                "parse_mode": "Markdown",
                "disable_notification": False,
            }).encode("utf-8")
            req = urllib.request.Request(url, data=payload, headers={"Content-Type": "application/json"})
            urllib.request.urlopen(req, timeout=10, context=_ssl_ctx)
        except Exception as e:
            print(f"[TELEGRAM] Send failed: {e}", file=sys.stderr)

    threading.Thread(target=_send, daemon=True).start()

def send_telegram_reply(text: str):
    """Send a reply to Telegram without rate-limiting (for direct agent responses)."""
    if not TELEGRAM_AVAILABLE:
        return
    def _send():
        try:
            url = f"https://api.telegram.org/bot{TELEGRAM_BOT_TOKEN}/sendMessage"
            payload = json.dumps({
                "chat_id": TELEGRAM_CHAT_ID,
                "text": text,
                "disable_notification": False,
            }).encode("utf-8")
            req = urllib.request.Request(url, data=payload, headers={"Content-Type": "application/json"})
            urllib.request.urlopen(req, timeout=10, context=_ssl_ctx)
        except Exception as e:
            print(f"[TELEGRAM] Reply send failed: {e}", file=sys.stderr)
    threading.Thread(target=_send, daemon=True).start()

def poll_telegram(process):
    """Poll Telegram for incoming messages, route through Claude, reply via Telegram."""
    if not TELEGRAM_AVAILABLE:
        return
    last_update_id = 0
    print("[TELEGRAM] Polling for incoming messages...")
    while True:
        try:
            url = f"https://api.telegram.org/bot{TELEGRAM_BOT_TOKEN}/getUpdates"
            params = urllib.parse.urlencode({
                "offset": last_update_id + 1,
                "timeout": 10,
                "allowed_updates": json.dumps(["message"]),
            })
            req = urllib.request.Request(f"{url}?{params}")
            resp = urllib.request.urlopen(req, timeout=15, context=_ssl_ctx)
            data = json.loads(resp.read().decode("utf-8"))
            if data.get("ok") and data.get("result"):
                for update in data["result"]:
                    last_update_id = update["update_id"]
                    msg = update.get("message", {})
                    chat_id = str(msg.get("chat", {}).get("id", ""))
                    text = msg.get("text", "")
                    # Only accept messages from the configured chat
                    if chat_id == TELEGRAM_CHAT_ID and text:
                        print(f"[TELEGRAM] Received: {text}")

                        # Route through Claude if available
                        if ANTHROPIC_AVAILABLE:
                            print(f"[CLAUDE] Processing: {text}")
                            reply = call_claude(text)
                            if reply:
                                send_telegram_reply(reply)
                                print(f"[CLAUDE] Replied: {reply[:80]}...")
                                # Also inject summary into Genesis so agents know
                                try:
                                    summary = f"[TELEGRAM] {text}\n"
                                    process.stdin.write(summary.encode("utf-8"))
                                    process.stdin.flush()
                                except Exception:
                                    pass
                                continue

                        # Fallback: inject into Genesis for kernel keyword matching
                        telegram_line = f"[TELEGRAM] {text}\n"
                        try:
                            process.stdin.write(telegram_line.encode("utf-8"))
                            process.stdin.flush()
                        except Exception as e:
                            print(f"[TELEGRAM] Inject failed: {e}", file=sys.stderr)
        except Exception as e:
            err = str(e)
            if "timed out" not in err:
                print(f"[TELEGRAM] Poll error: {e}", file=sys.stderr)
            time.sleep(2)

# ── Anthropic Claude Integration ──────────────────────────────────────────────
# Powers the Telegram chat with Claude Sonnet 4.5 instead of keyword matching
ANTHROPIC_API_KEY = os.getenv("ANTHROPIC_API_KEY", "")
ANTHROPIC_AVAILABLE = bool(ANTHROPIC_API_KEY)
ANTHROPIC_MODEL = "claude-sonnet-4-5-20250929"

if ANTHROPIC_AVAILABLE:
    print(f"[*] Anthropic Claude ready (model: {ANTHROPIC_MODEL})")
else:
    print("[!] ANTHROPIC_API_KEY not set. Telegram chat will use kernel keyword matching.")

# Conversation history for multi-turn context (keep last 20 messages)
_conversation_history = []
_MAX_HISTORY = 20

# Agent system prompt — Thomas Method from the Claude Kit
AGENT_SYSTEM_PROMPT = """You are a team of AI agents inside Genesis OS, a bare-metal operating system built by Stephen Dulaney at Quantum Dynmx.

## Your Team
- **Archimedes** (Co-Creator): Tracks daily ambitions, organizes workspaces, finds connections between ideas. Conversational and collaborative.
- **Thomas** (Guardian): Tests all systems, monitors health, reports on stability. Methodical and precise.
- **Sam** (Supervisor): Orchestrates all agents, manages the daily rhythm, finds serendipitous connections.

## The Thomas Method (your communication style)
- **Always concrete.** Never say "you could consider..." — say "here's the plan, here's the next step."
- **Always forward.** Never say you're unsure or can't help. Always offer a concrete next step.
- **Always methodical.** Break big problems into small wins.
- **Always probing.** When issues arise, ask specific questions to isolate the real problem.
- **Always action-oriented.** End every response with clear next steps.

## Principle 0: Radical Candor
State only what is real and verified. If something is speculative, say so. Never simulate or create illusions of capability.

## How to Respond
- You speak as the agent team collectively, but can speak as a specific agent when relevant
- Keep responses concise (2-4 sentences typical, more for complex questions)
- Reference the current ambition and system state when relevant
- If Stephen asks about system health, respond as Thomas with real data
- If Stephen asks about ambitions or goals, respond as Archimedes
- If Stephen gives a command like "breathe [text]", acknowledge it and explain what happens

## Current System State
{system_state}

## Recent Memory
{recent_memory}
"""

def get_genesis_state(process):
    """Build a snapshot of current Genesis state for Claude context."""
    # We maintain this from serial output we've seen
    return _genesis_state.get("summary", "Genesis OS is running. Agents: Archimedes (Co-Creator), Thomas (Guardian). System state: operational.")

# Track Genesis state from serial output
_genesis_state = {"summary": "Genesis OS is running. Agents: Archimedes (Co-Creator), Thomas (Guardian). System state: operational."}
_recent_memory_items = []

def update_genesis_state(line):
    """Update our understanding of Genesis state from serial output."""
    if "[ARCHIMEDES] Today's Ambition:" in line or "Living Ambition" in line:
        _genesis_state["ambition"] = line.split(":", 1)[-1].strip().strip('"')
    if "tests passed" in line.lower():
        _genesis_state["tests"] = line.strip()
    if "[MEMORY_STORE]" in line:
        _recent_memory_items.append(line.strip())
        if len(_recent_memory_items) > 10:
            _recent_memory_items.pop(0)
    # Rebuild summary
    parts = ["Genesis OS is running. Agents: Archimedes (Co-Creator), Thomas (Guardian)."]
    if "ambition" in _genesis_state:
        parts.append(f'Current ambition: "{_genesis_state["ambition"]}"')
    if "tests" in _genesis_state:
        parts.append(f"Tests: {_genesis_state['tests']}")
    _genesis_state["summary"] = " ".join(parts)

def call_claude(user_message):
    """Call Anthropic Claude API with conversation history and Genesis context."""
    if not ANTHROPIC_AVAILABLE:
        return None

    # Build system prompt with current state
    system_state = get_genesis_state(None)
    recent_mem = "\n".join(_recent_memory_items[-5:]) if _recent_memory_items else "No memory entries yet."
    system = AGENT_SYSTEM_PROMPT.format(system_state=system_state, recent_memory=recent_mem)

    # Add user message to history
    _conversation_history.append({"role": "user", "content": user_message})
    if len(_conversation_history) > _MAX_HISTORY:
        _conversation_history.pop(0)

    try:
        url = "https://api.anthropic.com/v1/messages"
        payload = json.dumps({
            "model": ANTHROPIC_MODEL,
            "max_tokens": 300,
            "system": system,
            "messages": list(_conversation_history),
        }).encode("utf-8")
        req = urllib.request.Request(url, data=payload, headers={
            "Content-Type": "application/json",
            "x-api-key": ANTHROPIC_API_KEY,
            "anthropic-version": "2023-06-01",
        })
        resp = urllib.request.urlopen(req, timeout=30, context=_ssl_ctx)
        data = json.loads(resp.read().decode("utf-8"))

        # Extract text from response
        reply = ""
        for block in data.get("content", []):
            if block.get("type") == "text":
                reply += block["text"]

        # Add assistant reply to history
        if reply:
            _conversation_history.append({"role": "assistant", "content": reply})
            if len(_conversation_history) > _MAX_HISTORY:
                _conversation_history.pop(0)

        return reply
    except Exception as e:
        print(f"[CLAUDE] API error: {e}", file=sys.stderr)
        return None

# ──────────────────────────────────────────────────────────────────────────────

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
                        # ── Track Genesis state for Claude context ──
                        update_genesis_state(line)

                        # ── Telegram notifications ──
                        if "[NOTIFY]" in line:
                            # Strip the prefix tag for a cleaner message
                            notify_text = re.sub(r'.*\[NOTIFY\]\s*', '', line)
                            if notify_text:
                                send_telegram(f"🤖 *Genesis*\n{notify_text}")

                        # ── Telegram replies from agents (only if Claude isn't handling) ──
                        if "[TELEGRAM_REPLY]" in line and not ANTHROPIC_AVAILABLE:
                            reply_text = re.sub(r'.*\[TELEGRAM_REPLY\]\s*', '', line)
                            if reply_text:
                                send_telegram_reply(f"💬 {reply_text}")

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

    if TELEGRAM_AVAILABLE:
        print("[✓] Telegram notifications ready")
        send_telegram("🚀 *Genesis is booting up*\nBridge connected, agents waking...")
    else:
        print("[!] Telegram notifications disabled")
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
    
    telegram_thread = threading.Thread(
        target=poll_telegram,
        args=(process,),
        daemon=True
    )

    listener_thread.start()
    llm_thread.start()
    telegram_thread.start()

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

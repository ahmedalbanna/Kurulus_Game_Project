#!/usr/bin/env python3
"""
bevy-debugger-bridge.py — stdio-to-TCP bridge for bevy-debugger-mcp.

bevy-debugger-mcp v0.1.8 has stdio mode marked as "not fully implemented".
TCP mode (--tcp) works correctly. This bridge:
  1. Starts bevy-debugger-mcp --tcp as a background subprocess
  2. Connects to its TCP port (3000 by default, configurable via MCP_PORT env)
  3. Forwards stdin (JSON-RPC) → TCP and TCP → stdout
  4. Cleans up the subprocess on exit

Usage (configured in opencode.json):
  "command": ["python3", "scripts/bevy-debugger-bridge.py"]

Environment variables:
  BEVY_DEBUGGER_PATH  — path to bevy-debugger-mcp binary (default: auto-detect)
  MCP_PORT            — TCP port for bevy-debugger-mcp (default: 3000)
  BEVY_BRP_HOST       — Bevy Remote Protocol host (default: localhost)
  BEVY_BRP_PORT       — Bevy Remote Protocol port (default: 15702)
"""

import json
import os
import select
import signal
import socket
import subprocess
import sys
import threading
import time

# ─── Configuration ───────────────────────────────────────────────────────────

MCP_PORT = int(os.environ.get("MCP_PORT", "3000"))
BEVY_BRP_HOST = os.environ.get("BEVY_BRP_HOST", "localhost")
BEVY_BRP_PORT = os.environ.get("BEVY_BRP_PORT", "15702")
RUST_LOG = os.environ.get("RUST_LOG", "error")

# Resolve the bevy-debugger-mcp binary path
DEBUGGER_BIN = os.environ.get("BEVY_DEBUGGER_PATH")
if not DEBUGGER_BIN:
    candidates = [
        os.path.expanduser("~/.cargo/bin/bevy-debugger-mcp"),
        "/teamspace/studios/this_studio/.cargo/bin/bevy-debugger-mcp",
    ]
    for c in candidates:
        if os.path.isfile(c) and os.access(c, os.X_OK):
            DEBUGGER_BIN = c
            break
    if not DEBUGGER_BIN:
        # fall back to PATH lookup
        import shutil

        DEBUGGER_BIN = shutil.which("bevy-debugger-mcp")
        if not DEBUGGER_BIN:
            print(
                "bevy-debugger-mcp: binary not found. "
                "Set BEVY_DEBUGGER_PATH or install via: cargo install bevy_debugger_mcp",
                file=sys.stderr,
            )
            sys.exit(1)

# ─── Global state ────────────────────────────────────────────────────────────

debugger_process: subprocess.Popen | None = None
tcp_socket: socket.socket | None = None
shutdown_event = threading.Event()


# ─── Signal handling ─────────────────────────────────────────────────────────


def handle_signal(signum, frame):
    """Forward termination signals to the debugger subprocess."""
    shutdown_event.set()
    if debugger_process and debugger_process.poll() is None:
        debugger_process.terminate()
        # give it a moment, then kill if needed
        try:
            debugger_process.wait(timeout=3)
        except subprocess.TimeoutExpired:
            debugger_process.kill()
    sys.exit(0)


signal.signal(signal.SIGTERM, handle_signal)
signal.signal(signal.SIGINT, handle_signal)


# ─── Subprocess management ───────────────────────────────────────────────────


def start_debugger() -> subprocess.Popen:
    """Launch bevy-debugger-mcp in TCP mode."""
    env = os.environ.copy()
    env.update(
        {
            "BEVY_BRP_HOST": BEVY_BRP_HOST,
            "BEVY_BRP_PORT": BEVY_BRP_PORT,
            "MCP_PORT": str(MCP_PORT),
            "RUST_LOG": RUST_LOG,
        }
    )

    proc = subprocess.Popen(
        [DEBUGGER_BIN, "--tcp"],
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        env=env,
    )
    return proc


def wait_for_tcp_port(port: int, timeout: float = 10.0) -> bool:
    """Wait until the TCP server is listening."""
    deadline = time.monotonic() + timeout
    while time.monotonic() < deadline:
        try:
            s = socket.create_connection(("127.0.0.1", port), timeout=0.5)
            s.close()
            return True
        except (ConnectionRefusedError, OSError):
            time.sleep(0.1)
    return False


# ─── TCP ↔ stdio relay ───────────────────────────────────────────────────────


def forward_stderr(proc: subprocess.Popen):
    """Read stderr from the subprocess and write to our stderr (for logging)."""
    try:
        while not shutdown_event.is_set():
            line = proc.stderr.readline()
            if not line:
                break
            sys.stderr.buffer.write(line)
            sys.stderr.buffer.flush()
    except (IOError, ValueError):
        pass


def read_tcp(sock: socket.socket, bufsize: int = 65536) -> bytes:
    """Read one JSON-RPC message from TCP (newline-delimited)."""
    data = b""
    while True:
        ch = sock.recv(1)
        if not ch:
            break
        data += ch
        if ch == b"\n":
            break
    return data


def relay_stdio_to_tcp():
    """Main loop: read from stdin, forward to TCP, send response back."""
    global debugger_process, tcp_socket

    # ── 1. Start the debugger server ──────────────────────────────────────
    debugger_process = start_debugger()

    if not wait_for_tcp_port(MCP_PORT):
        print(
            f"bevy-debugger-bridge: timed out waiting for TCP port {MCP_PORT}",
            file=sys.stderr,
        )
        debugger_process.terminate()
        sys.exit(1)

    # ── 2. Connect to TCP ────────────────────────────────────────────────
    try:
        tcp_socket = socket.create_connection(("127.0.0.1", MCP_PORT), timeout=5)
    except ConnectionRefusedError as e:
        print(f"bevy-debugger-bridge: connection refused: {e}", file=sys.stderr)
        debugger_process.terminate()
        sys.exit(1)

    # ── 3. Start stderr forwarding thread ─────────────────────────────────
    stderr_thread = threading.Thread(
        target=forward_stderr, args=(debugger_process,), daemon=True
    )
    stderr_thread.start()

    # ── 4. Main relay loop ────────────────────────────────────────────────
    try:
        stdin_buf = sys.stdin.buffer
        while not shutdown_event.is_set():
            # Check if stdin has data (non-blocking)
            readable, _, exceptional = select.select(
                [stdin_buf, tcp_socket], [], [], 0.5
            )

            if exceptional:
                break

            for fd in readable:
                if fd == stdin_buf:
                    # Read a JSON-RPC message from stdin (newline-delimited)
                    line = stdin_buf.readline()
                    if not line:
                        # stdin closed
                        shutdown_event.set()
                        break
                    tcp_socket.sendall(line)

                elif fd == tcp_socket:
                    # Read response from TCP
                    response = tcp_socket.recv(65536)
                    if not response:
                        shutdown_event.set()
                        break
                    sys.stdout.buffer.write(response)
                    sys.stdout.buffer.flush()

    except (BrokenPipeError, ConnectionResetError, OSError):
        pass
    finally:
        shutdown_event.set()
        if tcp_socket:
            tcp_socket.close()
        if debugger_process and debugger_process.poll() is None:
            debugger_process.terminate()
            try:
                debugger_process.wait(timeout=3)
            except subprocess.TimeoutExpired:
                debugger_process.kill()


# ─── Entry point ─────────────────────────────────────────────────────────────

if __name__ == "__main__":
    relay_stdio_to_tcp()

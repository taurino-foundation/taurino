"""Taurino Runtime example with shared memory.

Demonstrates:
- Native Rust GUI in a spawned child process.
- Shared memory initialization with ipckit.
- Reading and writing binary data.
- Graceful application shutdown.
"""

from __future__ import annotations

import multiprocessing as mp
import time

from taurino.core import Runtime


def main() -> None:
    gui = Runtime(
        config=b"",
        name="taurino-example",
        start_timeout=15.0,
        stop_timeout=5.0,
        shared_memory_size=4096,
        method="spawn",
    )

    try:
        # Start the Rust GUI child process.
        print("[Python] Starting Taurino...")
        gui.start()

        print(f"[Python] GUI PID: {gui.pid}")
        print(f"[Python] Shared memory: {gui.shared_memory_name}")
        print(f"[Python] Shared memory size: {gui.shared_memory_size}")

        # Write to shared memory.
        message = b"Hello from Python!"

        gui.write_shared(0, message)

        print(f"[Python] Written: {message!r}")

        # Read the same data.
        result = gui.read_shared(0, len(message))

        print(f"[Python] Read: {result!r}")

        # Verify shared memory contents.
        assert result == message

        print("[Python] Shared memory verified.")
        print("[Python] Close the GUI window to exit.")

        # Wait until the Rust GUI exits.
        while gui.is_running:
            gui.join(timeout=0.5)
            time.sleep(0.01)

        print(f"[Python] GUI exit code: {gui.exitcode}")

    except KeyboardInterrupt:
        print("\n[Python] Shutdown requested...")

    finally:
        gui.close()
        print("[Python] Runtime closed.")


if __name__ == "__main__":
    mp.freeze_support()
    main()

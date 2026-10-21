"""Spawn-based GUI child lifecycle for Tao, Wry, and PyO3.

The Rust/PyO3 runtime runs on the child process's main thread.
The parent owns the ipckit shared memory and controls the GUI lifecycle.
"""

from __future__ import annotations

import multiprocessing as mp
import time
import traceback
from collections.abc import Callable
from multiprocessing.context import BaseContext
from multiprocessing.process import BaseProcess
from typing import Any, Literal, Protocol, cast
from uuid import uuid4

from ipckit import SharedMemory

from taurino.error import RuntimeStartError, RuntimeStateError

from ._taurino import runtime

DEFAULT_SHARED_MEMORY_SIZE = 4096


class StatusConnection(Protocol):
    """Minimal multiprocessing status pipe interface."""

    def send(self, obj: Any) -> None: ...
    def recv(self) -> Any: ...
    def poll(self, timeout: float = 0.0) -> bool: ...
    def close(self) -> None: ...


ProcessFactory = Callable[..., BaseProcess]


def _gui_process_entry(
    config: bytes,
    shutdown_event: Any,
    status: StatusConnection,
    shared_memory_name: str,
) -> None:
    """Run the native GUI on the child process's main thread."""

    ready_sent = False

    def notify_ready() -> None:
        nonlocal ready_sent

        if not ready_sent:
            status.send(("ready", None))
            ready_sent = True

    try:
        # Rust opens the already-created shared memory by name.
        runtime(
            config,
            shutdown_event,
            notify_ready,
            shared_memory_name,
        )

    except KeyboardInterrupt:
        shutdown_event.set()

    except BaseException:
        if not ready_sent:
            try:
                status.send(("error", traceback.format_exc()))
            except (OSError, EOFError):
                pass
        raise

    finally:
        status.close()


class Runtime:
    """Manage a native GUI process and its shared memory region."""

    def __init__(
        self,
        config: bytes,
        *,
        name: str = "taurino-gui",
        start_timeout: float = 15.0,
        stop_timeout: float = 5.0,
        shared_memory_size: int = DEFAULT_SHARED_MEMORY_SIZE,
        method: Literal["spawn", "fork"] = "spawn",
    ) -> None:
        if not isinstance(config, bytes):
            raise TypeError("GUI configuration must be bytes")

        if start_timeout <= 0 or stop_timeout < 0:
            raise ValueError("Invalid runtime timeout")

        if isinstance(shared_memory_size, bool) or not isinstance(shared_memory_size, int) or shared_memory_size <= 0:
            raise ValueError("Shared memory size must be a positive integer")

        self._config = config
        self._name = name
        self._start_timeout = start_timeout
        self._stop_timeout = stop_timeout
        self._shared_memory_size = shared_memory_size

        self._context: BaseContext = mp.get_context(method)

        self._process: BaseProcess | None = None
        self._shutdown_event: Any = self._context.Event()
        self._status_connection: StatusConnection | None = None

        self._shared_memory: SharedMemory | None = None
        self._shared_memory_name: str | None = None

    @property
    def pid(self) -> int | None:
        return self._process.pid if self._process is not None else None

    @property
    def exitcode(self) -> int | None:
        return self._process.exitcode if self._process is not None else None

    @property
    def is_running(self) -> bool:
        return self._process is not None and self._process.is_alive()

    @property
    def is_stopping(self) -> bool:
        return self.is_running and self._shutdown_event.is_set()

    @property
    def shared_memory_name(self) -> str | None:
        return self._shared_memory_name

    @property
    def shared_memory_size(self) -> int:
        return self._shared_memory_size

    @property
    def shared_memory(self) -> SharedMemory | None:
        return self._shared_memory

    def _create_shared_memory(self) -> None:
        """Create a uniquely named shared memory region."""

        if self._shared_memory is not None:
            raise RuntimeStateError("Shared memory already exists")

        # Use a shorter name to comply with macOS POSIX shared memory
        # naming limits while preserving randomness for uniqueness.
        name = f"taurino-{uuid4().hex[:12]}"

        shm = SharedMemory.create(
            name,
            self._shared_memory_size,
        )

        self._shared_memory = shm
        self._shared_memory_name = name

    def _release_shared_memory(self) -> None:
        """Release the owner's reference to the shared memory."""

        self._shared_memory = None
        self._shared_memory_name = None

    def _require_shared_memory(self) -> SharedMemory:
        shm = self._shared_memory

        if shm is None:
            raise RuntimeStateError("Shared memory is not initialized")

        return shm

    def write_shared(self, offset: int, data: bytes) -> None:
        """Write bytes to the shared memory."""

        if not isinstance(data, bytes):
            raise TypeError("Shared memory data must be bytes")

        if offset < 0 or offset + len(data) > self._shared_memory_size:
            raise ValueError("Shared memory write is out of bounds")

        self._require_shared_memory().write(offset, data)

    def read_shared(self, offset: int, size: int) -> bytes:
        """Read bytes from the shared memory."""

        if offset < 0 or size < 0 or offset + size > self._shared_memory_size:
            raise ValueError("Shared memory read is out of bounds")

        return self._require_shared_memory().read(offset, size)

    def start(self) -> None:
        """Start the native GUI and wait until initialization completes."""

        if self.is_running:
            raise RuntimeStateError("GUI runtime is already running")

        self.close()

        self._shutdown_event = self._context.Event()

        # Create shared memory before spawning the child.
        self._create_shared_memory()

        parent: StatusConnection | None = None
        child: StatusConnection | None = None

        try:
            parent, child = self._context.Pipe(duplex=False)

            process_factory = cast(
                ProcessFactory,
                getattr(self._context, "Process"),  # noqa: B009
            )

            process = process_factory(
                name=self._name,
                target=_gui_process_entry,
                args=(
                    self._config,
                    self._shutdown_event,
                    child,
                    self._shared_memory_name,
                ),
                daemon=False,
            )

            self._process = process
            self._status_connection = parent

            try:
                process.start()
            finally:
                child.close()

            self._wait_until_ready()

        except BaseException:
            if child is not None:
                child.close()

            self.close()
            raise

    def _wait_until_ready(self) -> None:
        """Wait for the GUI readiness notification."""

        connection = self._status_connection

        if connection is None:
            raise RuntimeStateError("GUI status pipe is not available")

        deadline = time.monotonic() + self._start_timeout

        while True:
            remaining = deadline - time.monotonic()

            if remaining <= 0:
                raise RuntimeStartError("GUI startup timed out")

            if connection.poll(min(remaining, 0.1)):
                try:
                    status, message = connection.recv()
                except EOFError as exc:
                    raise RuntimeStartError("GUI process disconnected during startup") from exc

                if status == "ready":
                    if not self.is_running:
                        raise RuntimeStartError("GUI exited during startup")
                    return

                if status == "error":
                    raise RuntimeStartError(f"GUI startup failed:\n{message}")

                raise RuntimeStartError(f"Unknown GUI status: {status}")

            if not self.is_running:
                raise RuntimeStartError(f"GUI exited with code {self.exitcode}")

    def request_stop(self) -> None:
        """Request graceful shutdown without blocking."""

        if self.is_running:
            self._shutdown_event.set()

    def stop(self, timeout: float | None = None) -> None:
        """Stop the child, escalating if necessary."""

        if timeout is not None and timeout < 0:
            raise ValueError("Stop timeout must not be negative")

        process = self._process

        if process is None or not process.is_alive():
            return

        self.request_stop()

        process.join(self._stop_timeout if timeout is None else timeout)

        if process.is_alive():
            process.terminate()
            process.join(2.0)

        if process.is_alive():
            process.kill()
            process.join()

    def join(self, timeout: float | None = None) -> int | None:
        """Wait for the GUI without requesting shutdown."""

        if self._process is None:
            return None

        self._process.join(timeout)
        return self._process.exitcode

    def close(self) -> None:
        """Release process, pipe, and shared memory resources."""

        self.stop()

        process = self._process

        if process is not None and process.is_alive():
            raise RuntimeStateError("Cannot release shared memory while GUI is running")

        connection = self._status_connection
        self._status_connection = None

        if connection is not None:
            connection.close()

        self._process = None

        if process is not None:
            process.close()

        # The owner releases shared memory after the child exits.
        self._release_shared_memory()

    def __enter__(self) -> Runtime:
        self.start()
        return self

    def __exit__(self, *args: object) -> None:
        self.close()

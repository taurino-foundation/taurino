"""Tests for Taurino's multiprocessing GUI runtime and shared memory."""

from __future__ import annotations

from collections.abc import Generator
from unittest.mock import MagicMock, patch

import pytest
from ipckit import SharedMemory

from taurino.core import Runtime, _gui_process_entry as gui_process_entry
from taurino.error import RuntimeStartError, RuntimeStateError

# ============================================================
# Configuration
# ============================================================


class TestRuntimeConfiguration:
    def test_default_state(self) -> None:
        runtime = Runtime(config=b"test")

        assert runtime.pid is None
        assert runtime.exitcode is None
        assert runtime.is_running is False
        assert runtime.is_stopping is False
        assert runtime.shared_memory is None
        assert runtime.shared_memory_name is None
        assert runtime.shared_memory_size == 4096

    def test_custom_shared_memory_size(self) -> None:
        runtime = Runtime(
            config=b"test",
            shared_memory_size=8192,
        )

        assert runtime.shared_memory_size == 8192

    @pytest.mark.parametrize(
        "config",
        [None, "invalid", 123, bytearray(b"test")],
    )
    def test_invalid_config(self, config: object) -> None:
        with pytest.raises(TypeError):
            Runtime(config=config)  # type: ignore[arg-type]

    @pytest.mark.parametrize(
        "size",
        [0, -1, -4096, 1.5, True, None],
    )
    def test_invalid_shared_memory_size(self, size: object) -> None:
        with pytest.raises(ValueError):
            Runtime(
                config=b"test",
                shared_memory_size=size,  # type: ignore[arg-type]
            )

    def test_invalid_timeouts(self) -> None:
        with pytest.raises(ValueError):
            Runtime(b"test", start_timeout=0)

        with pytest.raises(ValueError):
            Runtime(b"test", stop_timeout=-1)


# ============================================================
# Shared Memory
# ============================================================


class TestSharedMemory:
    @pytest.fixture
    def runtime(self) -> Generator[Runtime, None, None]:
        instance = Runtime(
            config=b"test",
            shared_memory_size=4096,
        )

        instance._create_shared_memory()

        try:
            yield instance
        finally:
            instance.close()

    def test_shared_memory_created(self, runtime: Runtime) -> None:
        assert runtime.shared_memory is not None
        assert runtime.shared_memory_name is not None
        assert runtime.shared_memory_name.startswith("taurino-")

    def test_shared_memory_write_read(self, runtime: Runtime) -> None:
        data = b"Hello Taurino!"

        runtime.write_shared(0, data)

        result = runtime.read_shared(0, len(data))

        assert result == data

    def test_shared_memory_offset(self, runtime: Runtime) -> None:
        runtime.write_shared(128, b"Python")
        runtime.write_shared(256, b"Rust")

        assert runtime.read_shared(128, 6) == b"Python"
        assert runtime.read_shared(256, 4) == b"Rust"

    def test_shared_memory_binary(self, runtime: Runtime) -> None:
        data = bytes(range(256))

        runtime.write_shared(0, data)

        assert runtime.read_shared(0, len(data)) == data

    def test_shared_memory_last_byte(self, runtime: Runtime) -> None:
        runtime.write_shared(4095, b"\xff")

        assert runtime.read_shared(4095, 1) == b"\xff"

    def test_shared_memory_out_of_bounds(self, runtime: Runtime) -> None:
        with pytest.raises(ValueError):
            runtime.write_shared(4096, b"x")

        with pytest.raises(ValueError):
            runtime.write_shared(-1, b"x")

        with pytest.raises(ValueError):
            runtime.read_shared(4095, 2)

        with pytest.raises(ValueError):
            runtime.read_shared(-1, 1)

    def test_write_rejects_non_bytes(self, runtime: Runtime) -> None:
        with pytest.raises(TypeError):
            runtime.write_shared(0, "invalid")  # type: ignore[arg-type]

    def test_shared_memory_not_initialized(self) -> None:
        runtime = Runtime(b"test")

        with pytest.raises(RuntimeStateError):
            runtime.read_shared(0, 4)

        with pytest.raises(RuntimeStateError):
            runtime.write_shared(0, b"test")

    def test_duplicate_creation(self, runtime: Runtime) -> None:
        with pytest.raises(RuntimeStateError):
            runtime._create_shared_memory()

    def test_shared_memory_name_unique(self) -> None:
        first = Runtime(b"test")
        second = Runtime(b"test")

        try:
            first._create_shared_memory()
            second._create_shared_memory()

            assert first.shared_memory_name != second.shared_memory_name

        finally:
            first.close()
            second.close()

    def test_shared_memory_reopen(self, runtime: Runtime) -> None:
        """Verify another handle can access the same region."""

        name = runtime.shared_memory_name
        assert name is not None

        runtime.write_shared(0, b"Taurino")

        second_handle = SharedMemory.open(name)

        try:
            assert second_handle.read(0, 7) == b"Taurino"

            second_handle.write(128, b"Rust")

            assert runtime.read_shared(128, 4) == b"Rust"

        finally:
            del second_handle

    def test_shared_memory_cleanup(self) -> None:
        runtime = Runtime(b"test")

        runtime._create_shared_memory()

        assert runtime.shared_memory is not None

        runtime.close()

        assert runtime.shared_memory is None
        assert runtime.shared_memory_name is None


# ============================================================
# GUI Child Entry
# ============================================================


class TestGuiProcessEntry:
    def test_runtime_receives_shared_memory_name(self) -> None:
        shutdown_event = MagicMock()
        status = MagicMock()

        def fake_runtime(
            config: bytes,
            shutdown: object,
            notify_ready: object,
            memory_name: str,
        ) -> None:
            assert config == b"config"
            assert shutdown is shutdown_event
            assert memory_name == "taurino-test"

            assert callable(notify_ready)
            notify_ready()

        with patch(
            "taurino.core.runtime",
            side_effect=fake_runtime,
            create=True,
        ):
            gui_process_entry(
                b"config",
                shutdown_event,
                status,
                "taurino-test",
            )

        status.send.assert_called_once_with(("ready", None))
        status.close.assert_called_once()

    def test_notify_ready_only_once(self) -> None:
        status = MagicMock()

        def fake_runtime(
            config: bytes,
            shutdown: object,
            notify_ready: object,
            memory_name: str,
        ) -> None:
            assert callable(notify_ready)

            notify_ready()
            notify_ready()
            notify_ready()

        with patch(
            "taurino.core.runtime",
            side_effect=fake_runtime,
            create=True,
        ):
            gui_process_entry(
                b"",
                MagicMock(),
                status,
                "taurino-test",
            )

        status.send.assert_called_once_with(("ready", None))

    def test_startup_exception(self) -> None:
        status = MagicMock()

        with (
            patch(
                "taurino.core.runtime",
                side_effect=RuntimeError("Engine failed"),
                create=True,
            ),
            pytest.raises(RuntimeError, match="Engine failed"),
        ):
            gui_process_entry(
                b"",
                MagicMock(),
                status,
                "taurino-test",
            )

        assert status.send.call_count == 1

        state, message = status.send.call_args.args[0]

        assert state == "error"
        assert "Engine failed" in message

        status.close.assert_called_once()

    def test_keyboard_interrupt(self) -> None:
        shutdown_event = MagicMock()
        status = MagicMock()

        with patch(
            "taurino.core.runtime",
            side_effect=KeyboardInterrupt,
            create=True,
        ):
            gui_process_entry(
                b"",
                shutdown_event,
                status,
                "taurino-test",
            )

        shutdown_event.set.assert_called_once()
        status.close.assert_called_once()


# ============================================================
# Runtime Lifecycle
# ============================================================


class TestRuntimeLifecycle:
    def test_startup_ready(self) -> None:
        runtime = Runtime(b"test")

        process = MagicMock()
        process.is_alive.return_value = True
        process.pid = 1234

        connection = MagicMock()
        connection.poll.return_value = True
        connection.recv.return_value = ("ready", None)

        runtime._process = process
        runtime._status_connection = connection

        runtime._wait_until_ready()

        connection.recv.assert_called_once()

    def test_startup_error(self) -> None:
        runtime = Runtime(b"test")

        process = MagicMock()
        process.is_alive.return_value = True

        connection = MagicMock()
        connection.poll.return_value = True
        connection.recv.return_value = (
            "error",
            "Native engine initialization failed",
        )

        runtime._process = process
        runtime._status_connection = connection

        with pytest.raises(
            RuntimeStartError,
            match="Native engine initialization failed",
        ):
            runtime._wait_until_ready()

    def test_startup_disconnected(self) -> None:
        runtime = Runtime(b"test")

        process = MagicMock()
        process.is_alive.return_value = True

        connection = MagicMock()
        connection.poll.return_value = True
        connection.recv.side_effect = EOFError

        runtime._process = process
        runtime._status_connection = connection

        with pytest.raises(RuntimeStartError, match="disconnected"):
            runtime._wait_until_ready()

    def test_request_stop(self) -> None:
        runtime = Runtime(b"test")

        process = MagicMock()
        process.is_alive.return_value = True

        runtime._process = process
        runtime._shutdown_event = MagicMock()

        runtime.request_stop()

        runtime._shutdown_event.set.assert_called_once()

    def test_join(self) -> None:
        runtime = Runtime(b"test")

        process = MagicMock()
        process.exitcode = 0

        runtime._process = process

        result = runtime.join(timeout=1.0)

        process.join.assert_called_once_with(1.0)

        assert result == 0

    def test_stop_rejects_negative_timeout(self) -> None:
        runtime = Runtime(b"test")

        with pytest.raises(ValueError):
            runtime.stop(timeout=-1)

    def test_stop_gracefully(self) -> None:
        runtime = Runtime(b"test")

        process = MagicMock()
        # Some Tests
        # 1. Initial check in stop()
        # 2. Check in request_stop()
        # 3. Check after graceful join()
        # 4. Final check before kill()
        process.is_alive.side_effect = [
            True,
            True,
            False,
            False,
        ]

        runtime._process = process
        runtime._shutdown_event = MagicMock()

        runtime.stop(timeout=1.0)

        runtime._shutdown_event.set.assert_called_once()
        process.join.assert_called_once_with(1.0)
        process.terminate.assert_not_called()
        process.kill.assert_not_called()

# SPDX-License-Identifier: MIT OR Apache-2.0

from __future__ import annotations

import argparse
import importlib.util
import os
import subprocess
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import Literal

Backend = Literal["pyinstaller", "nuitka"]
Mode = Literal["onefile", "onedir"]


@dataclass(frozen=True, slots=True)
class BuildConfig:
    entry: Path
    backend: Backend = "pyinstaller"
    mode: Mode = "onedir"
    name: str | None = None
    output_dir: Path = Path("dist")
    windowed: bool = False
    icon: Path | None = None
    clean: bool = False
    extra_args: tuple[str, ...] = ()


def validate_config(config: BuildConfig) -> None:
    if not config.entry.is_file():
        raise ValueError(f"Entry point does not exist: {config.entry}")

    if config.entry.suffix.lower() not in {".py", ".pyw"}:
        raise ValueError("Entry point must be a Python script.")

    if config.name is not None:
        if not config.name or config.name in {".", ".."}:
            raise ValueError("Invalid application name.")

        if "/" in config.name or "\\" in config.name:
            raise ValueError("Application name must not contain path separators.")

    if config.icon is not None and not config.icon.is_file():
        raise ValueError(f"Icon does not exist: {config.icon}")

    if config.backend == "nuitka" and config.icon:
        if os.name == "nt" and config.icon.suffix.lower() != ".ico":
            raise ValueError("Nuitka Windows builds require an .ico icon.")


def check_backend(backend: Backend) -> None:
    module = "PyInstaller" if backend == "pyinstaller" else "nuitka"

    if importlib.util.find_spec(module) is None:
        raise RuntimeError(f"{backend} is not installed.\nInstall it using: uv add --dev {backend}")


def get_nuitka_config() -> Path | None:
    config = Path(__file__).resolve().parent.parent / "_nuitka" / "taurino.nuitka-package.config.yml"

    return config if config.is_file() else None


def build_pyinstaller(config: BuildConfig) -> list[str]:
    command = [
        sys.executable,
        "-m",
        "PyInstaller",
        "--noconfirm",
        "--onefile" if config.mode == "onefile" else "--onedir",
        "--distpath",
        str(config.output_dir),
    ]

    if config.name:
        command.extend(["--name", config.name])

    if config.windowed:
        command.append("--windowed")

    if config.icon:
        command.extend(["--icon", str(config.icon)])

    if config.clean:
        command.append("--clean")

    command.extend(config.extra_args)
    command.append(str(config.entry))

    return command


def build_nuitka(config: BuildConfig) -> list[str]:
    command = [
        sys.executable,
        "-m",
        "nuitka",
        f"--mode={'onefile' if config.mode == 'onefile' else 'standalone'}",
        f"--output-dir={config.output_dir}",
        "--assume-yes-for-downloads",
    ]

    if config.name:
        command.append(f"--output-filename={config.name}")

    if config.windowed and os.name == "nt":
        command.append("--windows-console-mode=disable")

    if config.icon:
        if os.name == "nt":
            command.append(f"--windows-icon-from-ico={config.icon}")
        elif sys.platform == "darwin":
            command.append(f"--macos-app-icon={config.icon}")

    config_file = get_nuitka_config()

    if config_file:
        command.append(f"--user-package-configuration-file={config_file}")

    # Nuitka builds require a C compiler.
    # Allow Nuitka to discover the appropriate toolchain.
    command.extend(config.extra_args)
    command.append(str(config.entry))

    return command


def run_build(config: BuildConfig) -> int:
    validate_config(config)
    check_backend(config.backend)

    command = build_pyinstaller(config) if config.backend == "pyinstaller" else build_nuitka(config)

    print()
    print("=" * 60)
    print("Taurino Build")
    print("=" * 60)
    print(f"Backend : {config.backend}")
    print(f"Mode    : {config.mode}")
    print(f"Entry   : {config.entry}")
    print(f"Output  : {config.output_dir}")
    print(f"Windowed: {config.windowed}")
    print("=" * 60)
    print(flush=True)

    try:
        result = subprocess.run(
            command,
            check=False,
        )
    except KeyboardInterrupt:
        print("\nBuild interrupted.")
        return 130
    except OSError as exc:
        print(f"\nFailed to execute builder: {exc}", file=sys.stderr)
        return 1

    if result.returncode != 0:
        print(
            f"\nBuild failed (exit code {result.returncode}).",
            file=sys.stderr,
        )
        return result.returncode

    print()
    print("Build completed successfully.")
    print(f"Output directory: {config.output_dir}")

    return 0


def create_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        prog="taurino build",
        description="Build a Taurino desktop application.",
        allow_abbrev=False,
    )

    parser.add_argument(
        "entry",
        type=Path,
        help="Python application entry point.",
    )

    parser.add_argument(
        "--backend",
        choices=("pyinstaller", "nuitka"),
        default="pyinstaller",
    )

    mode = parser.add_mutually_exclusive_group()

    mode.add_argument(
        "--onefile",
        action="store_true",
        help="Create a single executable.",
    )

    mode.add_argument(
        "--onedir",
        action="store_true",
        help="Create a standalone application directory.",
    )

    parser.add_argument("--name")
    parser.add_argument("--output-dir", type=Path, default=Path("dist"))
    parser.add_argument("--icon", type=Path)
    parser.add_argument("--windowed", action="store_true")
    parser.add_argument("--clean", action="store_true")

    return parser


def main(argv: list[str] | None = None) -> int:
    parser = create_parser()
    args, extra = parser.parse_known_args(argv)

    # Forward native backend options after "--".
    if extra:
        if extra[0] != "--":
            parser.error("Unknown options. Pass backend-specific options after '--'.")
        extra = extra[1:]

    config = BuildConfig(
        entry=args.entry.resolve(),
        backend=args.backend,
        mode="onefile" if args.onefile else "onedir",
        name=args.name,
        output_dir=args.output_dir.resolve(),
        windowed=args.windowed,
        icon=args.icon.resolve() if args.icon else None,
        clean=args.clean,
        extra_args=tuple(extra),
    )

    try:
        return run_build(config)
    except (ValueError, RuntimeError) as exc:
        print(f"Error: {exc}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())

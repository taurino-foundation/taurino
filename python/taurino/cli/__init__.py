"""
Taurino Build CLI

I want to develop a dedicated build CLI for Taurino,
similar to `tauri build`, supporting PyInstaller and Nuitka.

The goal is to allow developers to package their Python
applications without manually executing compiler commands.
"""

from __future__ import annotations

from .build import BuildConfig, run_build

__all__ = ["BuildConfig", "run_build"]

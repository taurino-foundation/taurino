from __future__ import annotations

import argparse

from .build import main as build_main


def main() -> int:
    parser = argparse.ArgumentParser(
        prog="taurino",
        description="Taurino desktop application CLI.",
    )

    parser.add_argument(
        "command",
        choices=("build",),
        help="Command to execute.",
    )

    args, remaining = parser.parse_known_args()

    if args.command == "build":
        return build_main(remaining)

    return 1


"""
Nach der Installation:
uv sync --group build


Jetzt kannst du Taurino direkt ausführen:
uv run taurino build examples/basic.py --backend nuitka --onefile --windowed


Oder mit PyInstaller:
uv run taurino build examples/basic.py --backend pyinstaller --onefile --windowed


Auch zusätzliche Compileroptionen kannst du durchreichen:
uv run taurino build examples/basic.py --backend nuitka -- --jobs=4


"""
if __name__ == "__main__":
    raise SystemExit(main())

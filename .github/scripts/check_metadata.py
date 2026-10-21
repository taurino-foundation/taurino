#!/usr/bin/env python3
"""Validate the dynamic maturin/Cargo package version and workspace reference."""

import pathlib
import re
import sys
import tomllib

root = pathlib.Path(__file__).resolve().parents[2]
workspace = tomllib.loads((root / "Cargo.toml").read_text(encoding="utf8"))
member = tomllib.loads((root / "crates/taurino/Cargo.toml").read_text(encoding="utf8"))
project = tomllib.loads((root / "pyproject.toml").read_text(encoding="utf8"))
version = member["package"]["version"]
workspace_version = workspace["workspace"]["dependencies"]["taurino"]["version"]
assert re.fullmatch(r"\d+\.\d+\.\d+(?:[-+][0-9A-Za-z.-]+)?", version), version
assert version == workspace_version, f"member={version}, workspace-dep={workspace_version}"
assert project["project"]["name"] == "taurino"
assert "version" in project["project"]["dynamic"]
assert project["tool"]["maturin"]["manifest-path"] == "crates/taurino/Cargo.toml"
assert "abi3-py39" in workspace["workspace"]["dependencies"]["pyo3"]["features"]
print(f"Taurino metadata consistent: {version}")

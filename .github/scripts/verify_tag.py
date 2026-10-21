import pathlib
import sys
import tomllib

if len(sys.argv) != 2:
    raise SystemExit("Usage: verify_tag.py vX.Y.Z")
root = pathlib.Path(__file__).resolve().parents[2]
member = tomllib.loads((root / "crates/taurino/Cargo.toml").read_text(encoding="utf8"))
actual = member["package"]["version"]
if sys.argv[1] != "v" + actual:
    raise SystemExit(f"Release tag {sys.argv[1]} does not match crate version v{actual}")
print(f"Version verified: v{actual}")

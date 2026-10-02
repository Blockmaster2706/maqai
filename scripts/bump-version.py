import argparse
import json
from pathlib import Path
import re
import tomllib


def next_version(current, bump, exact=None):
    if not re.fullmatch(r"(?:0|[1-9][0-9]*)\.(?:0|[1-9][0-9]*)\.(?:0|[1-9][0-9]*)", current):
        raise ValueError("Current version must be a stable major.minor.patch version")
    parts = list(map(int, current.split(".")))
    if exact:
        if not re.fullmatch(r"(?:0|[1-9][0-9]*)\.(?:0|[1-9][0-9]*)\.(?:0|[1-9][0-9]*)", exact):
            raise ValueError("Version must be a stable major.minor.patch version")
        if tuple(map(int, exact.split("."))) <= tuple(parts):
            raise ValueError("The new version must be greater than the current version")
        return exact
    if bump == "current":
        return current
    index = {"major": 0, "minor": 1, "patch": 2}[bump]
    parts[index] += 1
    parts[index + 1:] = [0] * (2 - index)
    return ".".join(map(str, parts))


def bump_version(root, bump, exact=None, dry_run=False):
    config_path = root / "src-tauri/tauri.conf.json"
    config = json.loads(config_path.read_text())
    current = config["version"]
    files = {}
    for filename in ["Cargo.toml", "src-tauri/Cargo.toml"]:
        path = root / filename
        text = path.read_text()
        if tomllib.loads(text)["package"]["version"] != current:
            raise ValueError(f"{filename} version does not match the Tauri config")
        files[path] = text
    version = next_version(current, bump, exact)
    for path, text in files.items():
        updated, count = re.subn(r'(?m)^version = "' + re.escape(current) + r'"$', f'version = "{version}"', text, count=1)
        if count != 1:
            raise ValueError(f"Could not update the version in {path}")
        files[path] = updated
    lock_path = root / "Cargo.lock"
    lock = lock_path.read_text()
    for name in ["maqai", "maqai-ui"]:
        pattern = r'(\[\[package\]\]\nname = "' + name + r'"\nversion = ")' + re.escape(current) + '"'
        lock, count = re.subn(pattern, lambda m: m[1] + version + '"', lock)
        if count != 1:
            raise ValueError(f"Cargo.lock has no matching {name} version")
    files[lock_path] = lock
    config["version"] = version
    files[config_path] = json.dumps(config, indent=2) + "\n"
    if not dry_run and version != current:
        for path, text in files.items():
            path.write_text(text)
    return version


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="Synchronize Maqai's SemVer across all manifests")
    parser.add_argument("bump", choices=["current", "patch", "minor", "major"], nargs="?", default="patch")
    parser.add_argument("--version")
    parser.add_argument("--dry-run", action="store_true")
    args = parser.parse_args()
    try:
        print(bump_version(Path(__file__).resolve().parents[1], args.bump, args.version, args.dry_run))
    except ValueError as error:
        parser.exit(1, f"{error}\n")

import json
import os
from pathlib import Path


def configure(root, key):
    key = key.strip()
    if not key:
        raise ValueError("Set TAURI_UPDATER_PUBLIC_KEY before building a signed release")
    config = {"plugins": {"updater": {"pubkey": key}}}
    path = root / "src-tauri/tauri.release.conf.json"
    path.write_text(json.dumps(config, indent=2) + "\n")
    return path


if __name__ == "__main__":
    configure(Path(__file__).resolve().parents[1], os.environ.get("TAURI_UPDATER_PUBLIC_KEY", ""))

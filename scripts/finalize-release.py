import argparse
from datetime import datetime, timezone
import json
from pathlib import Path
import shutil

PLATFORMS = {"windows-x86_64", "linux-x86_64", "darwin-x86_64", "darwin-aarch64"}


def finalize(source, output, version):
    platforms = {}
    for path in source.glob("updater-*.json"):
        info = json.loads(path.read_text())
        if info["version"] != version or info["platform"] in platforms:
            raise ValueError("Mixed or duplicate release metadata")
        filename = info["url"].rsplit("/", 1)[-1]
        if not (source / filename).is_file() or not (source / (filename + ".sig")).is_file():
            raise ValueError(f"Missing signed artifact: {filename}")
        platforms[info["platform"]] = {key: info[key] for key in ["signature", "url"]}
    if set(platforms) != PLATFORMS:
        raise ValueError("All four platform builds must succeed before generating latest.json")
    output.mkdir(parents=True, exist_ok=True)
    for path in source.iterdir():
        if path.is_file() and not path.name.startswith("updater-"):
            shutil.copy2(path, output / path.name)
    manifest = {
        "version": version,
        "notes": f"Maqai {version}. See the GitHub release for full release notes.",
        "pub_date": datetime.now(timezone.utc).isoformat().replace("+00:00", "Z"),
        "platforms": platforms,
    }
    (output / "latest.json").write_text(json.dumps(manifest, indent=2) + "\n")


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--source", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--version", required=True)
    args = parser.parse_args()
    finalize(args.source, args.output, args.version)

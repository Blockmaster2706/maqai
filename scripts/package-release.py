import argparse
import json
from pathlib import Path
import shutil
from urllib.parse import quote

TARGETS = {
    "x86_64-pc-windows-msvc": ("windows-x86_64", ".exe"),
    "x86_64-unknown-linux-gnu": ("linux-x86_64", ".AppImage"),
    "x86_64-apple-darwin": ("darwin-x86_64", ".app.tar.gz"),
    "aarch64-apple-darwin": ("darwin-aarch64", ".app.tar.gz"),
}
SUFFIXES = [".app.tar.gz.sig", ".app.tar.gz", ".AppImage.sig", ".AppImage", ".exe.sig", ".exe", ".dmg", ".deb", ".rpm"]


def package(bundle_root, output, target, version, repository):
    platform, updater_suffix = TARGETS[target]
    output.mkdir(parents=True, exist_ok=True)
    files = {}
    for folder in ["nsis", "appimage", "macos", "dmg", "deb", "rpm"]:
        directory = bundle_root / folder
        if not directory.exists():
            continue
        for path in directory.iterdir():
            if not path.is_file():
                continue
            suffix = next((s for s in SUFFIXES if path.name.endswith(s)), None)
            if not suffix:
                continue
            if suffix in files:
                raise ValueError(f"More than one {suffix} bundle found for {target}")
            destination = output / f"Maqai_{version}_{target}{suffix}"
            shutil.copy2(path, destination)
            files[suffix] = destination
    if updater_suffix not in files or updater_suffix + ".sig" not in files:
        raise ValueError(f"Missing signed updater bundle for {target}")
    signature = files[updater_suffix + ".sig"].read_text().strip()
    if not signature:
        raise ValueError("Updater signature is empty")
    url = f"https://github.com/{repository}/releases/download/v{version}/{quote(files[updater_suffix].name)}"
    manifest = {"version": version, "platform": platform, "signature": signature, "url": url}
    (output / f"updater-{target}.json").write_text(json.dumps(manifest, indent=2) + "\n")


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--target", choices=TARGETS, required=True)
    parser.add_argument("--version", required=True)
    parser.add_argument("--repository", required=True)
    parser.add_argument("--bundle-root", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    package(args.bundle_root, args.output, args.target, args.version, args.repository)

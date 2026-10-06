#!/usr/bin/env python3
"""Download the latest Android platform tools for this OS (stdlib only)."""

import platform
import shutil
import tempfile
from pathlib import Path
from urllib.request import urlopen
from zipfile import ZipFile


def main():
    # Folder names match the platform-specific Tauri resource configurations.
    platforms = {
        "Windows": ("windows", "windows"),
        "Darwin": ("darwin", "macos"),
        "Linux": ("linux", "linux"),
    }
    system = platform.system()
    if system not in platforms:
        raise SystemExit(f"Unsupported platform: {system}")

    archive_platform, folder = platforms[system]
    destination = Path(__file__).resolve().parent / folder
    url = (
        "https://dl.google.com/android/repository/"
        f"platform-tools-latest-{archive_platform}.zip"
    )

    print(f"Downloading {url}")
    # The downloaded ZIP and staging files are removed even if extraction fails.
    with tempfile.TemporaryDirectory(prefix="platform-tools-") as temporary, tempfile.NamedTemporaryFile(
        prefix=f"platform-tools-latest-{archive_platform}-",
        suffix=".zip",
        dir=destination.parent,
    ) as archive:
        staging = Path(temporary)
        with urlopen(url, timeout=60) as response:
            shutil.copyfileobj(response, archive)
        archive.flush()
        archive.seek(0)

        with ZipFile(archive) as zipped:
            zipped.extractall(staging)
            # ZipFile does not restore Unix executable permissions itself.
            if system != "Windows":
                for member in zipped.infolist():
                    permissions = (member.external_attr >> 16) & 0o777
                    if permissions:
                        (staging / member.filename).chmod(permissions)

        shutil.copytree(staging / "platform-tools", destination, dirs_exist_ok=True)

    print(f"Installed platform tools in {destination}")


if __name__ == "__main__":
    main()

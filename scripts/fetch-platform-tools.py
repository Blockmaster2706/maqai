import argparse
from pathlib import Path
import shutil
import tempfile
import urllib.request
import zipfile


def fetch(platform, destination):
    archive_platform = {"windows": "windows", "linux": "linux", "macos": "darwin"}[platform]
    url = f"https://dl.google.com/android/repository/platform-tools-latest-{archive_platform}.zip"
    with tempfile.TemporaryDirectory() as temporary:
        archive = Path(temporary) / "platform-tools.zip"
        with urllib.request.urlopen(url, timeout=120) as response, archive.open("wb") as file:
            shutil.copyfileobj(response, file)
        destination.mkdir(parents=True, exist_ok=True)
        with zipfile.ZipFile(archive) as bundle:
            for info in bundle.infolist():
                relative = Path(info.filename)
                if not relative.parts or relative.parts[0] != "platform-tools":
                    continue
                relative = Path(*relative.parts[1:])
                if ".." in relative.parts or relative.is_absolute():
                    raise ValueError("Invalid archive path")
                target = destination / relative
                if info.is_dir():
                    target.mkdir(parents=True, exist_ok=True)
                    continue
                target.parent.mkdir(parents=True, exist_ok=True)
                with bundle.open(info) as source, target.open("wb") as file:
                    shutil.copyfileobj(source, file)
                if platform != "windows":
                    target.chmod((info.external_attr >> 16) & 0o777 or 0o644)
        executable = destination / ("adb.exe" if platform == "windows" else "adb")
        if not executable.is_file():
            raise ValueError("Google's archive does not contain ADB")
        if platform != "windows":
            executable.chmod(executable.stat().st_mode | 0o111)


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("platform", choices=["windows", "linux", "macos"])
    args = parser.parse_args()
    root = Path(__file__).resolve().parents[1]
    fetch(args.platform, root / "src-tauri/adb" / args.platform)

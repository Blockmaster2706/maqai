import importlib.util
import json
from pathlib import Path
import tempfile
import tomllib
import unittest


def load(name):
    spec = importlib.util.spec_from_file_location(name, Path(__file__).with_name(name + ".py"))
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


bump = load("bump-version")
package = load("package-release")
finalize = load("finalize-release")
configure = load("configure-updater")


class ReleaseTests(unittest.TestCase):
    def test_signing_config_requires_a_public_key(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "src-tauri").mkdir()
            with self.assertRaises(ValueError):
                configure.configure(root, "")
            path = configure.configure(root, "public-key\n")
            self.assertEqual(json.loads(path.read_text())["plugins"]["updater"]["pubkey"], "public-key")

    def test_semver_resets_and_manual_validation(self):
        self.assertEqual(bump.next_version("1.2.9", "patch"), "1.2.10")
        self.assertEqual(bump.next_version("1.2.9", "minor"), "1.3.0")
        self.assertEqual(bump.next_version("1.2.9", "major"), "2.0.0")
        self.assertEqual(bump.next_version("1.2.9", "current"), "1.2.9")
        for version in ["1.2.8", "1.02.10", "2.0.0-beta", "v2.0.0"]:
            with self.assertRaises(ValueError):
                bump.next_version("1.2.9", "patch", version)

    def test_all_versions_are_synchronized(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "src-tauri").mkdir()
            for filename, name in [("Cargo.toml", "maqai-ui"), ("src-tauri/Cargo.toml", "maqai")]:
                (root / filename).write_text(f'[package]\nname = "{name}"\nversion = "0.1.0"\n')
            (root / "src-tauri/tauri.conf.json").write_text('{"version": "0.1.0"}')
            (root / "Cargo.lock").write_text('version = 4\n\n[[package]]\nname = "maqai"\nversion = "0.1.0"\n\n[[package]]\nname = "maqai-ui"\nversion = "0.1.0"\n')
            self.assertEqual(bump.bump_version(root, "minor"), "0.2.0")
            self.assertEqual(json.loads((root / "src-tauri/tauri.conf.json").read_text())["version"], "0.2.0")
            self.assertEqual(tomllib.loads((root / "Cargo.lock").read_text())["package"][0]["version"], "0.2.0")
            self.assertEqual(tomllib.loads((root / "Cargo.toml").read_text())["package"]["version"], "0.2.0")
            self.assertEqual(tomllib.loads((root / "src-tauri/Cargo.toml").read_text())["package"]["version"], "0.2.0")

    def test_complete_manifest_has_each_signed_platform(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            source = root / "assets"
            for target, (platform, suffix) in package.TARGETS.items():
                bundle = root / target
                folder = {".exe": "nsis", ".AppImage": "appimage", ".app.tar.gz": "macos"}[suffix]
                (bundle / folder).mkdir(parents=True)
                (bundle / folder / ("Maqai" + suffix)).write_bytes(b"installer")
                (bundle / folder / ("Maqai" + suffix + ".sig")).write_text("signature-" + platform)
                package.package(bundle, source, target, "0.1.1", "owner/maqai")
            finalize.finalize(source, root / "ready", "0.1.1")
            manifest = json.loads((root / "ready/latest.json").read_text())
            self.assertEqual(set(manifest["platforms"]), finalize.PLATFORMS)
            for platform, info in manifest["platforms"].items():
                self.assertEqual(info["signature"], "signature-" + platform)
                self.assertIn("/releases/download/v0.1.1/", info["url"])

    def test_incomplete_release_cannot_generate_updater_manifest(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            source = root / "assets"
            source.mkdir()
            with self.assertRaises(ValueError):
                finalize.finalize(source, root / "ready", "0.1.1")
            self.assertFalse((root / "ready/latest.json").exists())

    def test_unsigned_bundle_is_rejected(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "bundle/nsis").mkdir(parents=True)
            (root / "bundle/nsis/Maqai.exe").write_bytes(b"installer")
            with self.assertRaises(ValueError):
                package.package(root / "bundle", root / "assets", "x86_64-pc-windows-msvc", "0.1.1", "owner/maqai")


if __name__ == "__main__":
    unittest.main()

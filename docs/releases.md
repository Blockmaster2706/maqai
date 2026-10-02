# Releases and updates

## One-time GitHub setup

Generate an updater signing key on a trusted machine:

```sh
cargo tauri signer generate -w /path/outside-the-repository/maqai-updater.key
```

Use a permanent location outside this repository and back up the key and its password. On Windows, choose a path such as `C:\Users\yourname\.tauri\maqai-updater.key`. Never commit the private key. Existing installations need the same signing key for future updates.

In the repository's **Settings → Secrets and variables → Actions**, configure:

| Type | Name | Value |
| --- | --- | --- |
| Secret | `TAURI_SIGNING_PRIVATE_KEY` | Contents of the generated private `.key` file |
| Secret | `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` | Its password; leave unset if the key is unencrypted |
| Variable | `TAURI_UPDATER_PUBLIC_KEY` | Contents of the generated `.key.pub` file |

The release workflow embeds the public key through a compile-time environment variable and a generated, gitignored Tauri config override. Tauri also uses that config to check that the signing key matches. The private key is used only by the Tauri bundler to sign update artifacts. Local development builds work without these values and do not check for updates automatically.

The workflow uses `GITHUB_TOKEN` with `contents: write` only in the preparation and upload jobs. It commits version bumps directly to the default branch. If branch protection forbids this, bump locally, merge the version change through your normal PR process, and select **current** in the release workflow.

## Make a release

1. Commit and merge the changes to release.
2. Open **Actions → Release → Run workflow**.
3. Choose **patch**, **minor**, **major**, or **current**. Optionally enter an exact stable version such as `1.2.0` to override the bump. Versions cannot go backward.
4. The workflow synchronizes `Cargo.toml`, `src-tauri/Cargo.toml`, `Cargo.lock`, and `tauri.conf.json`, commits the change, creates `vVERSION`, and creates a draft GitHub release.
5. Windows x64, Linux x64, macOS Intel, and macOS Apple Silicon builds run. Platform Tools are fetched from Google because the local ADB directories are gitignored.
6. When every build succeeds, the installers, signatures, and a complete `latest.json` are uploaded to the draft.
7. Review and test the installers, edit the release notes, then publish the draft as the latest stable release. This makes it available to Maqai's updater.

If a build fails, the draft remains unpublished. Fix the workflow or dependency issue and rerun failed jobs. To rebuild the same unchanged release commit, select **current**. An existing tag cannot be reused for a different commit or a published release.

The workflow always releases the default branch, regardless of the branch selected in the workflow dispatch UI. It does not publish automatically.

## Bump locally

```sh
python scripts/bump-version.py patch
python scripts/bump-version.py minor
python scripts/bump-version.py major
python scripts/bump-version.py --version 1.2.0
python scripts/bump-version.py patch --dry-run
```

Use Python 3.11 or newer. Commit all four updated files together.

## Updater behavior

Release builds check `https://github.com/Blockmaster2706/maqai/releases/latest/download/latest.json` on startup. The sidebar also has **Check for updates**. A new version shows an **Install update and restart** button; downloading and installation begin only when clicked. The updater verifies the signature before installation.

Update Maqai before starting or after finishing a rooting session. Installation is blocked while an ADB operation is running. Failed downloads can be retried.

Windows uses an NSIS installer in passive mode. Linux self-updates use AppImage; `.deb` and `.rpm` users should update through their package manager or a new package. macOS uses signed updater archives for both architectures. The updater signing key is separate from Apple/Windows OS code signing.

For a local release build with updater support, set `TAURI_UPDATER_PUBLIC_KEY`, `TAURI_SIGNING_PRIVATE_KEY`, and optionally `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` in the process environment, then run:

```sh
python scripts/configure-updater.py
cargo tauri build --config src-tauri/tauri.release.conf.json
```

`src-tauri/tauri.conf.json` intentionally keeps its default public-key field empty; signed builds use the generated override and embedded build variable.

## Optional OS code signing

For notarized macOS distribution, set repository secrets `APPLE_CERTIFICATE` (base64-encoded Developer ID `.p12`), `APPLE_CERTIFICATE_PASSWORD`, `APPLE_SIGNING_IDENTITY`, `APPLE_ID`, `APPLE_PASSWORD` (app-specific password), and `APPLE_TEAM_ID`. The workflow passes configured values to Tauri and omits empty optional values. Without Apple credentials, builds are not notarized and macOS may require the user to explicitly allow the app.

Windows updater signatures do not provide Authenticode signing or SmartScreen reputation. Windows code signing can be added when a certificate or signing service is available.

References: [Tauri updater](https://v2.tauri.app/plugin/updater/), [macOS signing](https://v2.tauri.app/distribute/sign/macos/), [Windows signing](https://v2.tauri.app/distribute/sign/windows/).

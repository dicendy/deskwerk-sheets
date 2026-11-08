# Releasing (CI)

`.github/workflows/release.yml` builds the packages with the scripts in this directory. It never
runs on branch pushes or pull requests (Actions minutes are limited; macOS minutes cost 10x,
Windows 2x).

## Cutting a release

1. Bump `[workspace.package] version` on `main` (`cargo xtask version set 0.16.0`) and merge.
2. Tag that commit `v<version>` and push the tag: `git tag v0.16.0 && git push origin v0.16.0`.
   A tag that differs from the Cargo.toml version fails at once.
3. The run builds every target and drafts a GitHub Release `Deskwerk Sheets v<version>` with all
   files and `SHA256SUMS.txt`. Review the draft and publish it. Re-running the workflow for the
   same tag replaces the draft's files.

## Test builds

Actions → Release → Run workflow (or `gh workflow run release.yml --ref <branch> -f targets=linux,web`):

- `targets`: any of `linux,web,windows,macos` (default: all).
- `macos_arch`: `universal` (release) or `aarch64` (half the macOS minutes).
- `draft_release`: also draft a release from what was built (delete it afterwards).

The packages are kept as workflow artifacts for 7 days.

## What is built

| Job | Runner | Files (`<v>` = version) |
|---|---|---|
| linux | ubuntu-22.04 (glibc 2.35) | `deskwerk-sheets-<v>-linux-x86_64.{AppImage,AppImage.zsync,deb,rpm,tar.gz}` |
| web | ubuntu-24.04 | `deskwerk-sheets-web-<v>.zip` |
| windows | windows-2022 | `deskwerk-sheets-<v>-windows-x64.msi`, `deskwerk-sheets-<v>-windows-x64-portable.zip` |
| macos | macos-14 | `deskwerk-sheets-<v>-macos-universal.dmg`, `deskwerk-sheets-cli-<v>-macos-universal.zip` |

Not built in CI: Linux aarch64/riscv64, Flatpak, FreeBSD, Windows x86/arm64 (the scripts support
them: `package.sh`, `flatpak-bundle.sh`, `freebsd/package.sh`, `package.ps1 -Arch x86|arm64`).

## Signing secrets (optional)

Without them every build still succeeds, with a warning: Windows files are unsigned, the macOS app
and DMG are ad-hoc signed and not notarized (Gatekeeper blocks them on other Macs until the user
allows them in System Settings › Privacy & Security). Add them under Settings › Secrets and
variables › Actions.

| Secret | What |
|---|---|
| `WINDOWS_CERTIFICATE` | Authenticode code-signing certificate, base64 of the `.pfx` |
| `WINDOWS_CERTIFICATE_PASSWORD` | its password |
| `AZURE_TENANT_ID`, `AZURE_CLIENT_ID`, `AZURE_CLIENT_SECRET` | alternative to the `.pfx`: Azure Trusted Signing service principal |
| `AZURE_SIGNING_ENDPOINT`, `AZURE_SIGNING_ACCOUNT`, `AZURE_CERT_PROFILE` | Trusted Signing endpoint (e.g. `https://weu.codesigning.azure.net`), account and certificate profile |
| `APPLE_CERTIFICATE` | "Developer ID Application" certificate with its private key, base64 of the `.p12` |
| `APPLE_CERTIFICATE_PASSWORD` | the `.p12` password |
| `KEYCHAIN_PASSWORD` | any random string for the temporary CI keychain (optional) |
| `APPLE_ID`, `APPLE_PASSWORD`, `APPLE_TEAM_ID` | notarization: Apple ID email, an app-specific password for it, the 10-character team id |

Windows uses the `.pfx` when `WINDOWS_CERTIFICATE` is set, else Azure when all six `AZURE_*` are
set (`windows/sign.ps1`). macOS notarizes only with a certificate and all three `APPLE_ID`,
`APPLE_PASSWORD`, `APPLE_TEAM_ID` (`macos/import-cert.sh`, `macos/package.sh`). The draft release
uses the workflow's own `GITHUB_TOKEN`; no secret is needed for it.

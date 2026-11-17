# Windows installation policy checks

New installations default to the current account under `Programs\Deskwerk\<App>`.
The first setup page offers installation for everyone. Scope is selected before duplicate
checks and directory costing. An update uses the existing context's remembered exact folder;
`INSTALLFOLDER` supplied explicitly is never replaced. It is a secure MSI property so the
chosen folder reaches the elevated installer. Silent machine updates pass `ALLUSERS=1`.
A cross-context copy is refused; changing context means uninstalling the old copy first.

App registration follows the selected context. Existing optional shortcuts and file-handler
choices stay app-owned. No new cleanup operation or model/settings migration is introduced.
Packaged resources remain package-owned; separately downloaded shared models are not caches
and are outside this installer's ownership.

## Setup wizard

Deskwerk Docs and Deskwerk Sheets share one wizard (`installer-ui.wxi`, the same file in both
repositories; WiX's standard dialogs from WixToolset.UI.wixext). A new installation or an
upgrade shows: who should use the app (the scope page above), Welcome, Ready to install with
**Create a desktop shortcut**, progress, and Finish with **Launch Deskwerk <App>** (ticked;
never in `/qn` or `/qb` installs). There is no licence page and no folder page; the licence
texts are installed as plain files next to the app. Running the MSI again shows Repair /
Remove (Change is off: there is nothing to change), then the same Ready page and progress.

The desktop shortcut goes on the desktop of the selected context (the common desktop for
everyone, the user's own for the current account) and uninstall removes it. The checkbox is
ticked on a new installation; an upgrade remembers the previous choice (stored as
`Software\Deskwerk\<App>\Installer\DesktopShortcut` under HKLM or HKCU). `DESKTOPSHORTCUT=0`
or `DESKTOPSHORTCUT=1` on the `msiexec` command line always wins. Deskwerk Docs 0.15.6 and
earlier kept only a marker under `Software\DeskwerkDocs\Installer`; an upgrade from those
reads it, so a removed shortcut stays removed.

The Windows release job renders the wizard's pages from the built MSI
(`preview-installer-dialogs.ps1`, artifact `installer-dialogs`) and installs fixture MSIs built
from these sources on the disposable runner (`test-desktop-shortcut.ps1`: default, explicit
choices, repair, upgrades including from the last release, uninstall by another account).

## Repeatable metadata tests

On Windows with WiX 5.0.2 and its matching UI/Util extensions:

```powershell
./packaging/windows/test-install-policy.ps1 -OutputDirectory ./target/install-policy-tests
```

The script builds x64, x86 and arm64 packages with deliberately non-executable payloads. It
uses the actual package source and compiled database. A safe Windows Installer session checks
24 scope/registration/maintenance cases, remembered and explicit custom folders, sequencing,
upgrade identity and the original registration inventory recorded in `install-policy.json`.
The release packaging script runs the compiled checker before signing. Packaging CI runs the
small fixture builds without compiling the Rust application.

Never install or distribute the `NOT-INSTALLABLE.msi` fixtures as an app. Passing these checks
does not prove real installation, elevation, legacy-version upgrades, registry writes, optional
shortcut/menu behaviour, cancellation, rollback or uninstall. Those still require the real
payload in an authorised clean Windows test environment. Cross-architecture metadata is not
execution evidence on those architectures. Existing open installer/association PRs must be
reconciled before publication. Nothing here changes macOS/Linux native packaging conventions.

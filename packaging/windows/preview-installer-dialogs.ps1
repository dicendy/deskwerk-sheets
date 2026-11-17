<#
.SYNOPSIS
  Capture native MSI dialog previews without installing the package.
.DESCRIPTION
  Opens the MSI read-only and uses Windows Installer's inactive UI preview API.
  These show authored dialog layout, not an end-to-end installation or runtime messages.
  Each wizard page is saved as <NN>-<Dialog>.png in the order a new installation (01-05)
  and a maintenance run (06-07) show them. The same script is used by Deskwerk Docs and
  Deskwerk Sheets; the dialog window is found by its title, "<ProductName> Setup".
  A preview cannot set properties: checkboxes show their Property-table defaults, and
  conditional texts show the new-installation case.
  https://learn.microsoft.com/windows/win32/msi/previewing-the-user-interface
#>
param(
  [Parameter(Mandatory = $true)] [string] $Msi,
  [Parameter(Mandatory = $true)] [string] $OutputDir
)
$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
if ([Environment]::OSVersion.Platform -ne [PlatformID]::Win32NT) {
  throw 'Native MSI dialog previews require Windows.'
}
$Msi = (Resolve-Path -LiteralPath $Msi).Path
New-Item -ItemType Directory -Force -Path $OutputDir | Out-Null
$OutputDir = (Resolve-Path -LiteralPath $OutputDir).Path
# Every dialog of both installers is titled "[ProductName] Setup".
$installer = $null; $readOnly = $null; $view = $null; $record = $null
try {
  $installer = New-Object -ComObject WindowsInstaller.Installer
  $readOnly = $installer.OpenDatabase($Msi, 0)
  $view = $readOnly.OpenView("SELECT ``Value`` FROM ``Property`` WHERE ``Property`` = 'ProductName'")
  [void]$view.Execute()
  $record = $view.Fetch()
  if ($null -eq $record) { throw 'The MSI has no ProductName.' }
  $ProductName = [string]$record.GetType().InvokeMember('StringData', 'GetProperty', $null, $record, @(1))
  [void]$view.Close()
} finally {
  foreach ($com in @($record, $view, $readOnly, $installer)) {
    if ($null -ne $com) { [void][Runtime.InteropServices.Marshal]::FinalReleaseComObject($com) }
  }
}
$WindowTitle = "$ProductName Setup"
Add-Type -AssemblyName System.Windows.Forms, System.Drawing
Add-Type -TypeDefinition @'
using System;
using System.Runtime.InteropServices;
using System.Text;
public static class DeskwerkMsiPreview {
    [DllImport("msi.dll", CharSet = CharSet.Unicode, ExactSpelling = true)]
    public static extern uint MsiOpenDatabaseW(string path, IntPtr persist, out uint database);
    [DllImport("msi.dll")]
    public static extern uint MsiEnableUIPreview(uint database, out uint preview);
    [DllImport("msi.dll", CharSet = CharSet.Unicode, ExactSpelling = true)]
    public static extern uint MsiPreviewDialogW(uint preview, string dialog);
    [DllImport("msi.dll")]
    public static extern uint MsiCloseHandle(uint handle);
    [StructLayout(LayoutKind.Sequential)]
    public struct Rect { public int Left, Top, Right, Bottom; }
    private delegate bool WindowCallback(IntPtr window, IntPtr state);
    [DllImport("user32.dll")]
    private static extern bool EnumWindows(WindowCallback callback, IntPtr state);
    [DllImport("user32.dll")]
    private static extern uint GetWindowThreadProcessId(IntPtr window, out uint process);
    [DllImport("user32.dll", CharSet = CharSet.Unicode)]
    private static extern int GetWindowText(IntPtr window, StringBuilder text, int count);
    [DllImport("user32.dll")]
    public static extern bool GetWindowRect(IntPtr window, out Rect rect);
    [DllImport("user32.dll")]
    public static extern bool PrintWindow(IntPtr window, IntPtr dc, uint flags);
    public static IntPtr FindDialog(uint process, string wanted) {
        IntPtr found = IntPtr.Zero;
        EnumWindows(delegate(IntPtr window, IntPtr state) {
            uint owner;
            GetWindowThreadProcessId(window, out owner);
            if (owner != process) return true;
            var title = new StringBuilder(128);
            GetWindowText(window, title, title.Capacity);
            if (title.ToString() != wanted) return true;
            found = window;
            return false;
        }, IntPtr.Zero);
        return found;
    }
}
'@
function Assert-MsiResult([uint32] $Result, [string] $Operation) {
  if ($Result -ne 0) { throw "$Operation failed with MSI error $Result" }
}
[uint32] $database = 0
[uint32] $preview = 0
try {
  Assert-MsiResult ([DeskwerkMsiPreview]::MsiOpenDatabaseW($Msi, [IntPtr]::Zero, [ref] $database)) 'Open read-only MSI'
  # New installation: scope (install-policy.wxi), welcome, ready (desktop shortcut), progress, finish.
  # Maintenance (repair or remove): maintenance welcome, maintenance type, then the same ready page.
  $dialogs = 'CraftScopeDlg', 'WelcomeDlg', 'DeskwerkReadyDlg', 'ProgressDlg', 'ExitDialog', 'MaintenanceWelcomeDlg', 'MaintenanceTypeDlg'
  $step = 0
  foreach ($dialog in $dialogs) {
    $step++
    # Give each dialog its own preview session; closing its handle releases the UI.
    Assert-MsiResult ([DeskwerkMsiPreview]::MsiEnableUIPreview($database, [ref] $preview)) "Enable preview for $dialog"
    Assert-MsiResult ([DeskwerkMsiPreview]::MsiPreviewDialogW($preview, $dialog)) "Preview $dialog"
    $window = [IntPtr]::Zero
    $wait = [Diagnostics.Stopwatch]::StartNew()
    while ($window -eq [IntPtr]::Zero -and $wait.Elapsed.TotalSeconds -lt 5) {
      [Windows.Forms.Application]::DoEvents()
      $window = [DeskwerkMsiPreview]::FindDialog([uint32] $PID, $WindowTitle)
      Start-Sleep -Milliseconds 50
    }
    if ($window -eq [IntPtr]::Zero) { throw "No native window found for $dialog" }
    [Windows.Forms.Application]::DoEvents()
    $rect = New-Object DeskwerkMsiPreview+Rect
    if (-not [DeskwerkMsiPreview]::GetWindowRect($window, [ref] $rect)) { throw "Cannot measure $dialog" }
    $width = $rect.Right - $rect.Left
    $height = $rect.Bottom - $rect.Top
    if ($width -lt 1 -or $height -lt 1 -or $width -gt 4096 -or $height -gt 4096) { throw "Invalid preview dimensions for $dialog" }
    $bitmap = New-Object Drawing.Bitmap($width, $height)
    $graphics = [Drawing.Graphics]::FromImage($bitmap)
    try {
      $dc = $graphics.GetHdc()
      try {
        if (-not [DeskwerkMsiPreview]::PrintWindow($window, $dc, 0)) { throw "Capture failed for $dialog" }
      } finally { $graphics.ReleaseHdc($dc) }
      $colors = New-Object 'Collections.Generic.HashSet[int]'
      for ($y = 0; $y -lt $height; $y += [Math]::Max(1, [int] ($height / 32))) {
        for ($x = 0; $x -lt $width; $x += [Math]::Max(1, [int] ($width / 32))) {
          $null = $colors.Add($bitmap.GetPixel($x, $y).ToArgb())
        }
      }
      if ($colors.Count -lt 2) { throw "Empty capture for $dialog" }
      $bitmap.Save((Join-Path $OutputDir ('{0:D2}-{1}.png' -f $step, $dialog)), [Drawing.Imaging.ImageFormat]::Png)
    } finally {
      $graphics.Dispose()
      $bitmap.Dispose()
    }
    Assert-MsiResult ([DeskwerkMsiPreview]::MsiCloseHandle($preview)) "Close preview for $dialog"
    $preview = 0
    Write-Host "Saved native MSI dialog preview: $dialog"
  }
  "$ProductName`: native MSI dialog previews only. No installation was run. Checkboxes show their defaults; progress has no live data." |
    Set-Content -LiteralPath (Join-Path $OutputDir 'README.txt') -Encoding utf8
} finally {
  if ($preview -ne 0) {
    $null = [DeskwerkMsiPreview]::MsiCloseHandle($preview)
  }
  if ($database -ne 0) { $null = [DeskwerkMsiPreview]::MsiCloseHandle($database) }
}

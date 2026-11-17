<#
.SYNOPSIS
  Exercise the actual MSI's desktop-shortcut choices on a disposable GitHub Windows runner.
.DESCRIPTION
  Compiles original no-op GUI/CLI fixtures instead of Rust, uses the app's existing icons,
  and builds adjacent MSI versions from this checkout's packaging sources. Installs the real
  package (including its registrations) silently for all users, checks the default (on),
  explicit choices, repair, remembered choices across major upgrades, the upgrade from the
  last release (built from -LegacySource, a checkout of that release's sources) and
  cross-account removal, then uninstalls in finally. Deskwerk Docs and Deskwerk Sheets use
  the same script. Run by the Windows job of release.yml, never on a developer's Windows
  installation.
#>
param(
  [Parameter(Mandatory = $true)] [string] $LegacySource,
  [string] $WorkDir
)
$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest

if ($env:GITHUB_ACTIONS -ne 'true' -or $env:RUNNER_OS -ne 'Windows' -or $env:RUNNER_ENVIRONMENT -ne 'github-hosted') {
  throw 'This install test is restricted to disposable GitHub-hosted Windows runners.'
}
$Root = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path
$policy = Get-Content -Raw -LiteralPath (Join-Path $PSScriptRoot 'install-policy.json') | ConvertFrom-Json
$App = "Deskwerk $($policy.name)"          # Deskwerk Docs / Deskwerk Sheets
$Id = $policy.id                            # deskwerk-docs / deskwerk-sheets
$Wxs = $policy.sources[0]
# The last release before the shared installer UI: its default and where it kept its marker.
$Legacy = switch ($Id) {
  'deskwerk-docs' { [pscustomobject]@{ Version = '0.15.6'; DefaultOn = $true; MarkerKey = 'HKLM:\Software\DeskwerkDocs\Installer'; MarkerName = 'DesktopShortcut' } }
  'deskwerk-sheets' { [pscustomobject]@{ Version = '0.16.4'; DefaultOn = $false; MarkerKey = 'HKLM:\Software\Deskwerk\Sheets\Installer'; MarkerName = 'DesktopShortcutComponent' } }
  default { throw "Unknown app $Id" }
}
if (-not $WorkDir) { $WorkDir = Join-Path $env:RUNNER_TEMP "$Id-shortcut-smoke" }
$InstallDir = Join-Path $env:ProgramFiles "Deskwerk\$($policy.name)"
$DesktopLink = Join-Path ([Environment]::GetFolderPath('CommonDesktopDirectory')) "$App.lnk"
$StartLink = Join-Path ([Environment]::GetFolderPath('CommonPrograms')) "$App.lnk"
$ChoiceKey = "HKLM:\Software\Deskwerk\$($policy.name)\Installer"
$UninstallRoot = 'HKLM:\Software\Microsoft\Windows\CurrentVersion\Uninstall'
$Icon = Join-Path $Root $policy.icon
foreach ($hive in 'HKLM:', 'HKCU:') {
  foreach ($key in "Software\Deskwerk\$($policy.name)", $policy.installDirKey) {
    if (Test-Path "$hive\$key") { throw "Refusing to run against an existing $App installation or its settings ($hive\$key)." }
  }
}
if ((Test-Path $InstallDir) -or (Test-Path $DesktopLink) -or (Test-Path $StartLink)) {
  throw "Refusing to run against an existing $App installation."
}
New-Item -ItemType Directory -Force -Path $WorkDir | Out-Null
$Bin = Join-Path $WorkDir 'bin'
$Notices = Join-Path $WorkDir 'notices'
New-Item -ItemType Directory -Force -Path $Bin, $Notices | Out-Null
Set-Content -LiteralPath (Join-Path $Notices 'NOTICE-fixture.txt') -Value 'Installer test fixture.'
$Rtf = Join-Path $WorkDir 'licence.rtf'
Set-Content -LiteralPath $Rtf -Encoding Ascii -Value '{\rtf1 Installer test fixture.}'

function Assert-That([bool] $Condition, [string] $Message) {
  if (-not $Condition) { throw $Message }
}

function Get-OptionalRegistryValue([string] $Path, [string] $Name) {
  if (-not (Test-Path -LiteralPath $Path)) { return $null }
  $key = Get-Item -LiteralPath $Path
  try { return $key.GetValue($Name, $null) }
  finally { $key.Close() }
}

function Get-ProductCode([string] $Msi) {
  $installer = New-Object -ComObject WindowsInstaller.Installer
  $database = $null
  $view = $null
  $record = $null
  try {
    $database = $installer.OpenDatabase($Msi, 0)
    $view = $database.OpenView('SELECT `Value` FROM `Property` WHERE `Property` = ''ProductCode''')
    $view.Execute()
    $record = $view.Fetch()
    if (-not $record) { throw "No ProductCode in $Msi" }
    return $record.StringData(1)
  } finally {
    if ($view) { $view.Close() }
    foreach ($item in $record, $view, $database, $installer) {
      if ($item) { [Runtime.InteropServices.Marshal]::FinalReleaseComObject($item) | Out-Null }
    }
  }
}

function Invoke-Msi([string] $Name, [string[]] $Arguments) {
  $log = Join-Path $WorkDir "$Name.log"
  # /norestart is unconditional. Each operation and the workflow have time limits.
  # The all-users route also covers cross-account removal.
  $process = Start-Process msiexec.exe -ArgumentList ($Arguments + @('ALLUSERS=1', '/qn', '/norestart', '/l*v', "`"$log`"")) -PassThru
  try {
    if (-not $process.WaitForExit(180000)) {
      $process.Kill()
      throw "$Name exceeded three minutes; see $log"
    }
    if ($process.ExitCode -ne 0) {
      if (Test-Path $log) { Get-Content $log -Tail 60 | Write-Host }
      throw "$Name exited $($process.ExitCode); see $log"
    }
    Write-Host "ok $Name"
  } finally {
    $process.Dispose()
  }
}

function Build-Msi([string] $Source, [string] $Version, [string] $Name) {
  $msi = Join-Path $WorkDir "$Name.msi"
  # Superset of the variables both apps' current and previous sources expect.
  & wix build $Source -arch x64 -ext WixToolset.UI.wixext -ext WixToolset.Util.wixext `
    -d "Version=$Version" -d "BinDir=$Bin" -d "IconPath=$Icon" -d "AppNoticesDir=$Notices" `
    -d "DocsDir=$Notices" -d 'HasCraftFontLicences=0' -d "LicenseRtf=$Rtf" -o $msi | Write-Host
  if ($LASTEXITCODE -ne 0) { throw "WiX failed to build $Name" }
  return [pscustomobject]@{ Path = $msi; Code = (Get-ProductCode $msi); Version = $Version }
}

function Assert-Package($Package) {
  $key = Join-Path $UninstallRoot $Package.Code
  $registered = Get-ItemProperty -Path $key -ErrorAction Stop
  Assert-That ($registered.DisplayVersion -eq $Package.Version) "Wrong installed product version: $($Package.Version)"
  foreach ($exe in "$Id.exe", "$Id-cli.exe") {
    Assert-That (Test-Path (Join-Path $InstallDir $exe)) "$exe missing after installation"
  }
  Assert-That (Test-Path $StartLink) 'The Start Menu shortcut is missing'
}

function Assert-ShortcutTarget {
  $shell = New-Object -ComObject WScript.Shell
  $shortcut = $null
  try {
    $shortcut = $shell.CreateShortcut($DesktopLink)
    Assert-That ($shortcut.TargetPath -ieq (Join-Path $InstallDir "$Id.exe")) 'Desktop shortcut targets the wrong executable'
    Assert-That ($shortcut.WorkingDirectory.TrimEnd('\') -ieq $InstallDir.TrimEnd('\')) 'Desktop shortcut has the wrong working directory'
    Assert-That ($shortcut.IconLocation -match '^(.*),\s*0$') 'Desktop shortcut must use icon index zero'
    $iconPath = [Environment]::ExpandEnvironmentVariables($Matches[1].Trim('"'))
    Assert-That (Test-Path $iconPath) 'Desktop shortcut icon is missing'
    Assert-That ((Get-FileHash $iconPath).Hash -eq (Get-FileHash $Icon).Hash) "Desktop shortcut does not use the packaged $App icon"
  } finally {
    foreach ($item in $shortcut, $shell) {
      if ($item) { [Runtime.InteropServices.Marshal]::FinalReleaseComObject($item) | Out-Null }
    }
  }
}

function Assert-Desktop([bool] $Enabled) {
  Assert-That ((Test-Path $DesktopLink) -eq $Enabled) "Desktop shortcut presence should be $Enabled"
  $marker = Get-OptionalRegistryValue $ChoiceKey 'DesktopShortcutComponent'
  Assert-That (($null -ne $marker) -eq $Enabled) 'Shortcut component marker must match the machine-wide shortcut'
  Assert-That ($null -eq (Get-OptionalRegistryValue ($ChoiceKey -replace '^HKLM:', 'HKCU:') 'DesktopShortcutComponent')) 'Shortcut component must not leave a per-user marker'
  $choice = Get-ItemPropertyValue -Path $ChoiceKey -Name DesktopShortcut
  Assert-That ($choice -eq $(if ($Enabled) { '1' } else { '0' })) 'Installer did not persist the selected desktop choice'
  if ($Legacy.MarkerKey -ne $ChoiceKey -or $Legacy.MarkerName -ne 'DesktopShortcutComponent') {
    Assert-That ($null -eq (Get-OptionalRegistryValue $Legacy.MarkerKey $Legacy.MarkerName)) "The $($Legacy.Version) marker must not remain"
  }
  if ($Enabled) { Assert-ShortcutTarget }
}

function Assert-LegacyDesktop([bool] $Enabled) {
  Assert-That ((Test-Path $DesktopLink) -eq $Enabled) "$($Legacy.Version) desktop shortcut presence should be $Enabled"
  Assert-That (($null -ne (Get-OptionalRegistryValue $Legacy.MarkerKey $Legacy.MarkerName)) -eq $Enabled) "$($Legacy.Version) marker must match its shortcut"
}

function Assert-Removed($Package) {
  Assert-That (-not (Test-Path (Join-Path $UninstallRoot $Package.Code))) 'MSI still registered after removal'
  Assert-That (-not (Test-Path (Join-Path $InstallDir "$Id.exe"))) 'App executable remains after uninstall'
  Assert-That (-not (Test-Path (Join-Path $InstallDir "$Id-cli.exe"))) 'CLI executable remains after uninstall'
  Assert-That (-not (Test-Path $DesktopLink)) 'Desktop shortcut remains after uninstall'
  Assert-That (-not (Test-Path $StartLink)) 'Start Menu shortcut remains after uninstall'
  Assert-That ($null -eq (Get-OptionalRegistryValue $ChoiceKey 'DesktopShortcut')) 'Saved choice remains after uninstall'
  Assert-That ($null -eq (Get-OptionalRegistryValue $ChoiceKey 'DesktopShortcutComponent')) 'Shortcut component registration remains after uninstall'
  Assert-That ($null -eq (Get-OptionalRegistryValue $Legacy.MarkerKey $Legacy.MarkerName)) "The $($Legacy.Version) marker remains after uninstall"
}

function Uninstall-AsSystem($Package) {
  # Exercise removal under a different account without creating a user or credentials.
  # This function is covered by the disposable hosted-runner guard at the top of the script.
  $taskName = "$Id-shortcut-" + [Guid]::NewGuid().ToString('N')
  $log = Join-Path $WorkDir 'uninstall-as-system.log'
  $action = New-ScheduledTaskAction -Execute (Join-Path $env:WINDIR 'System32\msiexec.exe') `
    -Argument "/x $($Package.Code) /qn /norestart /l*v `"$log`""
  $principal = New-ScheduledTaskPrincipal -UserId 'SYSTEM' -LogonType ServiceAccount -RunLevel Highest
  $settings = New-ScheduledTaskSettingsSet -ExecutionTimeLimit (New-TimeSpan -Minutes 8)
  try {
    Register-ScheduledTask -TaskName $taskName -Action $action -Principal $principal -Settings $settings | Out-Null
    $started = Get-Date
    Start-ScheduledTask -TaskName $taskName
    $deadline = $started.AddMinutes(8)
    do {
      Start-Sleep -Milliseconds 250
      $task = Get-ScheduledTask -TaskName $taskName
      $info = Get-ScheduledTaskInfo -TaskName $taskName
      if ($info.LastRunTime -ge $started.AddSeconds(-1) -and $task.State -ne 'Running' -and $info.LastTaskResult -ne 267009) {
        if ($info.LastTaskResult -ne 0 -and (Test-Path $log)) { Get-Content -LiteralPath $log -Tail 80 | Write-Host }
        Assert-That ($info.LastTaskResult -eq 0) "Cross-account uninstall failed with $($info.LastTaskResult); see $log"
        Write-Host "ok uninstall-as-system ($([int]((Get-Date) - $started).TotalSeconds)s)"
        return
      }
    } while ((Get-Date) -lt $deadline)
    Write-Host "Task state $($task.State), last result $($info.LastTaskResult), last run $($info.LastRunTime)"
    Get-Process msiexec -ErrorAction SilentlyContinue | Format-Table Id, StartTime, CPU -AutoSize | Out-String | Write-Host
    if (Test-Path $log) { Get-Content -LiteralPath $log -Tail 80 | Write-Host }
    throw "Cross-account uninstall exceeded eight minutes; see $log"
  } finally {
    if (Get-ScheduledTask -TaskName $taskName -ErrorAction SilentlyContinue) {
      Stop-ScheduledTask -TaskName $taskName -ErrorAction SilentlyContinue
      Unregister-ScheduledTask -TaskName $taskName -Confirm:$false
    }
  }
}

# Original fixture source, compiled by the .NET Framework compiler already on the runner.
# https://learn.microsoft.com/dotnet/csharp/language-reference/compiler-options/
$source = Join-Path $WorkDir 'Stub.cs'
'public static class Program { public static void Main() { } }' | Set-Content -Path $source -Encoding utf8
$csc = Join-Path $env:WINDIR 'Microsoft.NET\Framework64\v4.0.30319\csc.exe'
foreach ($fixture in @(@("$Id.exe", 'winexe'), @("$Id-cli.exe", 'exe'))) {
  & $csc /nologo /platform:x64 "/target:$($fixture[1])" "/out:$(Join-Path $Bin $fixture[0])" $source
  if ($LASTEXITCODE -ne 0) { throw 'Could not compile installer fixture' }
}
# Deskwerk Docs installs its document icon from the build output; Deskwerk Sheets from assets.
Copy-Item -LiteralPath (Join-Path $Root 'assets\app-icon\document.ico') -Destination (Join-Path $Bin "$Id-document.ico")

$legacyWxs = Join-Path $LegacySource "packaging\windows\$Wxs"
Assert-That (Test-Path $legacyWxs) "No $($Legacy.Version) packaging sources under $LegacySource"
$old = Build-Msi $legacyWxs '89.0.0' "$Id-legacy"
$first = Build-Msi (Join-Path $PSScriptRoot $Wxs) '90.0.0' "$Id-90.0.0"
$second = Build-Msi (Join-Path $PSScriptRoot $Wxs) '90.0.1' "$Id-90.0.1"
$third = Build-Msi (Join-Path $PSScriptRoot $Wxs) '90.0.2' "$Id-90.0.2"
$packages = @($old, $first, $second, $third)
$failure = $null
try {
  # New installation: the shortcut is on by default.
  Invoke-Msi 'default-on' @('/i', "`"$($first.Path)`"")
  Assert-Package $first
  Assert-Desktop $true
  Remove-Item -LiteralPath $DesktopLink
  # Repair uses existing choices; opt-in/out is only a fresh-install or major-upgrade option.
  # https://learn.microsoft.com/windows-server/administration/windows-commands/msiexec
  Invoke-Msi 'repair-on' @('/famus', $first.Code)
  Assert-Desktop $true
  Invoke-Msi 'upgrade-on' @('/i', "`"$($second.Path)`"")
  Assert-Package $second
  Assert-That (-not (Test-Path (Join-Path $UninstallRoot $first.Code))) 'Upgrade left the previous MSI installed'
  Assert-Desktop $true
  Invoke-Msi 'upgrade-explicit-off' @('/i', "`"$($third.Path)`"", 'DESKTOPSHORTCUT=0')
  Assert-Package $third
  Assert-Desktop $false
  Invoke-Msi 'repair-after-opt-out' @('/famus', $third.Code)
  Assert-Desktop $false
  Invoke-Msi 'uninstall-after-opt-out' @('/x', $third.Code)
  Assert-Removed $third

  # An explicit opt-out is remembered by the next upgrade, and an explicit value overrides it.
  Invoke-Msi 'explicit-off' @('/i', "`"$($first.Path)`"", 'DESKTOPSHORTCUT=0')
  Assert-Package $first
  Assert-Desktop $false
  Invoke-Msi 'upgrade-off' @('/i', "`"$($second.Path)`"")
  Assert-Package $second
  Assert-Desktop $false
  Invoke-Msi 'upgrade-explicit-on' @('/i', "`"$($third.Path)`"", 'DESKTOPSHORTCUT=1')
  Assert-Package $third
  Assert-That (-not (Test-Path (Join-Path $UninstallRoot $second.Code))) 'Opt-in upgrade left the previous MSI installed'
  Assert-Desktop $true
  Invoke-Msi 'uninstall-on' @('/x', $third.Code)
  Assert-Removed $third

  # Upgrades from the last release keep its shortcut state, whichever it was.
  $opposite = if ($Legacy.DefaultOn) { 'DESKTOPSHORTCUT=0' } else { 'DESKTOPSHORTCUT=1' }
  foreach ($case in @(@('legacy-default', @(), $Legacy.DefaultOn), @('legacy-opposite', @($opposite), (-not $Legacy.DefaultOn)))) {
    Invoke-Msi "$($case[0])-install" (@('/i', "`"$($old.Path)`"") + $case[1])
    Assert-LegacyDesktop $case[2]
    Invoke-Msi "$($case[0])-upgrade" @('/i', "`"$($first.Path)`"")
    Assert-Package $first
    Assert-That (-not (Test-Path (Join-Path $UninstallRoot $old.Code))) "Upgrade left $($Legacy.Version) installed"
    Assert-Desktop $case[2]
    Invoke-Msi "$($case[0])-uninstall" @('/x', $first.Code)
    Assert-Removed $first
  }

  Invoke-Msi 'cross-account-install' @('/i', "`"$($first.Path)`"")
  Assert-Desktop $true
  Uninstall-AsSystem $first
  Assert-Removed $first
} catch {
  $failure = $_
} finally {
  # Only uninstall the exact fixture products built above, including after a failed assertion.
  foreach ($package in $packages) {
    if (Test-Path (Join-Path $UninstallRoot $package.Code)) {
      try { Invoke-Msi "cleanup-$($package.Version)" @('/x', $package.Code) }
      catch {
        Write-Warning $_
        if (-not $failure) { $failure = $_ }
      }
    }
  }
}
if ($failure) { throw $failure }
Write-Host "$App desktop shortcut smoke passed: default on, explicit on/off, repair, remembered choice across upgrades, upgrade from $($Legacy.Version) and cross-account uninstall."

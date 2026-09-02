$ErrorActionPreference = 'Stop'

$pkgdef = Join-Path $PSScriptRoot 'Retro82.pkgdef'
if (!(Test-Path $pkgdef)) {
  throw "Theme pkgdef not found: $pkgdef"
}

$vswhere = "${env:ProgramFiles(x86)}\Microsoft Visual Studio\Installer\vswhere.exe"
if (!(Test-Path $vswhere)) {
  throw "vswhere.exe not found. Is Visual Studio installed?"
}

$vsPath = & $vswhere -latest -products Microsoft.VisualStudio.Product.Enterprise Microsoft.VisualStudio.Product.Professional Microsoft.VisualStudio.Product.Community -property installationPath
if (!$vsPath) {
  throw "No Visual Studio installation found."
}

$platformDir = Join-Path $vsPath 'Common7\IDE\CommonExtensions\Platform'
$target = Join-Path $platformDir 'Retro82.pkgdef'
Copy-Item -Path $pkgdef -Destination $target -Force

$devenv = Join-Path $vsPath 'Common7\IDE\devenv.exe'
& $devenv /updateConfiguration

Write-Host "Retro 82 theme installed for Visual Studio at: $target"
Write-Host "Restart Visual Studio, then select Tools > Theme > Retro 82."

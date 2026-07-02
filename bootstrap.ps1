param(
  [switch]$Force
)

$ErrorActionPreference = 'Stop'
$Dotfiles = Split-Path -Parent $MyInvocation.MyCommand.Path
$HomeDir = $env:USERPROFILE

function Ensure-Installed {
  param([string[]]$Packages)
  $isAdmin = ([Security.Principal.WindowsPrincipal][Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
  if (-not $isAdmin) {
    Write-Host "SKIP package installs — not running as admin" -ForegroundColor Yellow
    return
  }
  $installed = choco list --limit-output | ForEach-Object { $_.Split('|')[0] }
  foreach ($pkg in $Packages) {
    if ($pkg -notin $installed) {
      Write-Host "Installing $pkg..." -ForegroundColor Cyan
      choco install $pkg -y
    } else {
      Write-Host "$pkg already installed" -ForegroundColor Green
    }
  }
}

function Install-File {
  param([string]$Source, [string]$Dest)
  $src = Join-Path $Dotfiles $Source
  if (Test-Path $Dest) {
    if (-not $Force) {
      Write-Host "SKIP $Dest (exists, use -Force to overwrite)" -ForegroundColor Yellow
      return
    }
    Remove-Item -LiteralPath $Dest -Recurse -Force
  }
  $parent = Split-Path $Dest -Parent
  if (-not (Test-Path $parent)) { New-Item -ItemType Directory -Path $parent -Force | Out-Null }
  Copy-Item -LiteralPath $src -Destination $Dest -Recurse -Force
  Write-Host "Installed $Source -> $Dest" -ForegroundColor Green
}

# ── Chocolatey packages ──────────────────────────────────────
$packagesPath = Join-Path $Dotfiles 'choco-packages.txt'
if (Test-Path $packagesPath) {
  $packages = Get-Content $packagesPath | Where-Object { $_ -match '^[a-zA-Z0-9]' } | ForEach-Object { $_.Split('|')[0] }
  Ensure-Installed $packages
}

# ── PowerShell profile ───────────────────────────────────────
Install-File -Source 'Microsoft.PowerShell_profile.ps1' -Dest $PROFILE

# ── psmux ─────────────────────────────────────────────────────
Install-File -Source '.psmux.conf' -Dest "$HomeDir\.psmux.conf"
Install-File -Source '.psmux' -Dest "$HomeDir\.psmux"

# ── opencode ──────────────────────────────────────────────────
Install-File -Source '.opencode' -Dest "$HomeDir\.opencode"

# ── GlazeWM ───────────────────────────────────────────────────
Install-File -Source '.glzr\glazewm' -Dest "$HomeDir\.glzr\glazewm"

# ── Zebar ─────────────────────────────────────────────────────
Install-File -Source '.glzr\zebar' -Dest "$HomeDir\.glzr\zebar"

# ── VS Code settings ──────────────────────────────────────────
Install-File -Source 'vscode\settings.json' -Dest "$env:APPDATA\Code\User\settings.json"

# ── Neovim ────────────────────────────────────────────────────
Install-File -Source 'nvim' -Dest "$env:LOCALAPPDATA\nvim"

# ── Starship prompt ───────────────────────────────────────────
Install-File -Source '.config\starship.toml' -Dest "$HomeDir\.config\starship.toml"

# ── tmux ──────────────────────────────────────────────────────
Install-File -Source '.config\tmux\tmux.conf' -Dest "$HomeDir\.config\tmux\tmux.conf"
Install-File -Source '.config\tmux\gitmux.conf' -Dest "$HomeDir\.config\tmux\gitmux.conf"

# ── Scoop config ──────────────────────────────────────────────
Install-File -Source '.config\scoop\config.json' -Dest "$HomeDir\.config\scoop\config.json"

# ── NuGet config ─────────────────────────────────────────────
Install-File -Source 'nuget\NuGet.config' -Dest "$env:APPDATA\NuGet\NuGet.config"

# ── Windows Terminal ─────────────────────────────────────────
$wtPkg = Get-ChildItem "$env:LOCALAPPDATA\Packages\Microsoft.WindowsTerminal_*\LocalState" -ErrorAction SilentlyContinue
if ($wtPkg) {
  Install-File -Source 'windows-terminal\settings.json' -Dest "$($wtPkg.FullName)\settings.json"
} else {
  Write-Host "SKIP Windows Terminal — package path not found" -ForegroundColor Yellow
}

# ── Git config + global gitignore ────────────────────────────
$gitScript = Join-Path $Dotfiles 'git\setup.ps1'
if (Test-Path $gitScript) {
  & $gitScript -Force:$Force
}

# ── VS Code extensions ───────────────────────────────────────
$extFile = Join-Path $Dotfiles 'vscode\extensions.txt'
if (Test-Path $extFile) {
  $codeExe = "$env:LOCALAPPDATA\Programs\Microsoft VS Code\bin\code.cmd"
  if (Test-Path $codeExe) {
    $installed = & $codeExe --list-extensions
    Get-Content $extFile | Where-Object { $_ -match '^[a-zA-Z]' } | ForEach-Object {
      if ($_ -notin $installed) {
        Write-Host "Installing VS Code extension: $_" -ForegroundColor Cyan
        & $codeExe --install-extension $_ --force
      }
    }
    Write-Host "VS Code extensions checked" -ForegroundColor Green
  } else {
    Write-Host "SKIP VS Code extensions — code.cmd not found" -ForegroundColor Yellow
  }
}

# ── Auto npm install for Zebar ───────────────────────────────
$zebarMain = "$HomeDir\.glzr\zebar\acabrera\main"
if (Test-Path "$zebarMain\package.json") {
  if (-not (Test-Path "$zebarMain\node_modules")) {
    Write-Host "Installing Zebar dependencies..." -ForegroundColor Cyan
    Push-Location $zebarMain
    npm install
    Pop-Location
    Write-Host "Zebar dependencies installed" -ForegroundColor Green
  } else {
    Write-Host "Zebar dependencies already installed" -ForegroundColor Green
  }
}

# ── Scripts (helpers for GlazeWM, etc.) ──────────────────────
Install-File -Source 'scripts' -Dest "$HomeDir\.glzr\scripts"

Write-Host "`nDone. Reload your shell to pick up profile changes." -ForegroundColor Green

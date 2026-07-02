param(
  [switch]$Force
)

$Dotfiles = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)
$HomeDir = $env:USERPROFILE

# Restore git config
$configSrc = Join-Path $Dotfiles "git\config"
if (Test-Path $configSrc) {
  Get-Content $configSrc | ForEach-Object {
    if ($_ -match '^\t(\w+) = (.+)$') {
      $key = $Matches[1]
      $value = $Matches[2]
      git config --global $key $value
    }
  }
  Write-Host "Git config restored" -ForegroundColor Green
}

# Restore global gitignore
$ignoreSrc = Join-Path $Dotfiles "git\gitignore_global"
if (Test-Path $ignoreSrc) {
  $dest = "$HomeDir\.gitignore_global"
  if ((Test-Path $dest) -and -not $Force) {
    Write-Host "SKIP $dest (use -Force to overwrite)" -ForegroundColor Yellow
  } else {
    Copy-Item -LiteralPath $ignoreSrc -Destination $dest -Force
    git config --global core.excludesFile $dest
    Write-Host "Global gitignore installed" -ForegroundColor Green
  }
}

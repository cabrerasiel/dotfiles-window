<#
.SYNOPSIS
  Crea symlinks desde WSL hacia ~/dotfiles/.config/
.DESCRIPTION
  Corre DENTRO de WSL (wsl.exe). Enlaza configs de Starship, tmux, Neovim y git
  desde el directorio compartido de Windows (/mnt/c/Users/<user>/dotfiles).
#>

$wslDotfiles = "$env:USERPROFILE/dotfiles"  # ruta WSL-style

$links = @(
  @{ src = "$wslDotfiles/.config/starship.toml"; dest = "~/.config/starship.toml" }
  @{ src = "$wslDotfiles/.config/tmux/tmux.conf"; dest = "~/.config/tmux/tmux.conf" }
  @{ src = "$wslDotfiles/.config/tmux/gitmux.conf"; dest = "~/.config/tmux/gitmux.conf" }
  @{ src = "$wslDotfiles/nvim"; dest = "~/.config/nvim" }
  @{ src = "$wslDotfiles/git/gitignore_global"; dest = "~/.gitignore_global" }
)

foreach ($link in $links) {
  $src = $link.src
  $dest = $link.dest
  $parent = Split-Path $dest -Parent
  if (-not (Test-Path $parent)) { mkdir -p $parent | Out-Null }

  if ((Test-Path $dest) -and (-not (Get-Item $dest).LinkType)) {
    Write-Host "SKIP $dest (real file exists, remove manually)" -ForegroundColor Yellow
    continue
  }
  Remove-Item $dest -Force -ErrorAction SilentlyContinue
  New-Item -ItemType SymbolicLink -Path $dest -Target $src -Force | Out-Null
  Write-Host "Linked $src -> $dest" -ForegroundColor Green
}

# Git config dentro de WSL
Get-Content "$wslDotfiles/git/config" | ForEach-Object {
  if ($_ -match '^\t(\w+) = (.+)$') {
    git config --global $Matches[1] $Matches[2]
  }
}
git config --global core.excludesFile "~/.gitignore_global"
Write-Host "Git config applied" -ForegroundColor Green

Write-Host "`nWSL dotfiles ready. Run 'source ~/.bashrc' or similar." -ForegroundColor Green

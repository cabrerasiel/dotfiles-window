# dotfiles

Configuraciones personales para Windows: GlazeWM, Zebar, psmux, PowerShell, Chocolatey, opencode, VS Code, Neovim, tmux, Starship, Windows Terminal, git.

## Instalación

```powershell
# 1. Clonar el repo
git clone <url-del-repo> "$env:USERPROFILE\dotfiles"

# 2. Ejecutar bootstrap (como Admin para instalar paquetes)
cd "$env:USERPROFILE\dotfiles"
.\bootstrap.ps1        # solo copia lo que no existe
.\bootstrap.ps1 -Force # sobrescribe todo
```

## Componentes

| Componente | Origen → Destino |
|---|---|
| **Chocolatey packages** | `choco-packages.txt` → `choco install` |
| **Winget packages** | `winget-packages.txt` → `winget import` (manual) |
| **PowerShell profile** | `Microsoft.PowerShell_profile.ps1` → `$PROFILE` |
| **psmux** | `.psmux.conf` → `~/.psmux.conf`, `.psmux/` → `~/.psmux/` |
| **GlazeWM** | `.glzr/glazewm/` → `~/.glzr/glazewm/` |
| **Zebar** | `.glzr/zebar/` → `~/.glzr/zebar/` |
| **opencode session** | `.opencode/` → `~/.opencode/` |
| **VS Code settings** | `vscode/settings.json` → `%APPDATA%/Code/User/settings.json` |
| **VS Code extensions** | `vscode/extensions.txt` → `code --install-extension` |
| **Neovim** | `nvim/` → `%LOCALAPPDATA%/nvim/` |
| **Starship prompt** | `.config/starship.toml` → `~/.config/starship.toml` |
| **tmux** | `.config/tmux/tmux.conf`, `gitmux.conf` → `~/.config/tmux/` |
| **Scoop** | `.config/scoop/config.json` → `~/.config/scoop/config.json` |
| **NuGet** | `nuget/NuGet.config` → `%APPDATA%/NuGet/NuGet.config` |
| **Windows Terminal** | `windows-terminal/settings.json` → `%LOCALAPPDATA%/Packages/Microsoft.WindowsTerminal_*/LocalState/settings.json` |
| **Git config** | `git/config` → `git config --global` |
| **Git global ignore** | `git/gitignore_global` → `~/.gitignore_global` |
| **Scripts** | `scripts/` → `~/.glzr/scripts/` + helpers para GlazeWM |

## WSL

```bash
# Dentro de WSL:
wsl pwsh -c "~/.config/dotfiles/scripts/setup-wsl.ps1"
```

Crea symlinks desde `~/dotfiles/` hacia `~/.config/` para compartir configs de Starship, tmux, Neovim y git entre Windows y WSL.

## Requisitos

- PowerShell 7+
- Chocolatey (para instalar paquetes via bootstrap)
- Windows 10/11

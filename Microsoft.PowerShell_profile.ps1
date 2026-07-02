# PowerShell Profile — equivalente a .bashrc
# Ruta: $PROFILE (~\Documents\PowerShell\Microsoft.PowerShell_profile.ps1)
# ============================================
# ENVIRONMENT VARIABLES
# ============================================
$env:EDITOR = 'nvim'
$env:VISUAL = 'nvim'
$env:BAT_THEME = 'ansi'
$env:FZF_DEFAULT_OPTS = "--preview 'bat --style=numbers --color=always --line-range :500 {}'"
$env:NVM_DIR = "$env:ProgramData\nvm"

# === PATH ===
# Recargar PATH completo desde el registro (para que choco, git, etc. funcionen)
$env:Path = [Environment]::GetEnvironmentVariable("Path", "Machine") + ";" + [Environment]::GetEnvironmentVariable("Path", "User")

$opencodeBin = "$env:USERPROFILE\.opencode\bin"
if (Test-Path $opencodeBin) {
    $env:Path = "$opencodeBin;$env:Path"
}

# NVM for Windows
$nvmBin = "$env:ProgramData\nvm"
if (Test-Path $nvmBin) {
    $env:Path = "$nvmBin;$env:Path"
}

# Swift toolchain
$swiftBin = "$env:LOCALAPPDATA\Programs\Swift\Toolchains\6.3.2+Asserts\usr\bin"
$swiftRuntime = "$env:LOCALAPPDATA\Programs\Swift\Runtimes\6.3.2\usr\bin"
if (Test-Path $swiftBin) {
    $env:Path = "$swiftBin;$swiftRuntime;$env:Path"
    # Set Visual Studio 2026 env vars for Swift
    $vsvc = "C:\Program Files\Microsoft Visual Studio\18\Enterprise\VC\Tools\MSVC\14.51.36231"
    $winkit = "C:\Program Files (x86)\Windows Kits\10"
    $winkitVer = "10.0.26100.0"
    $env:LIB = "$vsvc\lib\x64;$winkit\Lib\$winkitVer\ucrt\x64;$winkit\Lib\$winkitVer\um\x64"
    $env:LIBPATH = "$vsvc\lib\x64"
    $env:INCLUDE = "$vsvc\include;$winkit\Include\$winkitVer\ucrt;$winkit\Include\$winkitVer\um;$winkit\Include\$winkitVer\shared"
}

# Scoop shims
$scoopShims = "$env:USERPROFILE\scoop\shims"
if (Test-Path $scoopShims) {
    $env:Path = "$scoopShims;$env:Path"
}

# ============================================
# KEYBINDINGS
# ============================================
# Ctrl+W: delete word backward (like bash)
Set-PSReadLineKeyHandler -Chord 'Ctrl+w' -Function BackwardKillWord

# Ctrl+U: clear line (like bash)
Set-PSReadLineKeyHandler -Chord 'Ctrl+u' -Function BackwardDeleteLine

# ============================================
# FUNCTIONS
# ============================================

# fe — fzf file finder -> opens in nvim
function fe {
    $file = fzf --preview 'bat --style=numbers --color=always --line-range :500 {}'
    if ($file) {
        nvim $file
    }
}

# edit-profile — abre el perfil de PowerShell para editar
function edit-profile {
    nvim $PROFILE
}

# reload-profile — recarga el perfil (equivalente a source ~/.bashrc)
function reload-profile {
    . $PROFILE
    Write-Host "Profile reloaded" -ForegroundColor Green
}
Remove-Alias -Name rp -Force -ErrorAction SilentlyContinue
Set-Alias -Name rp -Value reload-profile

# ============================================
# ALIASES (sobreescriben comandos nativos)
# ============================================

# Editor
Set-Alias -Name vi -Value nvim
Set-Alias -Name vim -Value nvim

# cls ya existe como Clear-Host

# cat -> bat (con estilo)
Remove-Alias -Name cat -ErrorAction SilentlyContinue
function cat { bat --style=plain --paging=never @args }

# ls -> eza con iconos
function ls { eza --icons=always --group-directories-first @args }
function ll { eza -lh --icons=always --group-directories-first @args }
function la { eza -a --icons=always --group-directories-first @args }
function lla { eza -lah --icons=always --group-directories-first @args }
function tree { eza --tree --icons=always @args }

# bashconfig / nb (editar config)
function bashconfig { nvim $PROFILE }
function nb { nvim $PROFILE }

# ============================================
# PROMPT — Starship
# ============================================
Invoke-Expression (&starship init powershell)

# ============================================
# ZOXIDE — navegación rápida
# ============================================
Invoke-Expression (& { (zoxide init powershell --cmd cd | Out-String) })

# `z` como alias rápido (como en bash/zsh)
function z { __zoxide_z @args }
function zi { __zoxide_zi @args }

# Atajos de zoxide (equivalentes a zp, zr, ze del bashrc)
function zp { zoxide add "$env:USERPROFILE\Proyects"; __zoxide_z Proyects }
function zr { zoxide add "$env:USERPROFILE\source\repos"; __zoxide_z repos }
function ze { zoxide add "$env:USERPROFILE\source\repos\eCare"; __zoxide_z eCare }

# ============================================
# NVM — Node Version Manager (Windows)
# ============================================
# nvm-windows se encarga de actualizar el PATH vía registro
# no necesita sourcing de .ps1

# ============================================
# MODULE IMPORTS
# ============================================
# Terminal-Icons (si está instalado)
if (Get-Module -ListAvailable -Name Terminal-Icons) {
    Import-Module -Name Terminal-Icons -ErrorAction SilentlyContinue
}

# PSReadLine
Set-PSReadLineOption -EditMode Emacs
try {
    Set-PSReadLineOption -PredictionSource HistoryAndPlugin
    Set-PSReadLineOption -PredictionViewStyle Inline
} catch {}

# ============================================
# OH-MY-POSH (alternativa a oh-my-bash)
# ============================================
if (Get-Module -ListAvailable -Name oh-my-posh) {
    # oh-my-posh init pwsh --config "$env:POSH_THEMES_PATH\powerlevel10k_lean.omp.json" | Invoke-Expression
}


function Invoke-MsBuild {
    param (
        [string]$Solution = ""
    )

    # Lista de rutas posibles para VS 2022
    $rutasVS = @(
        "C:\Program Files\Microsoft Visual Studio\18\Enterprise\MSBuild\Current\Bin\MSBuild.exe"
    )

    # Buscar la ruta que exista en el disco
    $msbuildPath = $rutasVS | Where-Object { Test-Path $_ } | Select-Object -First 1

    if ($msbuildPath) {
        if ($Solution) {
            & $msbuildPath $Solution
        } else {
            & $msbuildPath
        }
    } else {
        Write-Error "No se encontró MSBuild para Visual Studio 2018 en las rutas por defecto."
    }
}

# Crear un alias corto para usarlo fácilmente
Set-Alias -Name msbuild -Value Invoke-MsBuild


function Start-ClassicWeb {
    param (
        [int]$Port = 8080
    )
    
    $iisExpress = "C:\Program Files\IIS Express\iisexpress.exe"
    $projectPath = Resolve-Path ".\ILS.Authorization.Middleware.ServiceHost"

    if (Test-Path $iisExpress) {
        Write-Host "Iniciando IIS Express en el puerto $Port..." -ForegroundColor Cyan
        Write-Host "Presiona Ctrl+C para detener el servidor." -ForegroundColor Yellow
        # Ejecuta IIS Express apuntando a la carpeta de tu proyecto web
        & $iisExpress /path:$projectPath /port:$Port
    } else {
        Write-Error "IIS Express no está instalado en este equipo."
    }
}
Set-Alias -Name runweb Start-ClassicWeb

# ============================================
# WELCOME
# ============================================
# Write-Host "PowerShell ready! 🚀" -ForegroundColor Cyan

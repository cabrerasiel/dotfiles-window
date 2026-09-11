# Actualiza variables de entorno globales de psmux con la rama e info de git
# del pane actualmente activo. psmux no soporta shell command substitution
# (#()) en el status-bar como tmux, así que este loop en segundo plano
# hace polling (cada 3s, igual que el cache de 5s de WezTerm) y expone el
# resultado vía `set-environment -g`, leíble en status-right como
# #{PSMUX_GIT_BRANCH} / #{PSMUX_GIT_CHANGES}.

while ($true) {
  try {
    $cwd = (psmux display-message -p "#{pane_current_path}").Trim()
  } catch {
    $cwd = $null
  }

  $branch = ""
  $changes = 0

  if ($cwd -and (Test-Path $cwd)) {
    $branchOutput = git -C $cwd rev-parse --abbrev-ref HEAD 2>$null
    if ($LASTEXITCODE -eq 0 -and $branchOutput) {
      $branch = $branchOutput.Trim()
      $statusOutput = git -C $cwd status --porcelain 2>$null
      if ($statusOutput) { $changes = ($statusOutput | Measure-Object -Line).Lines }
    }
  }

  psmux set-environment -g PSMUX_GIT_BRANCH "$branch" | Out-Null
  psmux set-environment -g PSMUX_GIT_CHANGES "$changes" | Out-Null

  Start-Sleep -Seconds 3
}

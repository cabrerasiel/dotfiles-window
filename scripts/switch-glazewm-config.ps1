Add-Type -AssemblyName System.Windows.Forms

$dir = "$env:USERPROFILE\.glzr\glazewm"
$count = [System.Windows.Forms.Screen]::AllScreens.Length

if ($count -ge 3) {
  $template = "$dir\config-3mon.yaml"
  Write-Host "Detectados $count monitores -> usando config-3mon.yaml"
} else {
  $template = "$dir\config-2mon.yaml"
  Write-Host "Detectados $count monitores -> usando config-2mon.yaml"
}

Copy-Item -Path $template -Destination "$dir\config.yaml" -Force

try {
  & "glazewm" command wm-reload-config 2>&1 | Out-Null
  Write-Host "Config recargada en GlazeWM"
} catch {
  Write-Host "GlazeWM no está corriendo. La config se usará en el próximo inicio."
}

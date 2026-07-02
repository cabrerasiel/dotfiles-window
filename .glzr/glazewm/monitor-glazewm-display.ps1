Add-Type -AssemblyName System.Windows.Forms

$dir = "$env:USERPROFILE\.glzr\glazewm"
$currentCount = -1

while ($true) {
  $count = [System.Windows.Forms.Screen]::AllScreens.Length
  if ($count -ne $currentCount) {
    $currentCount = $count
    if ($count -ge 3) {
      $template = "$dir\config-3mon.yaml"
    } else {
      $template = "$dir\config-2mon.yaml"
    }
    Copy-Item -Path $template -Destination "$dir\config.yaml" -Force
    try {
      & "glazewm" command wm-reload-config 2>&1 | Out-Null
    } catch {}
  }
  Start-Sleep -Seconds 5
}

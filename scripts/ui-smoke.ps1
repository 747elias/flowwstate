param([Parameter(Mandatory)][int]$AppProcessId)
$ErrorActionPreference='Stop'
Add-Type -AssemblyName UIAutomationClient
Add-Type -AssemblyName UIAutomationTypes
$condition = New-Object System.Windows.Automation.PropertyCondition([System.Windows.Automation.AutomationElement]::ProcessIdProperty, $AppProcessId)
$window = [System.Windows.Automation.AutomationElement]::RootElement.FindFirst([System.Windows.Automation.TreeScope]::Children,$condition)
if ($null -eq $window) { throw 'No accessible application window found' }
function Find-Element([string]$name) {
  $c = New-Object System.Windows.Automation.PropertyCondition([System.Windows.Automation.AutomationElement]::NameProperty, $name)
  return $window.FindFirst([System.Windows.Automation.TreeScope]::Descendants,$c)
}
Add-Type -AssemblyName System.Drawing
$bounds=$window.Current.BoundingRectangle
$image=New-Object System.Drawing.Bitmap([int]$bounds.Width,[int]$bounds.Height)
$graphics=[System.Drawing.Graphics]::FromImage($image)
$graphics.CopyFromScreen([int]$bounds.X,[int]$bounds.Y,0,0,$image.Size)
New-Item -ItemType Directory -Path artifacts/screenshots -Force | Out-Null
$image.Save((Join-Path (Get-Location) 'artifacts/screenshots/dashboard.png'),[System.Drawing.Imaging.ImageFormat]::Png)
$graphics.Dispose();$image.Dispose()

foreach ($page in @('Aufgaben','Projekte','Kalender','AutoPlan','Focus Studio','Sound Studio','Workspaces','Analytics','Tagesrückblick','Einstellungen','Über Flowstate','Dashboard')) {
  $item=Find-Element $page
  if ($null -eq $item) { throw "Missing navigation: $page" }
  $pattern=$null
  if ($item.TryGetCurrentPattern([System.Windows.Automation.SelectionItemPattern]::Pattern,[ref]$pattern)) { $pattern.Select() }
  elseif ($item.TryGetCurrentPattern([System.Windows.Automation.InvokePattern]::Pattern,[ref]$pattern)) { $pattern.Invoke() }
  else { throw "Navigation is not keyboard accessible: $page" }
  Start-Sleep -Milliseconds 500
  if ((Get-Process -Id $AppProcessId -ErrorAction Stop).HasExited) { throw "App exited on $page" }
  Write-Output "Navigation OK: $page"
}
if ($null -eq (Find-Element 'Raum für das Wesentliche.')) { throw 'Dashboard content not rendered' }

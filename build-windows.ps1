param([switch]$SkipChecks)
$ErrorActionPreference = 'Stop'
Set-Location $PSScriptRoot
if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    throw 'Rust stable mit MSVC-Toolchain fehlt. Siehe README: Aus Quellcode bauen.'
}
function Invoke-Cargo {
    param([string[]]$CargoArguments)
    & cargo @CargoArguments
    if ($LASTEXITCODE -ne 0) { throw "Cargo fehlgeschlagen: $CargoArguments" }
}
if (-not $SkipChecks) {
    Invoke-Cargo @('fmt', '--all', '--', '--check')
    Invoke-Cargo @('test', '--workspace', '--locked', '--target', 'x86_64-pc-windows-msvc')
    Invoke-Cargo @('clippy', '--workspace', '--all-targets', '--locked', '--target', 'x86_64-pc-windows-msvc', '--', '-D', 'warnings')
}
Invoke-Cargo @('build', '--release', '--locked', '--target', 'x86_64-pc-windows-msvc')
Write-Host 'Fertig: target\x86_64-pc-windows-msvc\release\folder-mind.exe'

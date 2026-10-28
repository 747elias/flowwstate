$ErrorActionPreference = 'Stop'
Set-Location (Join-Path $PSScriptRoot '..')
[xml]$props = Get-Content Directory.Build.props
$version = $props.Project.PropertyGroup.Version
& dotnet test tests/Flowstate.Tests -c Release
if ($LASTEXITCODE -ne 0) { throw 'Tests failed' }
& dotnet publish src/Flowstate.App -c Release -p:Platform=x64 -r win-x64 --self-contained true -o artifacts/app
if ($LASTEXITCODE -ne 0) { throw 'Publish failed' }
& "${env:ProgramFiles(x86)}\Inno Setup 6\ISCC.exe" "/DAppVersion=$version" installer/Flowstate.iss
if ($LASTEXITCODE -ne 0) { throw 'Installer failed' }
$file = Get-ChildItem artifacts/installer/*.exe | Select-Object -First 1
$hash = (Get-FileHash $file.FullName -Algorithm SHA256).Hash.ToLowerInvariant()
"$hash  $($file.Name)" | Set-Content artifacts/installer/SHA256SUMS.txt -Encoding ascii

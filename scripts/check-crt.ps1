# Fails if a Windows binary imports the dynamic VC++ runtime
# (VCRUNTIME140*.dll, MSVCP140*.dll). Prints `dumpbin /dependents` for the log.
param([Parameter(Mandatory = $true)][string]$Exe)
$ErrorActionPreference = 'Stop'
$vswhere = Join-Path ${env:ProgramFiles(x86)} 'Microsoft Visual Studio\Installer\vswhere.exe'
$vs = & $vswhere -latest -products * -property installationPath
$dumpbin = Get-ChildItem -Path "$vs\VC\Tools\MSVC\*\bin\*\*\dumpbin.exe" | Select-Object -First 1
if (-not $dumpbin) { throw "dumpbin.exe not found under $vs" }
Write-Host "dumpbin: $($dumpbin.FullName)"
$out = & $dumpbin.FullName /nologo /headers $Exe | Select-String -Pattern 'machine \('
Write-Host "Machine: $out"
$deps = & $dumpbin.FullName /nologo /dependents $Exe
$deps | ForEach-Object { Write-Host $_ }
$bad = $deps | Select-String -Pattern '(?i)(vcruntime|msvcp)\d+.*\.dll'
if ($bad) { throw "VC++ runtime import found: $($bad -join ', ')" }
Write-Host "OK: $Exe imports no VC++ runtime DLL (static CRT)."

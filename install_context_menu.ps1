# Get the path to the current YAAM executable
$yaamPath = "$PSScriptRoot\target\release\yaam.exe"

if (-not (Test-Path $yaamPath)) {
    Write-Error "Could not find yaam.exe! Make sure you ran 'cargo build --release' first."
    exit
}

$regPathFiles = "HKCU:\Software\Classes\*\shell\YAAM"
$regPathFolders = "HKCU:\Software\Classes\Directory\shell\YAAM"

New-Item -Path $regPathFiles -Force | Out-Null
Set-ItemProperty -Path $regPathFiles -Name "(default)" -Value "Extract with YAAM"
New-Item -Path "$regPathFiles\command" -Force | Out-Null
Set-ItemProperty -Path "$regPathFiles\command" -Name "(default)" -Value "`"$yaamPath`" extract `"%1`""

New-Item -Path $regPathFolders -Force | Out-Null
Set-ItemProperty -Path $regPathFolders -Name "(default)" -Value "Pack to .xip"
New-Item -Path "$regPathFolders\command" -Force | Out-Null
Set-ItemProperty -Path "$regPathFolders\command" -Name "(default)" -Value "`"$yaamPath`" pack `"%1`" `"%1.xip`""

Write-Host "? YAAM Context Menu installed!" -ForegroundColor Green
Write-Host "Try right-clicking a folder and selecting 'Pack to .xip'"
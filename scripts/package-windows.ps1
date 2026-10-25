param([Parameter(Mandatory=$true)][string]$Tag)
$ErrorActionPreference='Stop'
if($Tag -notmatch '^(v[0-9]+\.[0-9]+\.[0-9]+|manual-[0-9]+)$'){throw 'Expected a version tag such as v0.4.0'}
$root=Split-Path $PSScriptRoot -Parent
$packageRoot=Join-Path $root "target/release-package/PhaseUP-$Tag-Windows"
New-Item -ItemType Directory -Path $packageRoot -Force | Out-Null
Copy-Item -LiteralPath (Join-Path $root 'target/release/phaseup.exe') -Destination $packageRoot -Force
# Package runtime assets, excluding source artwork, music experiments and backups.
foreach($relative in @('fonts','layouts','neon_forge/cards','neon_forge/ui')){
    $source=Join-Path $root "assets/$relative"
    $destination=Join-Path $packageRoot "assets/$relative"
    New-Item -ItemType Directory -Path $destination -Force | Out-Null
    Get-ChildItem -LiteralPath $source -File | Copy-Item -Destination $destination -Force
}
$instructions=@'
PhaseUP

Extract the entire ZIP before playing, then launch phaseup.exe.
Keep the assets folder beside phaseup.exe; it contains the artwork and fonts.
The game works offline. Settings are saved automatically.
Music and sound effects are planned for a future version.
'@
Set-Content -LiteralPath (Join-Path $packageRoot 'START-HERE.txt') -Value $instructions
$archive=Join-Path $root "PhaseUP-$Tag-Windows.zip"
Get-ChildItem -LiteralPath $packageRoot | Compress-Archive -DestinationPath $archive -Force
Write-Output $archive

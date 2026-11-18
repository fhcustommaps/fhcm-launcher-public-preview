param([Parameter(Mandatory)][string]$InputDirectory)
$ErrorActionPreference = 'Stop'

$source = (Resolve-Path -LiteralPath $InputDirectory).Path
$manifest = Get-Content -Raw -LiteralPath (Join-Path $PSScriptRoot 'build-inputs.json') | ConvertFrom-Json
$verified = foreach($item in $manifest.files) {
    $relative = $item.path
    if ($relative -notmatch '^[a-zA-Z0-9._/-]+$' -or $relative -match '(^|/)\.{1,2}(/|$)' -or $relative.StartsWith('/')) {
        throw "Invalid build input path $relative"
    }
    $file = Get-Item -LiteralPath (Join-Path $source $relative)
    if ($file.PSIsContainer -or $file.Length -ne $item.size -or
        (Get-FileHash -LiteralPath $file.FullName -Algorithm SHA256).Hash -ne $item.sha256) {
        throw "Build input did not match the manifest at $relative"
    }
    [pscustomobject]@{ Source = $file.FullName; Target = Join-Path $PSScriptRoot $relative }
}
foreach($file in $verified) {
    New-Item -ItemType Directory -Path (Split-Path $file.Target) -Force | Out-Null
    Copy-Item -LiteralPath $file.Source -Destination $file.Target -Force
}
Write-Output "Staged $($verified.Count) verified build inputs."

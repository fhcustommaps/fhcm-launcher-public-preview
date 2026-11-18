$ErrorActionPreference = 'Stop'
Push-Location $PSScriptRoot
try {
    $node = (Get-Command node -ErrorAction Stop).Source
    & $node build.mjs --native
    if($LASTEXITCODE){throw 'Frontend build failed'}
    & $node node_modules/@tauri-apps/cli/tauri.js build --bundles nsis
    if($LASTEXITCODE){throw 'Native build failed'}
} finally {
    Pop-Location
}

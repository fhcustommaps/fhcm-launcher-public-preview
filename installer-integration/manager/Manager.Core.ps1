$ErrorActionPreference = 'Stop'
function Get-Fh2Hash([string]$LiteralPath,[string]$Algorithm='SHA256') {
    if($Algorithm -ne 'SHA256'){throw 'Only SHA256 is supported.'}
    $stream=[IO.File]::OpenRead($LiteralPath);$sha=[Security.Cryptography.SHA256]::Create()
    try {$hash=[BitConverter]::ToString($sha.ComputeHash($stream)).Replace('-','');return [pscustomobject]@{Hash=$hash;Path=$LiteralPath;Algorithm='SHA256'}}
    finally {$sha.Dispose();$stream.Dispose()}
}
$script:Config = Get-Content -LiteralPath (Join-Path $PSScriptRoot 'profiles.json') -Raw | ConvertFrom-Json
function Get-Fh2Profile([string]$Name) {
    if ($Name -notin @('play','dev')) { throw 'Unknown profile.' }
    return $script:Config.profiles.$Name
}
function Assert-Fh2Stopped {
    # FHCM sandbox save lease guard
    $fhcmLease=Join-Path $script:Config.devRoot 'fhcm-sandbox/active-session.json'
    if($env:FHCM_SANDBOX_OWNER -and -not(Test-Path -LiteralPath $fhcmLease)){throw 'Sandbox session expired; launch cancelled.'}
    if(Test-Path -LiteralPath $fhcmLease){$fhcmOwner=(Get-Content -LiteralPath $fhcmLease -Raw | ConvertFrom-Json).id;if($env:FHCM_SANDBOX_OWNER -ne $fhcmOwner){throw 'FHCM Sandbox owns the active save. Close the game and let FHCM restore the personal save before using the map manager.'}}

    $running = @(Get-Process -Name azimuth -ErrorAction SilentlyContinue)
    if ($running.Count) { throw 'Close the running FH2 game before launching another profile or activating a map.' }
}
function Read-Fh2Ini([string]$Path) {
    $result = @{}; $section = ''
    foreach ($line in Get-Content -LiteralPath $Path) {
        if ($line -match '^\s*\[(.+)\]') { $section = $Matches[1].ToLowerInvariant() }
        elseif ($line -match '^\s*([^;=#]+?)\s*=\s*(.*?)\s*$') { $result[($section + '/' + $Matches[1]).ToLowerInvariant()] = $Matches[2] }
    }
    return $result
}
function Test-Fh2Profile([string]$Name,$VerifiedMap=$null) {
    $p = Get-Fh2Profile $Name
    foreach ($path in @((Join-Path $p.runtime 'azimuth.exe'),(Join-Path $p.runtime 'data\azimuth.ini'),(Join-Path $p.game 'Mount\forza_durango_release_final.exe'),$p.save,$p.content) | Where-Object {$_}) {
        if (-not (Test-Path -LiteralPath $path)) { throw "Missing profile file or folder: $path" }
    }
    $ini = Read-Fh2Ini (Join-Path $p.runtime 'data\azimuth.ini')
    foreach ($key in @('launcher/game_folder','title-1a96545b/folder')) {
        if ($ini[$key] -ne $p.game) { throw "Profile game path mismatch: $key" }
    }
    if ($p.content -and $ini['title-1a96545b/content'] -ne $p.content) { throw 'DLC path mismatch.' }
    if ($Name -eq 'dev') {
        $devRoot = [IO.Path]::GetFullPath($script:Config.devRoot).TrimEnd('\') + '\'
        foreach ($path in @($p.runtime,$p.game,$p.save,$p.content) | Where-Object {$_}) {
            $absolute = [IO.Path]::GetFullPath($path)
            if (-not $absolute.StartsWith($devRoot,[StringComparison]::OrdinalIgnoreCase)) { throw 'DevLab points outside its isolated folder.' }
            if ((Get-Item -LiteralPath $path).Attributes -band [IO.FileAttributes]::ReparsePoint) { throw 'DevLab root must not be a link.' }
        }
            if ($p.save -eq $script:Config.profiles.play.save) { throw 'Profiles must not share saves.' }
        $expectedDatabase=$p.baselineDatabaseSha256
        if(Get-Command Get-Fh2ActiveMap -ErrorAction SilentlyContinue){$map=$VerifiedMap;if(-not $map){$map=Get-Fh2ActiveMap};$expectedDatabase=($map.changes | Where-Object {$_.target -eq 'Mount/mediaoverride/stripped/gamedbRC.slt'}).sha256}
        if ($expectedDatabase -and ((Get-Fh2Hash -LiteralPath (Join-Path $p.game 'Mount\mediaoverride\stripped\gamedbRC.slt') -Algorithm SHA256).Hash -ne $expectedDatabase)) { throw 'DevLab database differs from the selected map package.' }
    }
    return "$($p.label): paths verified; save is $($p.save)"
}
function Start-Fh2Profile([string]$Name,[switch]$Wait,[object]$LaunchMap=$null) {
    Assert-Fh2Stopped
    if($Name -eq 'dev' -and !$LaunchMap){$LaunchMap=Get-Fh2ActiveMap}
    $validation = Test-Fh2Profile $Name $launchMap
    $p = Get-Fh2Profile $Name
    if ($Name -eq 'dev') { Assert-Fh2ActiveMap $launchMap; if($launchMap.id -like 'fhcm-*'){Reset-Fh2SkidpadSpawn $launchMap} }
    $record = [ordered]@{ time = (Get-Date).ToString('o'); profile = $Name; runtime = $p.runtime; game = $p.game; save = $p.save }
    ($record | ConvertTo-Json -Compress) | Add-Content -LiteralPath (Join-Path $PSScriptRoot 'launch-history.jsonl') -Encoding UTF8
    $process = Start-Process -FilePath (Join-Path $p.runtime 'azimuth.exe') -ArgumentList '--skip-launcher' -WorkingDirectory $p.runtime -WindowStyle Hidden -PassThru
    if ($Wait) { $process.WaitForExit() }
    return $validation
}
. (Join-Path $PSScriptRoot 'Manager.Maps.ps1')



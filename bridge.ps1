param([Parameter(Mandatory)][string]$ManagerDirectory,[Parameter(Mandatory)][ValidatePattern('^(play|rogue-pch|rogue-gunsai|fhcm-[a-z0-9-]+)$')][string]$MapId,[switch]$PrepareOnly)
$ErrorActionPreference='Stop'
. (Join-Path $ManagerDirectory 'Manager.Core.ps1')
Assert-Fh2Stopped
$started=Get-Date
if($MapId -eq 'play'){Start-Fh2Profile 'play'}
else {
    $active=Get-Fh2ActiveMap
    if($active.id -ne $MapId){Set-Fh2Map $MapId | Out-Null;$active=Get-Fh2ActiveMap}
    if($env:FHCM_PCH_RACER_MODE -eq 'tour'){
        $descriptor=Get-Content -Raw -LiteralPath (Join-Path $script:Config.devRoot "maps/$MapId/fhcm-map.json") | ConvertFrom-Json
        if($descriptor.racerAdapter -cne 'pch-racers-v1' -or $descriptor.id -cne 'pacific-estate36-racers'){
            throw 'This PCH package does not support Experimental Tour.'
        }
        $active.spawnPosition=$descriptor.tourSpawn.spawnPosition
        $active.spawnFacing=$descriptor.tourSpawn.spawnFacing
        if([string]::IsNullOrEmpty($active.spawnPosition) -or [string]::IsNullOrEmpty($active.spawnFacing)){
            throw 'The Tour parking spawn is missing from the package.'
        }
    }
    if($env:FHCM_PCH_TRAFFIC_SETTINGS){
        $env:FHCM_PCH_TRAFFIC_SPAWN=$active.spawnPosition
        $env:FHCM_PCH_TRAFFIC_FACING=$active.spawnFacing
    }
    if($PrepareOnly){
        Test-Fh2Profile 'dev' $active | Out-Null
        Assert-Fh2ActiveMap $active
        Reset-Fh2SkidpadSpawn $active
        return
    }
    Start-Fh2Profile 'dev' -LaunchMap $active
}
Start-Sleep -Seconds 8
$profileName=if($MapId -eq 'play'){'play'}else{'dev'}
$logs=Join-Path $script:Config.profiles.$profileName.runtime 'data\logs'
$crashes=@(Get-ChildItem -LiteralPath $logs -Filter '*.dmp' -ErrorAction SilentlyContinue | Where-Object {$_.CreationTime -ge $started})
if($crashes.Count){throw "Azimuth crashed during startup. Map activation completed; see $logs"}
if(-not (Get-Process azimuth -ErrorAction SilentlyContinue)){throw 'Azimuth exited during startup.'}

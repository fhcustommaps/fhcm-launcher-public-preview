# Every package owns the same fifteen files. Saves are isolated by map family.
$script:Fh2MapTargets=@(
 'Mount/media/tracks/homespace02/GeoChunk0.minizip',
 'Mount/media/tracks/homespace02/GeoChunk1.minizip',
 'Mount/media/tracks/homespace02/GeoChunk2.minizip',
 'Mount/media/tracks/homespace02/GeoChunk3.minizip',
 'Mount/mediaoverride/stripped/gamedbRC.slt',
 'Mount/mediaoverride/StateFlow/boot.xml',
 'Mount/mediaoverride/StateFlow/free_roam_start.xml',
 'Mount/media/tracks/homespace02/ribbon_00/HomeSpace02_00.pvs',
 'Mount/media/tracks/homespace02/bin.zip',
 'Mount/media/tracks/homespace02/ChunkMap0.dat',
 'Mount/media/tracks/homespace02/ChunkMap1.dat',
 'Mount/media/tracks/homespace02/ChunkMap2.dat',
 'Mount/media/tracks/homespace02/ChunkMap3.dat',
 'Mount/media/zipmanifest.xml',
 'Mount/mediaoverride/zipmanifest.xml'
)
function Assert-Fh2DevPaths([string[]]$Paths){
 $root=[IO.Path]::GetFullPath($script:Config.devRoot).TrimEnd('\')+'\'
 foreach($path in $Paths){
  if(-not [IO.Path]::GetFullPath($path).StartsWith($root,[StringComparison]::OrdinalIgnoreCase)){throw 'Map transaction path leaves DevLab.'}
 }
}
# FHCM_LIBRARY_V1
. (Join-Path $PSScriptRoot 'Manager.Library.ps1')
function Get-Fh2Map([string]$MapId) {
 if($MapId -like 'fhcm-*'){return Get-FhcmLibraryMap $MapId}
 if($MapId -ne 'stock'){throw 'Unregistered map package.'}
 $root=Join-Path $script:Config.devRoot 'maps/stock'
 $map=Get-Content (Join-Path $root 'map.json') -Raw|ConvertFrom-Json
 if($map.schema -ne 2 -or $map.id -ne 'stock' -or $map.saveSlot -ne 'baseline'){throw 'Invalid baseline.'}
 foreach($change in $map.changes){if($change.target -cnotin $script:Fh2MapTargets){throw 'Invalid baseline target.'};if(-not $change.absent){if((Get-Fh2Hash (Join-Path $root $change.source)).Hash -ne $change.sha256){throw 'Baseline changed.'}}}
 return $map
}
function Assert-Fh2MapFiles($Map){
 $game=$script:Config.profiles.dev.game
 if($Map.customCarAddon){
  foreach($entry in $Map.customCarAddon.files.PSObject.Properties){
   $asset=Join-Path $game $entry.Name
   Assert-Fh2DevPaths @($asset)
   if((Get-Fh2Hash -LiteralPath $asset).Hash -ne $entry.Value){throw "Custom car asset differs: $asset"}
  }
 }
 foreach($change in $Map.changes){
  $target=Join-Path $game $change.target
  if($change.absent){if(Test-Path -LiteralPath $target){throw "Unexpected startup override: $target"}}
  elseif((Get-Fh2Hash -LiteralPath $target -Algorithm SHA256).Hash -ne $change.sha256){
   # FHCM_SESSION_CONDITIONS_V1: one tracked generated StateFlow variant only.
   $accepted=$false
   $conditionsFile=Join-Path $script:Config.devRoot 'fhcm-conditions\active.json'
   if(($Map.id -like 'fhcm-*' -or $Map.id -in @('rogue-pch','rogue-gunsai')) -and $change.target -ceq 'Mount/mediaoverride/StateFlow/free_roam_start.xml' -and (Test-Path -LiteralPath $conditionsFile)){
    $conditions=Get-Content -LiteralPath $conditionsFile -Raw | ConvertFrom-Json
    $accepted=$conditions.map -eq $Map.id -and $conditions.originalHash -eq $change.sha256 -and $conditions.sessionHash -eq (Get-Fh2Hash -LiteralPath $target -Algorithm SHA256).Hash
   }
   if(-not $accepted){throw "Untracked map change: $target"}
  }
 }
}
function Get-Fh2ActiveMap {
 $active=Get-Content -LiteralPath (Join-Path $script:Config.devRoot 'active-map.json') -Raw | ConvertFrom-Json
 return Get-Fh2Map $active.id
}
function Assert-Fh2ActiveMap($VerifiedMap=$null) {
 $map=$VerifiedMap;if(-not $map){$map=Get-Fh2ActiveMap}; Assert-Fh2MapFiles $map
 if((Test-Path (Join-Path $script:Config.devRoot 'skidpad-enabled.flag')) -ne [bool]$map.enableVisibilityMod){throw 'Map visibility state differs.'}
 if($map.enableVisibilityMod -and (Get-Fh2Hash (Join-Path $script:Config.profiles.dev.runtime 'fh2-mods/DevLabVisibility.dll')).Hash -ne $map.visibilityModuleSha256){throw 'Visibility runtime differs.'}
 foreach($entry in $map.seattleRuntime.PSObject.Properties){if((Get-Fh2Hash (Join-Path $script:Config.profiles.dev.runtime ('fh2-mods/'+$entry.Name))).Hash -ne $entry.Value){throw 'Map runtime differs.'}}
 foreach($entry in $map.seattleData.PSObject.Properties){Assert-Fh2DevPaths @($entry.Name);if((Get-Fh2Hash $entry.Name).Hash -ne $entry.Value){throw 'Map data differs.'}}
}
function Reset-Fh2SkidpadSpawn($VerifiedMap=$null) {
 Assert-Fh2Stopped
 $save=$script:Config.profiles.dev.save
 $activeMap=$VerifiedMap;if(-not $activeMap){$activeMap=Get-Fh2ActiveMap}
 $spawn=$activeMap.spawnPosition
 if(-not $spawn){$spawn='0,2,0'}
 if($spawn -notmatch '^-?\d+(?:\.\d+)?,-?\d+(?:\.\d+)?,-?\d+(?:\.\d+)?$'){throw 'Invalid map spawn coordinates.'}
 $coords=@($spawn.Split(',') | ForEach-Object {[double]::Parse($_,[Globalization.CultureInfo]::InvariantCulture)})
 if($activeMap.id -eq 'native-road-reference'){if($spawn -cne '-3451.7354,67.5192,-1694.9530'){throw 'Native reference spawn differs from verified source triangle.'}}
 elseif($activeMap.id -eq 'shutoko-c1'){if($spawn -cne '0,42.93384,0'){throw 'Full C1 spawn differs from verified source surface.'}}
 elseif($activeMap.id -eq 'shutoko-pilot'){if([Math]::Abs($coords[0]) -gt 350 -or [Math]::Abs($coords[2]) -gt 350 -or $coords[1] -lt 2 -or $coords[1] -gt 40){throw 'Shuto pilot spawn outside imported geometry bounds.'}}
 elseif($activeMap.id -like 'fhcm-*' -or $activeMap.id -in @('rogue-pch','rogue-san-antonio','rogue-gunsai','fm4-fujimi')){if([Math]::Abs($coords[0]) -gt 50000 -or [Math]::Abs($coords[2]) -gt 50000 -or $coords[1] -lt -100 -or $coords[1] -gt 1500){throw 'PCH spawn outside authored extent.'}}
 elseif($activeMap.id -like 'fhcm-*' -or $activeMap.id -in @('rogue-seattle','rogue-seattle-baseline','rogue-seattle-network','rogue-the-eight','rogue-pch','rogue-san-antonio','rogue-gunsai','fm4-fujimi')){if([Math]::Abs($coords[0]) -gt 20000 -or [Math]::Abs($coords[2]) -gt 20000 -or [Math]::Abs($coords[1]) -gt 200){throw 'Seattle spawn outside authored extent.'}}
 elseif([Math]::Abs($coords[0]) -gt 900 -or [Math]::Abs($coords[2]) -gt 900 -or $coords[1] -lt 0.1 -or $coords[1] -gt 10){throw 'Map spawn is outside approved bounds.'}
 $backup=Join-Path $script:Config.devRoot ('backups\spawn\'+(Get-Date -Format 'yyyyMMdd-HHmmss-fff'))
 New-Item -ItemType Directory -Path $backup -Force | Out-Null
 Copy-Item -LiteralPath $save -Destination (Join-Path $backup 'player-1') -Recurse
 foreach($file in Get-ChildItem -LiteralPath $save -Filter SaveState -Recurse -File){
  $text=[Text.Encoding]::ASCII.GetString([IO.File]::ReadAllBytes($file.FullName))
  foreach($key in @('PlayerPosition','RestorePosition')){
   $pattern='(<Property Name="'+$key+'" Value=")[^"]*("/>)'
   if([regex]::Matches($text,$pattern).Count -ne 1){throw 'Unexpected save-state format; spawn reset refused.'}
   $text=[regex]::Replace($text,$pattern,('${1}'+$spawn+'${2}'))
  }
  if($activeMap.id -like 'fhcm-*' -or $activeMap.id -in @('rogue-oval','rogue-variation','rogue-seattle','rogue-seattle-baseline','rogue-seattle-network','rogue-the-eight','rogue-pch','rogue-san-antonio','rogue-gunsai','fm4-fujimi')){
   $facingPattern='(<Property Name="PlayerFacing" Value=")[^"]*("/>)'
   if([regex]::Matches($text,$facingPattern).Count -ne 1){throw 'Unexpected oval facing format; reset refused.'}
   $text=[regex]::Replace($text,$facingPattern,'${1}0,0,1${2}')
  }
  if($activeMap.id -like 'fhcm-*' -or $activeMap.id -in @('rogue-seattle','rogue-seattle-baseline','rogue-seattle-network','rogue-the-eight','rogue-pch','rogue-san-antonio','rogue-gunsai','fm4-fujimi')){
   $facing=$activeMap.spawnFacing
   if($facing -notmatch '^-?[0-9.]+,-?[0-9.]+,-?[0-9.]+$'){throw 'Invalid Seattle facing.'}
   $text=[regex]::Replace($text,'(<Property Name="PlayerFacing" Value=")[^"]*("/>)',('${1}'+$facing+'${2}'))
  }
  $null=[xml]$text
  [IO.File]::WriteAllBytes($file.FullName,[Text.Encoding]::ASCII.GetBytes($text))
  $rev=$file.Directory.Parent.FullName
  $commit=Join-Path $rev 'commit';$lines=@()
  foreach($line in [IO.File]::ReadAllLines($commit,[Text.Encoding]::ASCII)){
   if($line -match '^[0-9a-f]{64} \d+ (.+)$'){
    $data=Join-Path (Join-Path $rev 'data') $Matches[1]
    $bytes=[IO.File]::ReadAllBytes($data)
    $hash=(Get-Fh2Hash -LiteralPath $data -Algorithm SHA256).Hash.ToLowerInvariant()
    $lines+=($hash+' '+$bytes.Length+' '+$Matches[1])
   }else{$lines+=$line}
  }
  # The save manifest requires ASCII and LF; CRLF produces a device-full error.
  [IO.File]::WriteAllBytes($commit,[Text.Encoding]::ASCII.GetBytes(($lines -join "`n")+"`n"))
 }
}
function Set-Fh2Map([string]$MapId){
 Assert-Fh2Stopped
 $null=Test-Fh2Profile 'dev'
 Assert-Fh2ActiveMap
 $current=Get-Fh2ActiveMap;$next=Get-Fh2Map $MapId
 $game=$script:Config.profiles.dev.game;$save=$script:Config.profiles.dev.save
 $activeFile=Join-Path $script:Config.devRoot 'active-map.json'
 $backup=Join-Path $script:Config.devRoot ('backups\maps\'+(Get-Date -Format 'yyyyMMdd-HHmmss-fff'))
 New-Item -ItemType Directory -Path $backup -Force | Out-Null
 Copy-Item -LiteralPath $save -Destination (Join-Path $backup 'player-1') -Recurse
 Copy-Item -LiteralPath $activeFile -Destination (Join-Path $backup 'active-map.json')
 foreach($change in $current.changes){
  if($change.absent){continue}
  $dest=Join-Path $backup $change.target
  New-Item -ItemType Directory -Path (Split-Path $dest) -Force | Out-Null
  Copy-Item -LiteralPath (Join-Path $game $change.target) -Destination $dest
 }
 $flag=Join-Path $script:Config.devRoot 'skidpad-enabled.flag';$hadFlag=Test-Path -LiteralPath $flag
 Assert-Fh2DevPaths @($game,$save,$backup,$activeFile,$flag)
 if($hadFlag){Copy-Item -LiteralPath $flag -Destination (Join-Path $backup 'skidpad-enabled.flag')}
 # Match map geometry, collision and lane provider as one transaction.
 $runtimeCopies=@()
 if($next.id -like 'fhcm-*' -or $next.id -in @('rogue-seattle','rogue-seattle-baseline','rogue-seattle-network','rogue-the-eight','rogue-pch','rogue-san-antonio','rogue-gunsai','fm4-fujimi')){
  foreach($entry in $next.seattleRuntimeSource.PSObject.Properties){
   if($entry.Name -notin @('SeattleTrafficProbe.dll','SeattleRacerProbe.dll','SeattleOvalCollision.dll','SeattleSceneTrace.dll')){throw 'Unexpected Seattle runtime source.'}
   $source=[IO.Path]::GetFullPath($entry.Value)
   if(-not $source.StartsWith((Join-Path $script:Config.devRoot ('maps\'+$next.id+'\runtime\')),[StringComparison]::OrdinalIgnoreCase)){throw 'Runtime source outside package.'}
   if((Get-Fh2Hash $source).Hash -ne $next.seattleRuntime.($entry.Name)){throw 'Runtime source checksum mismatch.'}
   $target=Join-Path $script:Config.profiles.dev.runtime ('fh2-mods\'+$entry.Name)
   $old=Join-Path $backup ('runtime\'+$entry.Name)
   Assert-Fh2DevPaths @($target,$old)
   New-Item -ItemType Directory -Path (Split-Path $old) -Force | Out-Null
   if(Test-Path -LiteralPath $target){Copy-Item -LiteralPath $target -Destination $old}else{$old=$null}
   $runtimeCopies+= [pscustomobject]@{Source=$source;Target=$target;Backup=$old;Hash=$next.seattleRuntime.($entry.Name)}
  }
 }
 $swapping=$current.saveSlot -ne $next.saveSlot
 if($swapping){
  $outgoing=Join-Path $script:Config.devRoot ('save-slots\'+$current.saveSlot+'\player-1')
  $incoming=Join-Path $script:Config.devRoot ('save-slots\'+$next.saveSlot+'\player-1')
  Assert-Fh2DevPaths @($outgoing,$incoming,(Join-Path $backup 'previous-slot-snapshot'),(Join-Path $backup 'outgoing-live-save'),(Join-Path $backup 'failed-incoming-save'))
  if(-not(Test-Path -LiteralPath $incoming)){throw 'Missing map-specific save.'}
  if(Test-Path -LiteralPath $outgoing){Move-Item -LiteralPath $outgoing -Destination (Join-Path $backup 'previous-slot-snapshot')}
  New-Item -ItemType Directory -Path (Split-Path $outgoing) -Force | Out-Null
  Copy-Item -LiteralPath $save -Destination $outgoing -Recurse
 }
 try{
  # Density startup settings belong only to the experimental network package.
  $densityTarget='Mount/mediaoverride/EntityModel/AI/CarLoadingSettings.xml'
  if($densityTarget -cin $current.changes.target -and $densityTarget -cnotin $next.changes.target){
   $densityPath=Join-Path $game $densityTarget
   if(Test-Path -LiteralPath $densityPath){
    $densityRetired=Join-Path (Join-Path $backup 'retired') $densityTarget
    New-Item -ItemType Directory -Path (Split-Path $densityRetired) -Force | Out-Null
    Move-Item -LiteralPath $densityPath -Destination $densityRetired
   }
  }
  $paintTarget='Mount/media/cars/_library/materials/traffic_car_colours/rogue_neon_green.materialbin'
  if($paintTarget -cin $current.changes.target -and $paintTarget -cnotin $next.changes.target){
   $paintPath=Join-Path $game $paintTarget
   if(Test-Path -LiteralPath $paintPath){
    $paintRetired=Join-Path (Join-Path $backup 'retired') $paintTarget
    New-Item -ItemType Directory -Path (Split-Path $paintRetired) -Force | Out-Null
    Move-Item -LiteralPath $paintPath -Destination $paintRetired
   }
  }
  $paintArchive='Mount/media/cars/_library/Materials.zip'
  if($paintArchive -cin $current.changes.target -and $paintArchive -cnotin $next.changes.target){
   Copy-Item -LiteralPath (Join-Path $script:Config.devRoot 'backups/traffic-materials-original.zip') -Destination (Join-Path $game $paintArchive) -Force
  }
  foreach($change in $next.changes){
   $target=Join-Path $game $change.target
   if($change.absent){
    if(Test-Path -LiteralPath $target){
     $retired=Join-Path (Join-Path $backup 'retired') $change.target
     New-Item -ItemType Directory -Path (Split-Path $retired) -Force | Out-Null
     Move-Item -LiteralPath $target -Destination $retired
    }
   }else{
    New-Item -ItemType Directory -Path (Split-Path $target) -Force | Out-Null
    Copy-Item -LiteralPath (Join-Path (Join-Path $script:Config.devRoot ('maps\'+$MapId)) $change.source) -Destination ($target+'.devlab-new') -Force
    Move-Item -LiteralPath ($target+'.devlab-new') -Destination $target -Force
   }
  }
  if($swapping){Move-Item -LiteralPath $save -Destination (Join-Path $backup 'outgoing-live-save');Copy-Item -LiteralPath $incoming -Destination $save -Recurse}
  if($next.enableVisibilityMod){[IO.File]::WriteAllText($flag,'Native skidpad visibility options')}
  elseif(Test-Path -LiteralPath $flag){Move-Item -LiteralPath $flag -Destination (Join-Path $backup 'retired-visibility.flag')}
  foreach($entry in $runtimeCopies){
   Copy-Item -LiteralPath $entry.Source -Destination $entry.Target -Force
   if((Get-Fh2Hash $entry.Target).Hash -ne $entry.Hash){throw 'Installed runtime checksum mismatch.'}
  }
  Assert-Fh2MapFiles $next
  [ordered]@{id=$MapId;title=$next.title;saveSlot=$next.saveSlot;activated=(Get-Date).ToString('o');rollback=$backup;status='experimental'} | ConvertTo-Json | Set-Content -LiteralPath $activeFile -Encoding UTF8
 }catch{
  foreach($entry in $runtimeCopies){if($entry.Backup){Copy-Item -LiteralPath $entry.Backup -Destination $entry.Target -Force}elseif(Test-Path -LiteralPath $entry.Target){Move-Item -LiteralPath $entry.Target -Destination (Join-Path $backup ('failed-'+(Split-Path -Leaf $entry.Target)))}}
  foreach($change in $current.changes){
   $target=Join-Path $game $change.target
   if($change.absent){
    if(Test-Path -LiteralPath $target){$retired=Join-Path (Join-Path $backup 'failed-artifacts') $change.target;New-Item -ItemType Directory -Path (Split-Path $retired) -Force | Out-Null;Move-Item -LiteralPath $target -Destination $retired -Force}
   }else{Copy-Item -LiteralPath (Join-Path $backup $change.target) -Destination $target -Force}
  }
  if($swapping -and (Test-Path -LiteralPath (Join-Path $backup 'outgoing-live-save'))){
   if(Test-Path -LiteralPath $save){Move-Item -LiteralPath $save -Destination (Join-Path $backup 'failed-incoming-save')}
   Copy-Item -LiteralPath (Join-Path $backup 'player-1') -Destination $save -Recurse
  }
  if($hadFlag){Copy-Item -LiteralPath (Join-Path $backup 'skidpad-enabled.flag') -Destination $flag -Force}
  elseif(Test-Path -LiteralPath $flag){Move-Item -LiteralPath $flag -Destination (Join-Path $backup 'failed-visibility.flag') -Force}
  Copy-Item -LiteralPath (Join-Path $backup 'active-map.json') -Destination $activeFile -Force
  throw
 }
 return ('DevLab map activated: '+$next.title+'. Separate save selected; previous files backed up.')
}

# FHCM_LIBRARY_SPAWN_V1

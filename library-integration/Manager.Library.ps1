# FHCM portable package registry adapter. Only native-validated immutable installs.
function Get-FhcmLibraryMap([string]$MapId){
 if($MapId -notmatch '^fhcm-[a-z0-9-]+$'){throw 'Invalid imported map ID.'}
 $registry=Get-Content -LiteralPath (Join-Path $script:Config.devRoot 'fhcm-library/registry.json') -Raw | ConvertFrom-Json
 $entries=@($registry.maps)+@($registry.history)
 $entry=$entries | Where-Object {$_.packageId -ceq $MapId -and $_.imported} | Select-Object -First 1
 if(-not $entry){throw 'Imported map is not registered.'}
 $root=Join-Path $script:Config.devRoot ('maps\'+$MapId)
 $manifest=Join-Path $root 'map.json'
 if((Get-Fh2Hash -LiteralPath $manifest).Hash -ne $entry.manifestHash){throw 'Imported activation manifest changed.'}
 $map=Get-Content -LiteralPath $manifest -Raw | ConvertFrom-Json
 if($map.id -cne $MapId -or $map.saveSlot -cne $entry.slot){throw 'Imported identity mismatch.'}
 foreach($change in $map.changes){
  $source=Join-Path $root $change.source
  Assert-Fh2DevPaths @($source)
  if((Get-Fh2Hash -LiteralPath $source).Hash -ne $change.sha256){throw 'Imported map payload changed.'}
 }
 foreach($item in $map.seattleRuntimeSource.PSObject.Properties){
  Assert-Fh2DevPaths @($item.Value)
  if((Get-Fh2Hash -LiteralPath $item.Value).Hash -ne $map.seattleRuntime.($item.Name)){throw 'Imported runtime changed.'}
 }
 return $map
}

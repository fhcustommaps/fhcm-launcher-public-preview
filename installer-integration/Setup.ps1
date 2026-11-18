param([Parameter(Mandatory)][string]$Request)
$ErrorActionPreference='Stop'
$requestData=Get-Content -LiteralPath $Request -Raw|ConvertFrom-Json
$dest=[IO.Path]::GetFullPath($requestData.destination).TrimEnd('\')
$assets=Split-Path -Parent $Request
function Report($message){[IO.File]::WriteAllText((Join-Path $assets 'progress.txt'),$message)}
function CheckSource($path){
 $full=[IO.Path]::GetFullPath($path).TrimEnd('\')
 if(-not(Test-Path -LiteralPath $full -PathType Container)){throw "Missing source folder: $full"}
 if($dest.StartsWith($full+'\',[StringComparison]::OrdinalIgnoreCase) -or $full.StartsWith($dest+'\',[StringComparison]::OrdinalIgnoreCase) -or $full -eq $dest){throw 'Installation and source folders must be separate.'}
 if((Get-Item -LiteralPath $full).Attributes -band [IO.FileAttributes]::ReparsePoint){throw 'Select a real source directory, not a link.'}
 $links=@(Get-ChildItem -LiteralPath $full -Recurse -Force -Attributes ReparsePoint)
 if($links.Count){throw "Source contains links; use a complete local installation: $full"}
 return $full
}
if(Get-Process azimuth -ErrorAction SilentlyContinue){throw 'Close FH2 before preparing a separate installation.'}
if(Test-Path -LiteralPath $dest){throw 'Choose a new, empty destination name. Existing folders are never overwritten.'}
$game=CheckSource $requestData.game
$runtime=CheckSource $requestData.runtime
$content=$null;if($requestData.content){$content=CheckSource $requestData.content}
if(-not(Test-Path -LiteralPath (Join-Path $game 'Mount/forza_durango_release_final.exe'))){throw 'Select the FH2 folder containing Mount/forza_durango_release_final.exe.'}
if(-not(Test-Path -LiteralPath (Join-Path $runtime 'azimuth.exe'))){throw 'Select the Azimuth folder containing azimuth.exe.'}
$saveRelative='data/titles/1A96545B/saves/player-1'
if(-not(Test-Path -LiteralPath (Join-Path $runtime $saveRelative))){throw 'Launch FH2 once in Azimuth to create a save, then run setup again.'}
$total=0L
foreach($source in @($game,$runtime,$content)){if($source){$total+=(Get-ChildItem -LiteralPath $source -Recurse -File|Measure-Object Length -Sum).Sum}}
$parent=Split-Path -Parent $dest
while(-not(Test-Path -LiteralPath $parent)){$parent=Split-Path -Parent $parent}
if((Get-Item -LiteralPath $parent).Attributes -band [IO.FileAttributes]::ReparsePoint){throw 'Destination parent must not be a link.'}
$drive=[IO.DriveInfo]::new([IO.Path]::GetPathRoot($dest))
if($drive.AvailableFreeSpace -lt $total+2GB){throw ('Not enough free space. Allow at least '+[Math]::Ceiling(($total+2GB)/1GB)+' GiB for the independent copy.')}
New-Item -ItemType Directory -Path $dest -ErrorAction Stop|Out-Null
function CopyTree($from,$to){
 & robocopy.exe $from $to /E /COPY:DAT /DCOPY:DAT /R:1 /W:1 /XJ /NFL /NDL /NJH /NJS /NP /XD logs .git /XF '*.dmp' |Out-Null
 if($LASTEXITCODE -ge 8){throw "Copy failed ($LASTEXITCODE): $from. Partial files were retained at $dest."}
}
Report 'Copying your game into the independent FHCM library…'
CopyTree $game (Join-Path $dest 'game')
Report 'Copying Azimuth and your save…'
CopyTree $runtime (Join-Path $dest 'runtime')
New-Item -ItemType Directory -Path (Join-Path $dest 'content/StormIsland') -Force|Out-Null
if($content){Report 'Copying Storm Island…';CopyTree $content (Join-Path $dest 'content/StormIsland')}
Report 'Preparing the independent runtime and map manager…'
$runtimeCopy=Join-Path $dest 'runtime'
$gameCopy=Join-Path $dest 'game'
$contentCopy=if($content){Join-Path $dest 'content/StormIsland'}else{''}
# Retire copied native mods rather than run source-folder-specific hooks.
$nativeMods=Join-Path $runtimeCopy 'fh2-mods'
if(Test-Path $nativeMods){Move-Item -LiteralPath $nativeMods -Destination (Join-Path $dest 'source-mods-retained')}
New-Item -ItemType Directory -Path $nativeMods -Force|Out-Null
Copy-Item -LiteralPath (Join-Path $assets 'DevLabVisibility.dll'),(Join-Path $assets 'DevLabKeyboard.dll') -Destination $nativeMods
Copy-Item -LiteralPath (Join-Path $assets 'd3dcompiler_47.dll') -Destination $runtimeCopy -Force
# All edits below apply only to the newly copied files.
foreach($file in Get-ChildItem -LiteralPath $runtimeCopy -Recurse -File -Include '*.ini','*.json','*.cfg'){
 if($file.Length -gt 4MB){continue}
 $text=[IO.File]::ReadAllText($file.FullName)
 $next=$text.Replace($runtime,$runtimeCopy).Replace($game,$gameCopy)
 if($content){$next=$next.Replace($content,$contentCopy)}
 if($next -cne $text){[IO.File]::WriteAllText($file.FullName,$next,[Text.UTF8Encoding]::new($false))}
}
$ini=Join-Path $runtimeCopy 'data/azimuth.ini'
$text=[IO.File]::ReadAllText($ini)
$text=[regex]::Replace($text,'(?m)^game_folder=.*$',('game_folder='+$gameCopy))
$text=[regex]::Replace($text,'(?m)^folder=.*$',('folder='+$gameCopy))
$text=[regex]::Replace($text,'(?m)^content=.*$',('content='+$contentCopy))
if($text -notmatch '(?m)^content='){$text+="`r`n[title-1a96545b]`r`nfolder=$gameCopy`r`ncontent=$contentCopy`r`n"}
[IO.File]::WriteAllText($ini,$text,[Text.UTF8Encoding]::new($false))
# The legacy FH2Patch TLS shim has one fixed absolute LoadLibraryA literal.
# Relocate only that literal in the copy, with a terminated shorter path.
$shim=Join-Path $runtimeCopy 'data/bin/azimuth-tls.dll.original'
if(Test-Path $shim){
 $bytes=[IO.File]::ReadAllBytes($shim);$ascii=[Text.Encoding]::ASCII.GetString($bytes)
 foreach($hit in [regex]::Matches($ascii,'[A-Za-z]:\\[^\x00\r\n]+?\\FH2Patch\.dll')){
  $replacement=[Text.Encoding]::ASCII.GetBytes('data\bin\FH2Patch.dll')
  if($replacement.Length -gt $hit.Length){throw 'TLS path relocation capacity exceeded.'}
  [Array]::Clear($bytes,$hit.Index,$hit.Length);[Array]::Copy($replacement,0,$bytes,$hit.Index,$replacement.Length)
 }
 [IO.File]::WriteAllBytes($shim,$bytes)
}
$manager=Join-Path $dest 'manager';New-Item -ItemType Directory -Path $manager -Force|Out-Null
Copy-Item -LiteralPath (Join-Path $assets 'Manager.Core.ps1'),(Join-Path $assets 'Manager.Maps.ps1'),(Join-Path $assets 'Manager.Library.ps1') -Destination $manager
$cfg=[ordered]@{schema=1;devRoot=$dest;profiles=@{
 play=@{label='Original FH2';runtime=$runtime;game=$game;content=$content;save=(Join-Path $runtime $saveRelative)}
 dev=@{label='FHCM custom maps';runtime=$runtimeCopy;game=$gameCopy;content=$contentCopy;save=(Join-Path $runtimeCopy $saveRelative)}
}}
$cfg|ConvertTo-Json -Depth 6|Set-Content -LiteralPath (Join-Path $manager 'profiles.json') -Encoding UTF8
. (Join-Path $manager 'Manager.Core.ps1')
$baseline=Join-Path $dest 'maps/stock'
$changes=@()
foreach($target in $script:Fh2MapTargets){
 $source=Join-Path $gameCopy $target
 if(Test-Path -LiteralPath $source){
  $copy=Join-Path $baseline ('files/'+$target);New-Item -ItemType Directory -Path (Split-Path $copy) -Force|Out-Null;Copy-Item -LiteralPath $source -Destination $copy
  $changes+=@{target=$target;source=('files/'+$target);sha256=(Get-Fh2Hash $copy).Hash.ToLowerInvariant()}
 }else{$changes+=@{target=$target;absent=$true}}
}
@{schema=2;id='stock';title='Original copied baseline';status='ready';saveSlot='baseline';enableVisibilityMod=$false;changes=$changes}|ConvertTo-Json -Depth 6|Set-Content -LiteralPath (Join-Path $baseline 'map.json') -Encoding UTF8
@{id='stock';saveSlot='baseline'}|ConvertTo-Json|Set-Content -LiteralPath (Join-Path $dest 'active-map.json') -Encoding UTF8
New-Item -ItemType Directory -Path (Join-Path $dest 'fhcm-library') -Force|Out-Null
@{schema=1;maps=@();history=@()}|ConvertTo-Json|Set-Content -LiteralPath (Join-Path $dest 'fhcm-library/registry.json') -Encoding UTF8
[IO.File]::WriteAllText((Join-Path $dest 'keyboard-bridge.flag'),'FHCM focused keyboard controls')
Assert-Fh2ActiveMap
Test-Fh2Profile 'dev'|Out-Null
Report 'Setup complete. Import an FHCM map ZIP to get started.'

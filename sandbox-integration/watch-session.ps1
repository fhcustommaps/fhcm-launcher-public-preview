param([Parameter(Mandatory)][string]$DevRoot,[string]$SessionId,[switch]$RecoverOnly)
$ErrorActionPreference='Stop'
function Get-SandboxHash([string]$Path){
    $stream=[IO.File]::OpenRead($Path);$sha=[Security.Cryptography.SHA256]::Create()
    try{return [BitConverter]::ToString($sha.ComputeHash($stream)).Replace('-','').ToLowerInvariant()}finally{$sha.Dispose();$stream.Dispose()}
}
function Get-PersonalFingerprint([string]$Save){
    $users=@(Get-ChildItem -LiteralPath $Save -Directory -Filter 'User_*');if($users.Count -ne 1){throw 'Invalid personal profile count.'}
    $revision=Get-ChildItem -LiteralPath (Join-Path $users[0].FullName 'revisions') -Directory | Sort-Object Name -Descending | Where-Object {Test-Path -LiteralPath (Join-Path $_.FullName 'commit')} | Select-Object -First 1
    $a=[IO.File]::ReadAllBytes((Join-Path $revision.FullName 'data/ProfileData'));$b=[IO.File]::ReadAllBytes((Join-Path $revision.FullName 'data/PlayerDatabase'))
    $combined=New-Object byte[] ($a.Length+$b.Length);[Array]::Copy($a,0,$combined,0,$a.Length);[Array]::Copy($b,0,$combined,$a.Length,$b.Length)
    $sha=[Security.Cryptography.SHA256]::Create();try{return [BitConverter]::ToString($sha.ComputeHash($combined)).Replace('-','').ToLowerInvariant()}finally{$sha.Dispose()}
}
$root=[IO.Path]::GetFullPath($DevRoot).TrimEnd('\')+'\'
$sandboxRoot=Join-Path $root 'fhcm-sandbox'
$leaseFile=Join-Path $sandboxRoot 'active-session.json'
if(-not(Test-Path -LiteralPath $leaseFile)){exit 0}
if(-not $RecoverOnly){
    $seen=$false
    for($i=0;$i -lt 1200;$i++){
        if(-not(Test-Path -LiteralPath $leaseFile)){exit 0}
        $current=Get-Content -LiteralPath $leaseFile -Raw | ConvertFrom-Json
        if($current.id -ne $SessionId){exit 0}
        $running=@(Get-Process azimuth -ErrorAction SilentlyContinue)
        if($running.Count){$seen=$true;foreach($p in $running){Wait-Process -Id $p.Id -ErrorAction SilentlyContinue};break}
        Start-Sleep -Milliseconds 100
    }
}
if(Get-Process azimuth -ErrorAction SilentlyContinue){exit 0}
$lock=$null
for($i=0;$i -lt 200 -and -not $lock;$i++){
    try{$lock=[IO.File]::Open((Join-Path $sandboxRoot 'restore.lock'),[IO.FileMode]::OpenOrCreate,[IO.FileAccess]::ReadWrite,[IO.FileShare]::None)}catch{Start-Sleep -Milliseconds 100}
}
if(-not $lock){throw 'Another sandbox recovery is still running.'}
try{
    if(-not(Test-Path -LiteralPath $leaseFile)){exit 0}
    if(Get-Process azimuth -ErrorAction SilentlyContinue){exit 0}
    $lease=Get-Content -LiteralPath $leaseFile -Raw | ConvertFrom-Json
    if($SessionId -and $lease.id -ne $SessionId){exit 0}
    if($lease.map -notmatch '^[a-z0-9][a-z0-9-]{0,39}$' -or $lease.id -notmatch '^\d+$'){throw 'Invalid sandbox journal.'}
    foreach($name in @('live','personal','session','sandbox')){
        $absolute=[IO.Path]::GetFullPath($lease.$name)
        if(-not $absolute.StartsWith($root,[StringComparison]::OrdinalIgnoreCase)){throw 'Sandbox journal path leaves DevLab.'}
        if((Test-Path -LiteralPath $absolute) -and ((Get-Item -LiteralPath $absolute).Attributes -band [IO.FileAttributes]::ReparsePoint)){throw 'Linked sandbox paths are unsupported.'}
    }
    $expected=Join-Path (Join-Path $sandboxRoot 'sessions') $lease.id
    if([IO.Path]::GetFullPath($lease.session) -ne [IO.Path]::GetFullPath($expected)){throw 'Invalid sandbox session path.'}
    if([IO.Path]::GetFullPath($lease.personal) -ne [IO.Path]::GetFullPath((Join-Path $expected 'personal-player-1'))){throw 'Invalid preserved personal-save path.'}
    if([IO.Path]::GetFullPath($lease.sandbox) -ne [IO.Path]::GetFullPath((Join-Path (Join-Path (Join-Path $sandboxRoot 'saves') $lease.map) 'player-1'))){throw 'Invalid sandbox archive path.'}
    $archived=$false;$validationError=''
    if(Test-Path -LiteralPath $lease.personal){
        if(Test-Path -LiteralPath $lease.live){
            $outgoing=Join-Path $expected 'outgoing-sandbox-player-1'
            if(Test-Path -LiteralPath $outgoing){$outgoing=Join-Path $expected ('outgoing-sandbox-'+[guid]::NewGuid().ToString())}
            Move-Item -LiteralPath $lease.live -Destination $outgoing
            if($lease.phase -eq 'active'){
                $valid=$true
                try{
                    $users=@(Get-ChildItem -LiteralPath $outgoing -Directory -Filter 'User_*');if($users.Count -ne 1){throw 'Invalid profile count'}
                    $commit=Get-ChildItem -LiteralPath (Join-Path $users[0].FullName 'revisions') -Directory | Sort-Object Name -Descending | Where-Object {Test-Path -LiteralPath (Join-Path $_.FullName 'commit')} | Select-Object -First 1
                    $lines=Get-Content -LiteralPath (Join-Path $commit.FullName 'commit')
                    if($lines[0] -ne 'Azimuth complete save revision v2'){throw 'Invalid manifest'}
                    foreach($line in $lines | Select-Object -Skip 2){
                        if($line -notmatch '^([0-9a-f]{64}) (\d+) ([A-Za-z0-9_]+)$'){throw 'Invalid blob entry'}
                        $expectedHash=$Matches[1];$length=[long]$Matches[2];$file=Join-Path (Join-Path $commit.FullName 'data') $Matches[3]
                        if((Get-Item -LiteralPath $file).Length -ne $length -or (Get-SandboxHash $file) -ne $expectedHash){throw 'Sandbox save failed validation'}
                    }
                }catch{$valid=$false;$validationError=$_.Exception.Message}
                if($valid){
                    if(Test-Path -LiteralPath $lease.sandbox){Move-Item -LiteralPath $lease.sandbox -Destination (Join-Path $expected 'previous-sandbox-player-1')}
                    Move-Item -LiteralPath $outgoing -Destination $lease.sandbox
                    $archived=$true
                }
            }
        }
        Move-Item -LiteralPath $lease.personal -Destination $lease.live
    }
    if(-not(Test-Path -LiteralPath $lease.live)){throw 'Personal save could not be restored; sandbox journal retained.'}
    if((Get-PersonalFingerprint $lease.live) -ne $lease.personalFingerprint){throw 'Restored save does not match the preserved personal profile; journal retained.'}
    [ordered]@{restoredAt=(Get-Date).ToString('o');id=$lease.id;map=$lease.map;personalRestored=$true;sandboxArchived=$archived;validationError=$validationError} | ConvertTo-Json | Set-Content -LiteralPath (Join-Path $expected 'restoration.json') -Encoding UTF8
    # Concurrent status reads may briefly hold the journal on Windows.
    for($attempt=0;$attempt -lt 20;$attempt++){
        try{Remove-Item -LiteralPath $leaseFile -ErrorAction Stop;break}
        catch{if($attempt -eq 19){throw};Start-Sleep -Milliseconds 100}
    }
}finally{$lock.Dispose()}

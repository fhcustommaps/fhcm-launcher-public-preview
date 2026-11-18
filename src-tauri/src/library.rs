use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    collections::HashSet,
    fs,
    io::{Read, Write},
    path::{Path, PathBuf},
};
type Result<T> = std::result::Result<T, String>;
fn hash_file(p: &Path) -> Result<String> {
    let mut f = fs::File::open(p).map_err(|e| e.to_string())?;
    let mut hash = Sha256::new();
    let mut b = [0u8; 131072];
    loop {
        let n = f.read(&mut b).map_err(|e| e.to_string())?;
        if n == 0 {
            break;
        }
        hash.update(&b[..n]);
    }
    Ok(format!("{:x}", hash.finalize()))
}
fn hash(b: &[u8]) -> String {
    format!("{:x}", Sha256::digest(b))
}
fn module_header(bytes: &[u8]) -> Result<()> {
    if bytes.len() < 256 || &bytes[..2] != b"MZ" {
        return Err("Map module is not a Windows DLL".into());
    }
    let at = u32::from_le_bytes(bytes[60..64].try_into().unwrap()) as usize;
    if at + 24 > bytes.len()
        || &bytes[at..at + 4] != b"PE\0\0"
        || u16::from_le_bytes(bytes[at + 4..at + 6].try_into().unwrap()) != 0x8664
        || u16::from_le_bytes(bytes[at + 22..at + 24].try_into().unwrap()) & 0x2000 == 0
    {
        return Err("Map modules must be valid x64 DLLs".into());
    }
    Ok(())
}
pub fn dev() -> Result<PathBuf> {
    Ok(PathBuf::from(
        super::read_json(super::manager_dir()?.join("profiles.json"))?["devRoot"]
            .as_str()
            .ok_or("Missing DevLab")?,
    ))
}
fn state_path() -> Result<PathBuf> {
    Ok(dev()?.join("fhcm-library/registry.json"))
}
fn save_state(v: &Value) -> Result<()> {
    let p = state_path()?;
    fs::create_dir_all(p.parent().unwrap()).map_err(|e| e.to_string())?;
    let stage = p.with_extension("new");
    fs::write(&stage, serde_json::to_vec_pretty(v).unwrap()).map_err(|e| e.to_string())?;
    if p.exists() {
        fs::copy(&p, p.with_extension("previous.json")).map_err(|e| e.to_string())?;
    }
    fs::rename(stage, p).map_err(|e| e.to_string())
}
pub fn state() -> Result<Value> {
    let p = state_path()?;
    if p.exists() {
        return super::read_json(p);
    }
    let labels: Value = serde_json::from_str(include_str!("../../map-labels.json")).unwrap();
    let entries:Vec<Value>=super::MAPS.iter().enumerate().map(|(i,(alias,id))|json!({"id":alias,"packageId":id,"slot":id,"enabled":true,"favorite":false,"order":i,"imported":false,"name":labels[*alias]["name"],"region":labels[*alias]["region"],"img":labels[*alias]["img"],"artworkPlaceholder":true,"version":"Local","conditionState":if *alias=="pacific"{"pch_dry"}else{"gunsai_dry_daylight"}})).collect();
    let v = json!({"schema":1,"maps":entries});
    save_state(&v)?;
    Ok(v)
}
pub fn resolve(alias: &str) -> Result<Value> {
    state()?["maps"]
        .as_array()
        .ok_or("Invalid library registry")?
        .iter()
        .find(|v| v["id"] == alias)
        .cloned()
        .ok_or("Map is not in the local library".into())
}
pub fn package_root(entry: &Value) -> Result<PathBuf> {
    let id = entry["packageId"].as_str().ok_or("Missing package ID")?;
    slug(id)?;
    Ok(dev()?.join("maps").join(id))
}
pub fn available(alias: &str) -> Result<Value> {
    let e = resolve(alias)?;
    if e["enabled"] != true {
        return Err(
            "Map was removed from the installed library. Restore it before driving.".into(),
        );
    }
    Ok(e)
}
pub fn pch_racer_profiles(entry: &Value) -> bool {
    if entry["id"] != "pacific-estate36-racers" {
        return false;
    }
    package_root(entry).ok().and_then(|root| super::read_json(root.join("fhcm-map.json")).ok())
        .is_some_and(|descriptor| descriptor["racerAdapter"] == "pch-racers-v1")
}
pub fn catalog() -> Result<Value> {
    let mut entries = state()?["maps"]
        .as_array()
        .ok_or("Invalid library")?
        .clone();
    entries.sort_by_key(|v| v["order"].as_i64().unwrap_or(0));
    let mut installed = vec![];
    let mut removed = vec![];
    for e in entries {
        let root = package_root(&e)?;
        if !root.join("map.json").is_file() {
            continue;
        }
        let traffic = super::traffic::supported(&e);
        let racers = pch_racer_profiles(&e);
        let blurb = if traffic {"Time, weather, HUD and traffic apply at launch."}else{"Time, weather and HUD apply at launch. Traffic and AI use the map configuration."};
        let map = json!({"id":e["id"],"original":false,"name":e["name"],"region":e["region"],"img":e["img"],"artworkPlaceholder":e["artworkPlaceholder"],"favorite":e["favorite"],"imported":e["imported"],"blurb":blurb,"type":"Custom map","format":"Local package","length":"—","size":"—","status":"installed","version":e["version"],"latest":e["version"],"progress":0,"tone":"#3DB8FF","compat":{"traffic":traffic,"exp":racers,"pchRacers":racers,"conditions":true,"weather":["clear","wet","rain","storm"]}});
        if e["enabled"] == true {
            installed.push(map)
        } else {
            removed.push(map)
        }
    }
    Ok(json!({"maps":installed,"removed":removed}))
}
pub fn update(alias: &str, action: &str, other: Option<&str>) -> Result<Value> {
    let mut v = state()?;
    let entries = v["maps"].as_array_mut().ok_or("Invalid library")?;
    let i = entries
        .iter()
        .position(|e| e["id"] == alias)
        .ok_or("Unknown library map")?;
    match action {
        "remove" => entries[i]["enabled"] = json!(false),
        "restore" => entries[i]["enabled"] = json!(true),
        "favorite" => entries[i]["favorite"] = json!(entries[i]["favorite"] != true),
        "up" | "down" | "before" => {
            entries.sort_by_key(|e| e["order"].as_i64().unwrap_or(0));
            let i = entries.iter().position(|e| e["id"] == alias).unwrap();
            let j = if action == "before" {
                entries
                    .iter()
                    .position(|e| e["id"] == other.unwrap_or(""))
                    .ok_or("Unknown reorder target")?
            } else if action == "up" {
                i.saturating_sub(1)
            } else {
                (i + 1).min(entries.len() - 1)
            };
            let item = entries.remove(i);
            let destination = if action == "before" && i < j {
                j - 1
            } else {
                j
            };
            entries.insert(destination, item);
            for (i, e) in entries.iter_mut().enumerate() {
                e["order"] = json!(i);
            }
        }
        _ => return Err("Unknown library action".into()),
    }
    save_state(&v)?;
    catalog()
}
fn slug(s: &str) -> Result<()> {
    if s.is_empty()
        || s.len() > 72
        || !s
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
    {
        return Err("Invalid map ID".into());
    }
    Ok(())
}
fn relative(s: &str) -> Result<()> {
    if s.is_empty()
        || s.len() > 230
        || s.contains(['\\', ':', '\0'])
        || s.starts_with('/')
        || s.split('/').any(|c| {
            c.is_empty()
                || c == "."
                || c == ".."
                || c.ends_with(['.', ' '])
                || c.chars().any(|x| x < ' ' || "<>\"|?*".contains(x))
                || matches!(
                    c.split('.')
                        .next()
                        .unwrap_or("")
                        .to_ascii_uppercase()
                        .as_str(),
                    "CON"
                        | "PRN"
                        | "AUX"
                        | "NUL"
                        | "COM1"
                        | "COM2"
                        | "COM3"
                        | "COM4"
                        | "COM5"
                        | "COM6"
                        | "COM7"
                        | "COM8"
                        | "COM9"
                        | "LPT1"
                        | "LPT2"
                        | "LPT3"
                        | "LPT4"
                        | "LPT5"
                        | "LPT6"
                        | "LPT7"
                        | "LPT8"
                        | "LPT9"
                )
        })
    {
        return Err(format!("Unsafe package path: {s}"));
    }
    Ok(())
}
fn target(kind: &str, s: &str) -> Result<()> {
    relative(s)?;
    match kind {
        "game" if s.starts_with("Mount/media/") || s.starts_with("Mount/mediaoverride/") => {
            if s.to_ascii_lowercase().ends_with(".dll") || s.to_ascii_lowercase().ends_with(".exe")
            {
                return Err("Executable in game payload".into());
            }
        }
        "module"
            if [
                "SeattleTrafficProbe.dll",
                "SeattleRacerProbe.dll",
                "SeattleOvalCollision.dll",
                "SeattleSceneTrace.dll",
            ]
            .contains(&s) => {}
        "data" if s.starts_with("fhcm-data/") && s.ends_with(".bin") => {}
        "artwork" if s == "cover.png" => {}
        "provenance" if s == "map.json" => {}
        _ => return Err(format!("Unsupported package target: {kind}/{s}")),
    }
    Ok(())
}
fn descriptor(v: &Value) -> Result<()> {
    if v["format"] != "fhcm-map" || v["schema"] != 1 || v["adapter"] != "homespace02-v1" {
        return Err("This ZIP needs an FHCM V1 map package (fhcm-map.json). Export a working map from the launcher to create one.".into());
    }
    slug(v["id"].as_str().ok_or("Missing map ID")?)?;
    if v["id"].as_str().unwrap().len() > 40 {
        return Err("Map ID exceeds 40 characters".into());
    }
    for k in ["name", "region", "version", "conditionState"] {
        let s = v[k].as_str().ok_or(format!("Missing {k}"))?;
        if s.is_empty() || s.len() > 200 {
            return Err(format!("Invalid {k}"));
        }
    }
    let condition = v["conditionState"].as_str().unwrap();
    if !condition
        .bytes()
        .all(|b| b.is_ascii_alphanumeric() || b == b'_')
    {
        return Err("Invalid map condition state".into());
    }
    for key in ["spawnPosition", "spawnFacing"] {
        let a: Vec<_> = v[key].as_str().ok_or("Missing spawn")?.split(',').collect();
        if a.len() != 3
            || a.iter().any(|x| {
                x.parse::<f64>()
                    .map(|n| !n.is_finite() || n.abs() > 1e7)
                    .unwrap_or(true)
            })
        {
            return Err("Invalid map spawn".into());
        }
        if !v[key]
            .as_str()
            .unwrap()
            .bytes()
            .all(|b| b.is_ascii_digit() || b == b'-' || b == b'.' || b == b',')
        {
            return Err("Spawn coordinates must use plain decimal notation".into());
        }
        if key == "spawnPosition" {
            let coordinates: Vec<f64> = a.iter().map(|n| n.parse().unwrap()).collect();
            if coordinates[0].abs() > 50000.0
                || coordinates[2].abs() > 50000.0
                || coordinates[1] < -100.0
                || coordinates[1] > 1500.0
            {
                return Err("Map spawn is outside the supported homespace02-v1 extent".into());
            }
        }
    }
    let files = v["files"].as_array().ok_or("Missing file manifest")?;
    if files.is_empty() || files.len() > 50000 {
        return Err("Invalid package file count".into());
    }
    let mut seen = HashSet::new();
    let mut targets = HashSet::new();
    let mut total = 0u64;
    for f in files {
        let p = f["path"].as_str().ok_or("Missing payload path")?;
        relative(p)?;
        if !seen.insert(p.to_ascii_lowercase()) {
            return Err("Duplicate package path".into());
        }
        let k = f["kind"].as_str().ok_or("Missing target kind")?;
        let t = f["target"].as_str().ok_or("Missing target")?;
        target(k, t)?;
        if !targets.insert(format!("{k}/{}", t.to_ascii_lowercase())) {
            return Err("Duplicate installation target".into());
        }
        let h = f["sha256"].as_str().ok_or("Missing checksum")?;
        if h.len() != 64 || !h.bytes().all(|c| c.is_ascii_hexdigit()) {
            return Err("Invalid file checksum".into());
        }
        let n = f["size"].as_u64().ok_or("Missing file size")?;
        if n > 8 * 1024 * 1024 * 1024 {
            return Err("Package file exceeds 8 GiB".into());
        }
        total = total.checked_add(n).ok_or("Package size overflow")?;
        if total > 40 * 1024 * 1024 * 1024 {
            return Err("Package exceeds 40 GiB".into());
        }
    }
    for required in [
        "Mount/media/tracks/homespace02/GeoChunk0.minizip",
        "Mount/media/tracks/homespace02/GeoChunk1.minizip",
        "Mount/media/tracks/homespace02/GeoChunk2.minizip",
        "Mount/media/tracks/homespace02/GeoChunk3.minizip",
        "Mount/mediaoverride/stripped/gamedbRC.slt",
        "Mount/mediaoverride/StateFlow/boot.xml",
        "Mount/mediaoverride/StateFlow/free_roam_start.xml",
        "Mount/media/tracks/homespace02/ribbon_00/HomeSpace02_00.pvs",
        "Mount/media/tracks/homespace02/bin.zip",
        "Mount/media/tracks/homespace02/ChunkMap0.dat",
        "Mount/media/tracks/homespace02/ChunkMap1.dat",
        "Mount/media/tracks/homespace02/ChunkMap2.dat",
        "Mount/media/tracks/homespace02/ChunkMap3.dat",
        "Mount/media/zipmanifest.xml",
        "Mount/mediaoverride/zipmanifest.xml",
    ] {
        if !targets.contains(&format!("game/{}", required.to_ascii_lowercase())) {
            return Err(format!("Incomplete map package: missing {required}"));
        }
    }
    for m in [
        "SeattleTrafficProbe.dll",
        "SeattleRacerProbe.dll",
        "SeattleOvalCollision.dll",
        "SeattleSceneTrace.dll",
    ] {
        if !targets.contains(&format!("module/{}", m.to_ascii_lowercase())) {
            return Err(format!("Missing map runtime {m}"));
        }
    }
    Ok(())
}
pub fn install_manager() -> Result<()> {
    let dir = super::manager_dir()?;
    let maps = dir.join("Manager.Maps.ps1");
    let core = dir.join("Manager.Core.ps1");
    let mut m = fs::read_to_string(&maps).map_err(|e| e.to_string())?;
    if !m.contains("# FHCM_LIBRARY_V1") {
        let anchor = "function Get-Fh2Map([string]$MapId) {";
        if !m.contains(anchor) {
            return Err("Unsupported map manager layout".into());
        }
        let backup = dev()?.join("fhcm-library/manager-backup");
        fs::create_dir_all(&backup).map_err(|e| e.to_string())?;
        fs::copy(&maps, backup.join("Manager.Maps.ps1")).map_err(|e| e.to_string())?;
        fs::copy(&core, backup.join("Manager.Core.ps1")).map_err(|e| e.to_string())?;
        m=m.replace(anchor,"# FHCM_LIBRARY_V1\n. (Join-Path $PSScriptRoot 'Manager.Library.ps1')\nfunction Get-Fh2Map([string]$MapId) {\n if($MapId -like 'fhcm-*'){return Get-FhcmLibraryMap $MapId}");
        // The package registry validates imported IDs and immutable manifests first.
        m = m.replace(
            "if($map.id -in @('rogue-seattle'",
            "if($map.id -like 'fhcm-*' -or $map.id -in @('rogue-seattle'",
        );
        m = m.replace(
            "if($next.id -in @('rogue-seattle'",
            "if($next.id -like 'fhcm-*' -or $next.id -in @('rogue-seattle'",
        );
        m = m.replace(
            "if($activeMap.id -in @('rogue-seattle'",
            "if($activeMap.id -like 'fhcm-*' -or $activeMap.id -in @('rogue-seattle'",
        );
        m=m.replace("if(-not ($source.StartsWith(","if(-not ($source.StartsWith((Join-Path $script:Config.devRoot ('maps\\'+$next.id+'\\runtime\\')),[StringComparison]::OrdinalIgnoreCase) -or $source.StartsWith(");
        m=m.replace("if(-not ($entry.Name.StartsWith(","if(-not ($entry.Name.StartsWith((Join-Path $script:Config.profiles.dev.runtime 'fhcm-data\\'),[StringComparison]::OrdinalIgnoreCase) -or $entry.Name.StartsWith(");
        m = m.replace(
            "$conditionsFile=Join-Path $script:Config.devRoot 'fhcm-conditions\\active.json'",
            "$conditionsFile=Join-Path $script:Config.devRoot 'fhcm-conditions\\active.json'",
        );
        m = m.replace(
            "if($Map.id -in @('rogue-pch','rogue-gunsai') -and",
            "if(($Map.id -like 'fhcm-*' -or $Map.id -in @('rogue-pch','rogue-gunsai')) -and",
        );
        let mut c = fs::read_to_string(&core).map_err(|e| e.to_string())?;
        c=c.replace("if($Name -eq 'dev' -and $launchMap.id -in @('rogue-pch'","if($Name -eq 'dev' -and ($launchMap.id -like 'fhcm-*' -or $launchMap.id -in @('rogue-pch'");
        c = c.replace(
            "'rogue-gunsai','fm4-fujimi')) {",
            "'rogue-gunsai','fm4-fujimi'))) {",
        );
        c = c.replace(
            "if($launchMap.saveSlot -in @(",
            "if($launchMap.id -like 'fhcm-*' -or $launchMap.saveSlot -in @(",
        );
        let maps_stage = maps.with_extension("fhcm-stage.ps1");
        let core_stage = core.with_extension("fhcm-stage.ps1");
        fs::write(&maps_stage, m).map_err(|e| e.to_string())?;
        fs::write(&core_stage, c).map_err(|e| e.to_string())?;
        for path in [&maps_stage, &core_stage] {
            let quoted = path.to_string_lossy().replace('\'', "''");
            super::powershell(&format!("$errors=$null;$null=[System.Management.Automation.Language.Parser]::ParseFile('{}',[ref]$null,[ref]$errors);if($errors){{throw ($errors.Message -join '; ')}}",quoted))?;
        }
        fs::rename(maps_stage, &maps).map_err(|e| e.to_string())?;
        fs::rename(core_stage, &core).map_err(|e| e.to_string())?;
    }
    install_spawn_support(&maps)?;
    fs::write(
        dir.join("Manager.Library.ps1"),
        include_str!("../../library-integration/Manager.Library.ps1"),
    )
    .map_err(|e| e.to_string())
}
fn install_spawn_support(path: &Path) -> Result<()> {
    let source = fs::read_to_string(path).map_err(|e| e.to_string())?;
    if source.contains("# FHCM_LIBRARY_SPAWN_V1") {
        return Ok(());
    }
    let anchor = "elseif($activeMap.id -in @('rogue-pch'";
    if !source.contains(anchor) {
        return Err("Imported map spawn validation layout changed".into());
    }
    let next = source.replace(
        anchor,
        "elseif($activeMap.id -like 'fhcm-*' -or $activeMap.id -in @('rogue-pch'",
    ) + "\n# FHCM_LIBRARY_SPAWN_V1\n";
    let stage = path.with_extension("fhcm-spawn-stage.ps1");
    fs::write(&stage, next).map_err(|e| e.to_string())?;
    let quoted = stage.to_string_lossy().replace('\'', "''");
    super::powershell(&format!("$errors=$null;$null=[System.Management.Automation.Language.Parser]::ParseFile('{}',[ref]$null,[ref]$errors);if($errors){{throw ($errors.Message -join '; ')}}",quoted))?;
    fs::rename(stage, path).map_err(|e| e.to_string())
}
pub fn prepare(alias: &str) -> Result<()> {
    let e = available(alias)?;
    if e["imported"] != true {
        return Ok(());
    }
    install_manager()?;
    let root = package_root(&e)?;
    let desc = super::read_json(root.join("fhcm-map.json"))?;
    let required_visibility = visibility_requirement(&desc, &root.join("payload"))?;
    let manifest_path = root.join("map.json");
    let mut normalized = super::read_json(manifest_path.clone())?;
    if normalized["visibilityModuleSha256"] != required_visibility {
        normalized["visibilityModuleSha256"] = json!(required_visibility);
        let bytes = serde_json::to_vec_pretty(&normalized).unwrap();
        fs::write(&manifest_path, &bytes).map_err(|e| e.to_string())?;
        let mut registry = state()?;
        for entry in registry["maps"].as_array_mut().ok_or("Invalid library")? {
            if entry["packageId"] == e["packageId"] {
                entry["manifestHash"] = json!(hash(&bytes));
            }
        }
        save_state(&registry)?;
    }
    let cfg = super::read_json(super::manager_dir()?.join("profiles.json"))?;
    let runtime = PathBuf::from(
        cfg["profiles"]["dev"]["runtime"]
            .as_str()
            .ok_or("Missing runtime")?,
    );
    if hash_file(&runtime.join("fh2-mods/DevLabVisibility.dll"))? != required_visibility {
        return Err("This package requires a different shared visibility runtime. Update the FHCM runtime before driving.".into());
    }
    for f in desc["files"].as_array().ok_or("Missing payload")? {
        if f["kind"] == "data" {
            let src = root.join("payload").join(f["path"].as_str().unwrap());
            if hash_file(&src)? != f["sha256"].as_str().unwrap() {
                return Err("Stored map dependency changed".into());
            }
            let dest = runtime.join(f["target"].as_str().unwrap());
            if dest.exists() && hash_file(&dest)? == f["sha256"].as_str().unwrap() {
                continue;
            }
            fs::create_dir_all(dest.parent().unwrap()).map_err(|e| e.to_string())?;
            fs::copy(src, dest).map_err(|e| e.to_string())?;
        }
    }
    super::vehicles::seed_map_slot(e["slot"].as_str().ok_or("Missing save slot")?)?;
    Ok(())
}
fn visibility_requirement(desc: &Value, payload: &Path) -> Result<String> {
    let value = if let Some(s) = desc["visibilityModuleSha256"].as_str() {
        s.to_owned()
    } else {
        let provenance = desc["files"]
            .as_array()
            .ok_or("Missing package files")?
            .iter()
            .find(|f| f["kind"] == "provenance")
            .ok_or("Missing visibility runtime requirement")?;
        let path = payload.join(provenance["path"].as_str().ok_or("Missing provenance")?);
        if hash_file(&path)?
            != provenance["sha256"]
                .as_str()
                .ok_or("Missing provenance checksum")?
                .to_ascii_lowercase()
        {
            return Err("Package provenance changed".into());
        }
        super::read_json(path)?["visibilityModuleSha256"]
            .as_str()
            .ok_or("Missing visibility runtime requirement")?
            .to_owned()
    };
    if value.len() != 64 || !value.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err("Invalid visibility runtime checksum".into());
    }
    Ok(value.to_ascii_lowercase())
}
pub fn import_zip(path: &Path) -> Result<Value> {
    let zip_hash = hash_file(path)?;
    let file = fs::File::open(path).map_err(|e| e.to_string())?;
    let mut zip = zip::ZipArchive::new(file).map_err(|e| format!("Invalid ZIP: {e}"))?;
    if zip.len() > 50001 {
        return Err("ZIP has too many entries".into());
    }
    let mut names = HashSet::new();
    let mut manifests = vec![];
    for i in 0..zip.len() {
        let f = zip.by_index(i).map_err(|e| e.to_string())?;
        let name = f.name().trim_end_matches('/');
        relative(name)?;
        if f.unix_mode()
            .map(|m| m & 0o170000 == 0o120000)
            .unwrap_or(false)
        {
            return Err("Linked ZIP entries are unsupported".into());
        }
        if !names.insert(f.name().to_ascii_lowercase()) {
            return Err("Duplicate ZIP entries".into());
        }
        if name.rsplit('/').next() == Some("fhcm-map.json") {
            manifests.push(name.to_owned());
        }
    }
    if manifests.len() != 1 {
        return Err("ZIP must contain exactly one fhcm-map.json. Use Export ZIP on an existing working map to create an FHCM package.".into());
    }
    let manifest = &manifests[0];
    let prefix = manifest.strip_suffix("fhcm-map.json").unwrap();
    let mut text = String::new();
    {
        let mut f = zip.by_name(manifest).map_err(|e| e.to_string())?;
        if f.size() > 8 * 1024 * 1024 {
            return Err("Package manifest is too large".into());
        }
        f.read_to_string(&mut text).map_err(|e| e.to_string())?;
    }
    let desc: Value =
        serde_json::from_str(text.trim_start_matches('\u{feff}')).map_err(|e| e.to_string())?;
    descriptor(&desc)?;
    let alias = desc["id"].as_str().unwrap();
    if alias == "fh2" {
        return Err("Original FH2 cannot be replaced".into());
    }
    let mut registry = state()?;
    if let Some(e) = registry["maps"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|e| e["id"] == alias && e["packageHash"] == zip_hash)
    {
        e["enabled"] = json!(true);
        save_state(&registry)?;
        return catalog();
    }
    let id = format!("fhcm-{}-{}", alias, &zip_hash[..12]);
    let final_root = dev()?.join("maps").join(&id);
    if final_root.exists() {
        return Err("This package directory already exists; no files were replaced".into());
    }
    fs::create_dir_all(dev()?.join("fhcm-library/staging")).map_err(|e| e.to_string())?;
    let temp =
        tempfile::tempdir_in(dev()?.join("fhcm-library/staging")).map_err(|e| e.to_string())?;
    let expected: HashSet<String> = desc["files"]
        .as_array()
        .unwrap()
        .iter()
        .map(|f| format!("{}{}", prefix, f["path"].as_str().unwrap()))
        .chain(std::iter::once(manifest.to_owned()))
        .collect();
    for i in 0..zip.len() {
        let f = zip.by_index(i).map_err(|e| e.to_string())?;
        if !f.is_dir() && !expected.contains(f.name()) {
            return Err(format!("Unlisted ZIP payload: {}", f.name()));
        }
    }
    for f in desc["files"].as_array().unwrap() {
        let rel = f["path"].as_str().unwrap();
        let mut entry = zip
            .by_name(&format!("{prefix}{rel}"))
            .map_err(|_| format!("Missing package file: {rel}"))?;
        if entry.size() != f["size"].as_u64().unwrap() {
            return Err(format!("Wrong payload size: {rel}"));
        }
        let dest = temp.path().join("payload").join(rel);
        fs::create_dir_all(dest.parent().unwrap()).map_err(|e| e.to_string())?;
        let mut out = fs::File::create(&dest).map_err(|e| e.to_string())?;
        let mut hasher = Sha256::new();
        let mut buf = [0u8; 131072];
        let mut size = 0u64;
        loop {
            let n = entry.read(&mut buf).map_err(|e| e.to_string())?;
            if n == 0 {
                break;
            }
            size += n as u64;
            if size > entry.size() {
                return Err("ZIP expanded past its declared size".into());
            }
            hasher.update(&buf[..n]);
            out.write_all(&buf[..n]).map_err(|e| e.to_string())?;
        }
        if size != f["size"].as_u64().unwrap()
            || format!("{:x}", hasher.finalize())
                != f["sha256"].as_str().unwrap().to_ascii_lowercase()
        {
            return Err(format!("Package checksum failed: {rel}"));
        }
    }
    let previous = registry["maps"]
        .as_array()
        .unwrap()
        .iter()
        .position(|e| e["id"] == alias);
    let slot = previous
        .map(|i| registry["maps"][i]["slot"].as_str().unwrap().to_owned())
        .unwrap_or(format!("fhcm-{alias}"));
    let cfg = super::read_json(super::manager_dir()?.join("profiles.json"))?;
    let runtime = PathBuf::from(cfg["profiles"]["dev"]["runtime"].as_str().unwrap());
    let visibility = visibility_requirement(&desc, &temp.path().join("payload"))?;
    if hash_file(&runtime.join("fh2-mods/DevLabVisibility.dll"))? != visibility {
        return Err("This package requires a different shared visibility runtime. Update the FHCM runtime before importing.".into());
    }
    let mut normalized = json!({"schema":2,"id":id,"title":desc["name"],"status":"ready","experimental":true,"saveSlot":slot,"enableVisibilityMod":true,"visibilityModuleSha256":visibility,"spawnPosition":desc["spawnPosition"],"spawnFacing":desc["spawnFacing"],"changes":[],"seattleRuntime":{},"seattleRuntimeSource":{},"seattleData":{}});
    for f in desc["files"].as_array().unwrap() {
        let src = temp
            .path()
            .join("payload")
            .join(f["path"].as_str().unwrap());
        if f["kind"] == "module" {
            module_header(&fs::read(&src).map_err(|e| e.to_string())?)?;
        }
        let t = f["target"].as_str().unwrap();
        let dest = match f["kind"].as_str().unwrap() {
            "game" => {
                normalized["changes"]
                    .as_array_mut()
                    .unwrap()
                    .push(json!({"target":t,"source":format!("files/{t}"),"sha256":f["sha256"]}));
                Some(temp.path().join("files").join(t))
            }
            "module" => {
                normalized["seattleRuntime"][t] = f["sha256"].clone();
                normalized["seattleRuntimeSource"][t] =
                    json!(final_root.join("runtime").join(t).to_string_lossy());
                Some(temp.path().join("runtime").join(t))
            }
            "data" => {
                normalized["seattleData"][runtime
                    .join(t.replace('/', "\\"))
                    .to_string_lossy()
                    .as_ref()] = f["sha256"].clone();
                None
            }
            _ => None,
        };
        if let Some(dest) = dest {
            fs::create_dir_all(dest.parent().unwrap()).map_err(|e| e.to_string())?;
            fs::copy(src, dest).map_err(|e| e.to_string())?;
        }
    }
    let state = desc["conditionState"].as_str().unwrap();
    let flow = fs::read_to_string(
        temp.path()
            .join("files/Mount/mediaoverride/StateFlow/free_roam_start.xml"),
    )
    .map_err(|e| e.to_string())?;
    super::conditions::render_state(
        &flow,
        &super::conditions::settings(&json!({"time":"noon","weather":"clear"}))?,
        state,
    )?;
    let map_text = serde_json::to_vec_pretty(&normalized).unwrap();
    fs::write(temp.path().join("map.json"), &map_text).map_err(|e| e.to_string())?;
    fs::write(
        temp.path().join("fhcm-map.json"),
        serde_json::to_vec_pretty(&desc).unwrap(),
    )
    .map_err(|e| e.to_string())?;
    let mut img = previous
        .map(|i| registry["maps"][i]["img"].clone())
        .unwrap_or(json!("/img/pch-placeholder.png"));
    for f in desc["files"].as_array().unwrap() {
        if f["kind"] == "artwork" {
            let bytes = fs::read(
                temp.path()
                    .join("payload")
                    .join(f["path"].as_str().unwrap()),
            )
            .map_err(|e| e.to_string())?;
            if bytes.len() > 10 * 1024 * 1024 || !bytes.starts_with(b"\x89PNG\r\n\x1a\n") {
                return Err("Cover must be a PNG under 10 MiB".into());
            }
            use base64::Engine;
            img = json!(format!(
                "data:image/png;base64,{}",
                base64::engine::general_purpose::STANDARD.encode(bytes)
            ));
        }
    }
    let e = json!({"id":alias,"packageId":id,"slot":slot,"enabled":true,"favorite":previous.map(|i|registry["maps"][i]["favorite"]==true).unwrap_or(false),"order":previous.map(|i|registry["maps"][i]["order"].clone()).unwrap_or(json!(registry["maps"].as_array().unwrap().len())),"imported":true,"name":desc["name"],"region":desc["region"],"version":desc["version"],"conditionState":desc["conditionState"],"img":img,"artworkPlaceholder":desc["artworkPlaceholder"],"packageHash":zip_hash,"manifestHash":hash(&map_text)});
    fs::create_dir_all(final_root.parent().unwrap()).map_err(|e| e.to_string())?;
    fs::rename(temp.path(), &final_root).map_err(|e| e.to_string())?;
    if let Some(i) = previous {
        let old = registry["maps"][i].clone();
        if registry["history"].is_null() {
            registry["history"] = json!([]);
        }
        registry["history"].as_array_mut().unwrap().push(old);
        registry["maps"][i] = e;
    } else {
        registry["maps"].as_array_mut().unwrap().push(e);
    }
    install_manager()?;
    save_state(&registry)?;
    catalog()
}
fn add_file(
    zip: &mut zip::ZipWriter<fs::File>,
    files: &mut Vec<Value>,
    path: &str,
    kind: &str,
    target: &str,
    source: &Path,
    bytes: Option<&[u8]>,
) -> Result<()> {
    let opts = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .compression_level(Some(1));
    zip.start_file(path, opts).map_err(|e| e.to_string())?;
    let (size, sha) = if let Some(b) = bytes {
        zip.write_all(b).map_err(|e| e.to_string())?;
        (b.len() as u64, hash(b))
    } else {
        let mut f = fs::File::open(source).map_err(|e| format!("{}: {e}", source.display()))?;
        let size = f.metadata().map_err(|e| e.to_string())?.len();
        std::io::copy(&mut f, zip).map_err(|e| e.to_string())?;
        (size, hash_file(source)?)
    };
    files.push(json!({"path":path,"kind":kind,"target":target,"size":size,"sha256":sha}));
    Ok(())
}
pub fn export_zip(alias: &str, path: &Path) -> Result<Value> {
    let e = resolve(alias)?;
    let root = package_root(&e)?;
    let map = super::read_json(root.join("map.json"))?;
    if path.exists() {
        return Err("Export destination already exists. Choose a new ZIP filename; existing files are never overwritten.".into());
    }
    let mut staging =
        tempfile::NamedTempFile::new_in(path.parent().ok_or("Invalid export directory")?)
            .map_err(|e| e.to_string())?;
    let mut zip = zip::ZipWriter::new(staging.reopen().map_err(|e| e.to_string())?);
    let mut files = vec![];
    let mut desc = if e["imported"] == true {
        let d = super::read_json(root.join("fhcm-map.json"))?;
        for f in d["files"].as_array().ok_or("Invalid package")? {
            let p = f["path"].as_str().unwrap();
            let src = root.join("payload").join(p);
            if hash_file(&src)? != f["sha256"].as_str().unwrap() {
                return Err("Stored package changed".into());
            }
            add_file(
                &mut zip,
                &mut files,
                p,
                f["kind"].as_str().unwrap(),
                f["target"].as_str().unwrap(),
                &src,
                None,
            )?;
        }
        d
    } else {
        let cfg = super::read_json(super::manager_dir()?.join("profiles.json"))?;
        let game = PathBuf::from(cfg["profiles"]["dev"]["game"].as_str().unwrap());
        let mut targets = HashSet::new();
        for f in map["changes"].as_array().ok_or("Missing map files")? {
            if f["absent"] == true {
                return Err("This adapter cannot export absent map files yet".into());
            }
            let t = f["target"].as_str().unwrap();
            let src = root.join(f["source"].as_str().unwrap());
            if hash_file(&src)? != f["sha256"].as_str().unwrap().to_ascii_lowercase() {
                return Err(format!("Map source checksum failed: {t}"));
            }
            targets.insert(t.to_ascii_lowercase());
            add_file(
                &mut zip,
                &mut files,
                &format!("game/{t}"),
                "game",
                t,
                &src,
                None,
            )?;
        }
        if let Some(addon) = map["customCarAddon"]["files"].as_object() {
            for (t, h) in addon {
                if targets.contains(&t.to_ascii_lowercase()) {
                    continue;
                }
                let src = game.join(t);
                if hash_file(&src)? != h.as_str().unwrap().to_ascii_lowercase() {
                    return Err(format!(
                        "Required installed addon asset is missing or changed: {t}"
                    ));
                }
                add_file(
                    &mut zip,
                    &mut files,
                    &format!("game/{t}"),
                    "game",
                    t,
                    &src,
                    None,
                )?;
            }
        }
        let mut replacements = vec![];
        for (old, h) in map["seattleData"]
            .as_object()
            .ok_or("Missing collision dependency manifest")?
        {
            let src = PathBuf::from(old);
            if hash_file(&src)? != h.as_str().unwrap().to_ascii_lowercase() {
                return Err("Collision dependency checksum failed".into());
            }
            let name = src.file_name().unwrap().to_string_lossy();
            let target = format!("fhcm-data/{}/{}", &h.as_str().unwrap()[..16], name);
            add_file(
                &mut zip,
                &mut files,
                &format!("runtime/{target}"),
                "data",
                &target,
                &src,
                None,
            )?;
            replacements.push((old.clone(), target.replace('/', "\\")));
        }
        for (name, h) in map["seattleRuntime"]
            .as_object()
            .ok_or("Missing map runtime manifest")?
        {
            let src = PathBuf::from(
                map["seattleRuntimeSource"][name]
                    .as_str()
                    .ok_or("Missing runtime source")?,
            );
            if hash_file(&src)? != h.as_str().unwrap().to_ascii_lowercase() {
                return Err("Map runtime source checksum failed".into());
            }
            let mut bytes = fs::read(&src).map_err(|e| e.to_string())?;
            for (old, new) in &replacements {
                let pattern: Vec<u8> = old
                    .encode_utf16()
                    .flat_map(u16::to_le_bytes)
                    .chain([0, 0])
                    .collect();
                let replacement: Vec<u8> = new
                    .encode_utf16()
                    .flat_map(u16::to_le_bytes)
                    .chain([0, 0])
                    .collect();
                if replacement.len() > pattern.len() {
                    return Err(
                        "Portable runtime path exceeds the existing literal capacity".into(),
                    );
                }
                let hits: Vec<_> = bytes
                    .windows(pattern.len())
                    .enumerate()
                    .filter(|(_, b)| *b == pattern.as_slice())
                    .map(|(i, _)| i)
                    .collect();
                for at in hits {
                    bytes[at..at + pattern.len()].fill(0);
                    bytes[at..at + replacement.len()].copy_from_slice(&replacement);
                }
            }
            add_file(
                &mut zip,
                &mut files,
                &format!("runtime/{name}"),
                "module",
                name,
                &src,
                Some(&bytes),
            )?;
        }
        let provenance = fs::read(root.join("map.json")).map_err(|e| e.to_string())?;
        add_file(
            &mut zip,
            &mut files,
            "provenance/map.json",
            "provenance",
            "map.json",
            Path::new(""),
            Some(&provenance),
        )?;
        let image = if alias == "gunsai" {
            include_bytes!("../../src/img/gunsai-placeholder.png").as_slice()
        } else {
            include_bytes!("../../src/img/pch-placeholder.png").as_slice()
        };
        add_file(
            &mut zip,
            &mut files,
            "cover.png",
            "artwork",
            "cover.png",
            Path::new(""),
            Some(image),
        )?;
        json!({"format":"fhcm-map","schema":1,"adapter":"homespace02-v1","id":alias,"name":e["name"],"region":e["region"],"version":format!("local-{}",&hash_file(&root.join("map.json"))?[..8]),"conditionState":e["conditionState"],"spawnPosition":map["spawnPosition"],"spawnFacing":map["spawnFacing"],"artworkPlaceholder":true,"visibilityModuleSha256":map["visibilityModuleSha256"]})
    };
    desc["files"] = json!(files);
    descriptor(&desc)?;
    zip.start_file(
        "fhcm-map.json",
        zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated),
    )
    .map_err(|e| e.to_string())?;
    zip.write_all(&serde_json::to_vec_pretty(&desc).unwrap())
        .map_err(|e| e.to_string())?;
    zip.finish().map_err(|e| e.to_string())?;
    staging.flush().map_err(|e| e.to_string())?;
    staging.persist_noclobber(path).map_err(|e| e.to_string())?;
    Ok(
        json!({"path":path.to_string_lossy(),"sha256":hash_file(path)?,"files":desc["files"].as_array().unwrap().len()}),
    )
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn hostile_paths_and_targets_are_rejected() {
        for p in [
            "../x",
            "C:/x",
            "/x",
            "a\\b",
            "a/../b",
            "x:stream",
            "a/NUL.txt",
            "a/COM1",
            "a./b",
            "a//b",
        ] {
            assert!(relative(p).is_err(), "{p}");
        }
        assert!(relative("game/Mount/media/a.zip").is_ok());
        assert!(target("game", "Mount/media/a.zip").is_ok());
        assert!(target("game", "Mount/media/x.dll").is_err());
        assert!(target("module", "other.dll").is_err());
        assert!(target("game", "runtime/data/save").is_err());
    }
}

#[cfg(test)]
mod archive_tests {
    use super::*;
    fn archive(names: &[&str]) -> tempfile::NamedTempFile {
        let out = tempfile::NamedTempFile::new().unwrap();
        let mut zip = zip::ZipWriter::new(out.reopen().unwrap());
        for n in names {
            zip.start_file(*n, zip::write::SimpleFileOptions::default())
                .unwrap();
            zip.write_all(b"invalid").unwrap();
        }
        zip.finish().unwrap();
        out
    }
    #[test]
    fn zip_traversal_duplicates_and_missing_manifest_fail_before_install() {
        for names in [
            vec!["../outside.txt"],
            vec!["data/x", "DATA/X"],
            vec!["safe.txt"],
            vec!["x:stream"],
            vec!["CON.txt"],
            vec!["a/fhcm-map.json", "b/fhcm-map.json"],
        ] {
            let zip = archive(&names);
            assert!(import_zip(zip.path()).is_err());
        }
    }
    #[test]
    fn invalid_dlls_and_incomplete_packages_are_rejected() {
        assert!(module_header(b"not a dll").is_err());
        assert!(descriptor(&json!({"format":"other","schema":1})).is_err());
        assert!(descriptor(&json!({"format":"fhcm-map","schema":1,"adapter":"homespace02-v1","id":"test-map","name":"Test","region":"Test","version":"1.0","conditionState":"test_dry","spawnPosition":"0,0,0","spawnFacing":"0,0,1","files":[]})).is_err());
    }
}

use super::*;
use std::{process::{Command,Stdio},os::windows::process::CommandExt,sync::Mutex};
static LOCK:Mutex<()>=Mutex::new(());
fn root()->Result<PathBuf>{Ok(PathBuf::from(config()?["devRoot"].as_str().ok_or("Missing DevLab root")?).join("fhcm-sandbox"))}
fn valid_map(map:&str)->Result<()>{if super::super::library::resolve(map).is_ok(){Ok(())}else{Err("Sandbox supports installed custom map packages. Original FH2 preserves its existing save.".into())}}
fn lease_path()->Result<PathBuf>{Ok(root()?.join("active-session.json"))}
fn lease()->Result<Option<Value>>{let p=lease_path()?;if p.exists(){Ok(Some(super::super::read_json(p)?))}else{Ok(None)}}
pub fn live_path()->Result<PathBuf>{Ok(PathBuf::from(config()?["profiles"]["dev"]["save"].as_str().ok_or("Missing live save")?))}
pub fn personal_live_override()->Result<Option<PathBuf>>{
 if let Some(l)=lease()?{let p=PathBuf::from(l["personal"].as_str().ok_or("Invalid sandbox journal")?);if p.is_dir(){return Ok(Some(p));}if l["phase"]=="active"{return Err("Sandbox restoration is pending; personal save cannot be read yet".into());}}
 Ok(None)
}
fn persist(map:&str)->Result<PathBuf>{Ok(root()?.join("saves").join(map).join("player-1"))}
fn stamp()->String{SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis().to_string()}
fn write_json(path:&Path,v:&Value)->Result<()>{fs::write(path,serde_json::to_vec_pretty(v).unwrap()).map_err(|e|e.to_string())}
pub fn ensure(map:&str)->Result<PathBuf>{
 valid_map(map)?;let _lock=LOCK.lock().map_err(|e|e.to_string())?;
 if let Some(l)=lease()? {if l["map"]==map&&l["phase"]=="active"{return live_path();}}
 let target=persist(map)?;if target.is_dir(){read_session(&target)?;return Ok(target);}
 let source=save_path(map)?;let source_session=read_session(&source)?;
 let stage=root()?.join("creation").join(stamp()).join("player-1");copy_tree(&source,&stage)?;
 let s=read_session(&stage)?;let (tmp,mut db)=open_db(&s)?;
 let templates:Vec<Value>=serde_json::from_str(include_str!("../../../sandbox-integration/stock-cars.json")).map_err(|e|e.to_string())?;
 let columns:Vec<String>=db.prepare("PRAGMA table_info(Career_Garage)").map_err(|e|e.to_string())?.query_map([],|r|r.get(1)).map_err(|e|e.to_string())?.collect::<std::result::Result<_,_>>().map_err(|e|e.to_string())?;
 let mut owned:std::collections::HashSet<i64>=db.prepare("SELECT CarId FROM Career_Garage").map_err(|e|e.to_string())?.query_map([],|r|r.get(0)).map_err(|e|e.to_string())?.collect::<std::result::Result<_,_>>().map_err(|e|e.to_string())?;
 let mut next:i64=db.query_row("SELECT MAX(Id) FROM Career_Garage",[],|r|r.get::<_,Option<i64>>(0)).map_err(|e|e.to_string())?.unwrap_or(0);
 next=next.max(db.query_row("SELECT LastUsedGarageId FROM LastUsedGarageId",[],|r|r.get(0)).map_err(|e|e.to_string())?);
 let mut added=0u32;let mut added_value=0u64;let cfg=config()?;let mount=PathBuf::from(cfg["profiles"]["dev"]["game"].as_str().ok_or("Missing game")?).join("Mount");
 let tx=db.transaction().map_err(|e|e.to_string())?;
 for c in templates {
  let model=c["id"].as_i64().ok_or("Invalid stock model")?;if owned.contains(&model){continue;}
  let media=c["mediaName"].as_str().ok_or("Missing installed model identity")?;
  if ![mount.join("media/cars"),mount.join("mediaoverride/cars")].iter().any(|p|p.join(format!("{media}.zip")).is_file()){continue;}
  next+=1;let mut entry=c["template"].as_object().ok_or("Missing stock template")?.clone();
  let h=hash(format!("{}-{map}-{model}-{next}",stage.display()).as_bytes());let guid=format!("{}-{}-{}-{}-{}",&h[..8],&h[8..12],&h[12..16],&h[16..20],&h[20..32]);
  entry.insert("Id".into(),json!(next));entry.insert("Guid".into(),json!(guid));entry.insert("OriginalOwner".into(),json!("FHCM Sandbox"));
  if columns.len()!=entry.len()||columns.iter().any(|k|!entry.contains_key(k)){return Err("Sandbox stock template does not match this save's garage schema".into());}
  let values:Vec<rusqlite::types::Value>=columns.iter().map(|k|match &entry[k]{Value::Null=>rusqlite::types::Value::Null,Value::String(s)=>s.clone().into(),Value::Number(n)=>if let Some(i)=n.as_i64(){i.into()}else{n.as_f64().unwrap().into()},Value::Bool(b)=>(*b as i64).into(),_=>rusqlite::types::Value::Null}).collect();
  let sql=format!("INSERT INTO Career_Garage ({}) VALUES ({})",columns.iter().map(|k|format!("\"{k}\"")).collect::<Vec<_>>().join(","),vec!["?";columns.len()].join(","));
  tx.execute(&sql,rusqlite::params_from_iter(values)).map_err(|e|e.to_string())?;owned.insert(model);added+=1;added_value+=c["price"].as_u64().unwrap_or(0);
 }
 tx.execute("UPDATE LastUsedGarageId SET LastUsedGarageId=?",[next]).map_err(|e|e.to_string())?;tx.commit().map_err(|e|e.to_string())?;
 let count:u32=db.query_row("SELECT COUNT(*) FROM Career_Garage",[],|r|r.get(0)).map_err(|e|e.to_string())?;
 let parts:i64=db.query_row("SELECT SUM(PartsValue) FROM Career_Garage",[],|r|r.get(0)).map_err(|e|e.to_string())?;drop(db);
 let mut profile=s.profile.clone();let valoff=profile_offset(&profile,"GarageValue")?;let oldvalue=u32::from_le_bytes(profile[valoff..valoff+4].try_into().unwrap());
 for (name,value) in [("Credits",2147483647),("CarsInGarage",count),("GarageValue",(oldvalue as u64+added_value).min(u32::MAX as u64) as u32),("GaragePartsValue",parts.max(0).min(u32::MAX as i64) as u32)]{let off=profile_offset(&profile,name)?;profile[off..off+4].copy_from_slice(&value.to_le_bytes());}
 let raw=bytes(tmp.path())?;let mut database=s.database.clone();if raw.len()>database.len()-8{return Err("Sandbox exceeds this save template's database capacity".into());}
 database[4..8].copy_from_slice(&(raw.len() as u32).to_le_bytes());database[8..].fill(0);database[8..8+raw.len()].copy_from_slice(&raw);
 let mut commit=s.commit.clone();for (name,data) in [("ProfileData",profile),("PlayerDatabase",database)]{let old=commit.lines().find(|l|l.ends_with(&format!(" {name}"))).ok_or("Missing manifest blob")?.to_owned();commit=commit.replace(&old,&format!("{} {} {name}",hash(&data),data.len()));fs::write(s.revision.join("data").join(name),data).map_err(|e|e.to_string())?;}
 fs::write(s.revision.join("commit"),commit).map_err(|e|e.to_string())?;read_session(&stage)?;
 if read_session(&source)?.token!=source_session.token{return Err("Personal save changed during sandbox creation; retry when the game is closed".into());}
 fs::create_dir_all(target.parent().unwrap()).map_err(|e|e.to_string())?;fs::rename(&stage,&target).map_err(|e|e.to_string())?;
 write_json(&target.parent().unwrap().join("creation.json"),&json!({"map":map,"credits":2147483647u32,"addedCars":added,"ownedCars":count,"sourceFingerprint":source_session.token,"personalUnchanged":true}))?;Ok(target)
}
fn watcher()->Result<PathBuf>{let p=root()?.join("watch-session.ps1");fs::create_dir_all(p.parent().unwrap()).map_err(|e|e.to_string())?;fs::write(&p,include_str!("../../../sandbox-integration/watch-session.ps1")).map_err(|e|e.to_string())?;Ok(p)}
pub fn recover()->Result<()> {
 if lease()?.is_none(){return Ok(());}
 let c=config()?;let out=Command::new("powershell.exe").args(["-NoProfile","-NonInteractive","-ExecutionPolicy","Bypass","-File"]).arg(watcher()?).arg("-DevRoot").arg(c["devRoot"].as_str().unwrap()).arg("-RecoverOnly").creation_flags(0x08000000).output().map_err(|e|e.to_string())?;
 if !out.status.success(){return Err(String::from_utf8_lossy(&out.stderr).into_owned());}Ok(())
}
fn ensure_manager_guard()->Result<()> {
 let file=super::super::manager_dir()?.join("Manager.Core.ps1");let text=fs::read_to_string(&file).map_err(|e|e.to_string())?;
 if text.contains("# FHCM sandbox save lease guard"){return Ok(());}
 let marker="function Assert-Fh2Stopped {";if !text.contains(marker){return Err("Map manager cannot be guarded for sandbox sessions".into());}
 fs::create_dir_all(root()?).map_err(|e|e.to_string())?;fs::copy(&file,root()?.join(format!("Manager.Core-before-sandbox-{}.ps1",stamp()))).map_err(|e|e.to_string())?;
 let guard="\n    # FHCM sandbox save lease guard\n    $fhcmLease=Join-Path $script:Config.devRoot 'fhcm-sandbox/active-session.json'\n    if($env:FHCM_SANDBOX_OWNER -and -not(Test-Path -LiteralPath $fhcmLease)){throw 'Sandbox session expired; launch cancelled.'}\n    if(Test-Path -LiteralPath $fhcmLease){$fhcmOwner=(Get-Content -LiteralPath $fhcmLease -Raw | ConvertFrom-Json).id;if($env:FHCM_SANDBOX_OWNER -ne $fhcmOwner){throw 'FHCM Sandbox owns the active save. Close the game and let FHCM restore the personal save before using the map manager.'}}\n";
 fs::write(file,text.replacen(marker,&format!("{marker}{guard}"),1)).map_err(|e|e.to_string())?;Ok(())
}
pub fn begin(map:&str)->Result<String>{
 valid_map(map)?;let sandbox=ensure(map)?;let _lock=LOCK.lock().map_err(|e|e.to_string())?;
 if lease()?.is_some(){return Err("A sandbox session is already active".into());}ensure_manager_guard()?;
 let live=live_path()?;let personal_session=read_session(&live)?;let id=stamp();let session=root()?.join("sessions").join(&id);fs::create_dir_all(&session).map_err(|e|e.to_string())?;
 let personal=session.join("personal-player-1");let mut journal=json!({"id":id,"map":map,"phase":"prepared","live":live,"personal":personal,"session":session,"sandbox":persist(map)?,"personalFingerprint":personal_session.token});
 write_json(&lease_path()?,&journal)?;fs::rename(&live,&personal).map_err(|e|e.to_string())?;
 if let Err(e)=copy_tree(&sandbox,&live){return Err(format!("Sandbox staging interrupted; personal save is preserved for recovery: {e}"));}
 read_session(&live)?;journal["phase"]=json!("active");write_json(&lease_path()?,&journal)?;
 let c=config()?;Command::new("powershell.exe").args(["-NoProfile","-NonInteractive","-ExecutionPolicy","Bypass","-File"]).arg(watcher()?).arg("-DevRoot").arg(c["devRoot"].as_str().unwrap()).arg("-SessionId").arg(&id).stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null()).creation_flags(0x08000000).spawn().map_err(|e|e.to_string())?;Ok(id)
}


use std::{fs,path::{Path,PathBuf},time::{SystemTime,UNIX_EPOCH}};
use serde_json::{json,Value};
use sha2::{Sha256,Digest};
use rusqlite::{Connection,params};
type Result<T> = std::result::Result<T,String>;
pub mod sandbox;
fn hash(bytes:&[u8])->String {format!("{:x}",Sha256::digest(bytes))}
fn bytes(p:&Path)->Result<Vec<u8>>{fs::read(p).map_err(|e|format!("{}: {e}",p.display()))}
fn config()->Result<Value>{super::read_json(super::manager_dir()?.join("profiles.json"))}
fn save_path(map:&str)->Result<PathBuf>{
 let c=config()?;
 if map=="fh2" {return Ok(PathBuf::from(c["profiles"]["play"]["save"].as_str().ok_or("Play save is missing")?));}
 let entry=super::library::resolve(map)?;let id=entry["packageId"].as_str().ok_or("Missing map ID")?;let slot=entry["slot"].as_str().ok_or("Missing map slot")?;
 let dev=PathBuf::from(c["devRoot"].as_str().ok_or("DevLab path missing")?);
 let active=super::read_json(dev.join("active-map.json"))?;
 Ok(if active["id"]==id||active["saveSlot"]==slot {sandbox::personal_live_override()?.unwrap_or(PathBuf::from(c["profiles"]["dev"]["save"].as_str().ok_or("DevLab save missing")?))}else{let path=dev.join("save-slots").join(slot).join("player-1");if !path.exists(){super::powershell("if(Get-Process azimuth -ErrorAction SilentlyContinue){throw 'Close FH2 before creating a map save.'}")?;seed_map_slot(slot)?;}path})
}
struct Session {root:PathBuf,revision:PathBuf,commit:String,profile:Vec<u8>,database:Vec<u8>,token:String}
fn read_session(root:&Path)->Result<Session>{
 if fs::symlink_metadata(root).map_err(|e|e.to_string())?.file_type().is_symlink(){return Err("Linked save roots are unsupported".into());}
 let users:Vec<_>=fs::read_dir(root).map_err(|e|e.to_string())?.filter_map(|e|e.ok()).filter(|e|e.file_name().to_string_lossy().starts_with("User_")).collect();
 if users.len()!=1{return Err("Select a save with exactly one user profile".into());}
 let mut revisions:Vec<_>=fs::read_dir(users[0].path().join("revisions")).map_err(|e|e.to_string())?.filter_map(|e|e.ok()).filter(|e|e.path().join("commit").is_file()).collect();
 revisions.sort_by_key(|e|u64::from_str_radix(&e.file_name().to_string_lossy(),16).unwrap_or(0));
 let revision=revisions.last().ok_or("No complete save revision")?.path();
 let commit=fs::read_to_string(revision.join("commit")).map_err(|e|e.to_string())?;
 if !commit.starts_with("Azimuth complete save revision v2\n") {return Err("Unsupported save manifest".into());}
 for line in commit.lines().skip(2){
  let parts:Vec<_>=line.split_whitespace().collect();if parts.is_empty(){continue;}
  if parts.len()!=3||parts[2].contains(['/', '\\']){return Err("Invalid save manifest entry".into());}
  let data=bytes(&revision.join("data").join(parts[2]))?;
  if hash(&data)!=parts[0]||data.len().to_string()!=parts[1]{return Err("Save revision changed or failed validation; refresh the garage".into());}
 }
 let profile=bytes(&revision.join("data/ProfileData"))?;let database=bytes(&revision.join("data/PlayerDatabase"))?;
 let token=hash(&[profile.as_slice(),database.as_slice()].concat());
 Ok(Session{root:root.into(),revision,commit,profile,database,token})
}
fn payload(b:&[u8])->Result<&[u8]>{
 if b.len()<8||&b[..4]!=b"ssmc"{return Err("Unsupported save container".into());}
 let n=u32::from_le_bytes(b[4..8].try_into().unwrap()) as usize;
 if n==0||n>b.len()-8||b[n+8..].iter().any(|v|*v!=0){return Err("Invalid save container length".into());}Ok(&b[8..8+n])
}
fn career_offset(profile:&[u8])->Result<usize>{
 profile_offset(profile,"CareerCar")
}
fn profile_offset(profile:&[u8],name:&str)->Result<usize>{
 let mut marker=(name.len() as u32).to_le_bytes().to_vec();marker.extend_from_slice(name.as_bytes());marker.extend_from_slice(&[32,0,0,0,0,3,0,0,0]);
 payload(profile)?;
 let hits:Vec<_>=profile.windows(marker.len()).enumerate().filter(|(_,b)|*b==marker).map(|(i,_)|i+marker.len()).collect();
 if hits.len()!=1||hits[0]+4>profile.len(){return Err("Unsupported active-car field".into());}Ok(hits[0])
}
fn open_db(s:&Session)->Result<(tempfile::NamedTempFile,Connection)>{
 let tmp=tempfile::NamedTempFile::new().map_err(|e|e.to_string())?;fs::write(tmp.path(),payload(&s.database)?).map_err(|e|e.to_string())?;
 let db=Connection::open(tmp.path()).map_err(|e|e.to_string())?;db.execute_batch("PRAGMA trusted_schema=OFF;").map_err(|e|e.to_string())?;
 let ok:String=db.query_row("PRAGMA integrity_check",[],|r|r.get(0)).map_err(|e|e.to_string())?;if ok!="ok"{return Err("Garage database integrity check failed".into());}Ok((tmp,db))
}
pub fn active_dev_car_class() -> Result<(u32, u32)> {
    let config = config()?;
    let save = config["profiles"]["dev"]["save"].as_str().ok_or("DevLab save missing")?;
    let session = read_session(Path::new(save))?;
    let offset = career_offset(&session.profile)?;
    let current = u32::from_le_bytes(session.profile[offset..offset + 4].try_into().unwrap());
    let (_file, db) = open_db(&session)?;
    db.query_row("SELECT CarId, ClassId FROM Career_Garage WHERE Id=?1", [current], |row| {
        Ok((row.get(0)?, row.get(1)?))
    }).map_err(|e| e.to_string())
}
pub fn catalog(map:&str)->Result<Value>{
 catalog_at(map,&save_path(map)?,"personal")
}
pub fn catalog_mode(map:&str,mode:&str)->Result<Value>{
 match mode {"personal"=>catalog(map),"sandbox"=>catalog_at(map,&sandbox::ensure(map)?,"sandbox"),_=>Err("Unknown save mode".into())}
}
fn catalog_at(map:&str,path:&Path,mode:&str)->Result<Value>{
 let s=read_session(path)?;let off=career_offset(&s.profile)?;let current=u32::from_le_bytes(s.profile[off..off+4].try_into().unwrap());
 let (_tmp,db)=open_db(&s)?;let meta:Value=serde_json::from_str(include_str!("../../vehicle-integration/catalog.json")).map_err(|e|e.to_string())?;
 let mut stmt=db.prepare("SELECT Id,CarId,PerformanceIndex,DefaultManufacturerColorIndex,LiveryFileName FROM Career_Garage ORDER BY Id").map_err(|e|e.to_string())?;
 let rows=stmt.query_map([],|r|Ok((r.get::<_,i64>(0)?,r.get::<_,i64>(1)?,r.get::<_,f64>(2)?,r.get::<_,i64>(3)?,r.get::<_,String>(4)?))).map_err(|e|e.to_string())?;
 let mut cars=vec![];
 for row in rows{
  let (gid,mid,pi,_paint,livery)=row.map_err(|e|e.to_string())?;
  if map=="fh2"&&gid!=current as i64{continue;}
  let mut car=meta[mid.to_string()].clone();if car.is_null(){car=json!({"make":"Unknown","model":format!("Model {mid}"),"year":0,"drive":"—","colors":[]});}
  car["id"]=json!(format!("g{gid}"));car["garageId"]=json!(gid);car["modelId"]=json!(mid);car["pi"]=json!((pi*1000.).round().clamp(100.,999.) as i64);
  car["factoryDrive"]=car["drive"].clone();car["drive"]=json!("Saved");car["drives"]=json!(["Saved"]);car["customLivery"]=json!(!livery.is_empty());
  let mut colors=vec![json!(["Saved appearance","#33353D",-1])];
  if map!="fh2" {if let Some(c)=car["colors"].as_array(){colors.extend(c.clone());}}
  car["colors"]=json!(colors);cars.push(car);
 }
 if cars.is_empty(){return Err("No owned vehicles could be read from this save".into());}
 let co=profile_offset(&s.profile,"Credits")?;
 Ok(json!({"map":map,"mode":mode,"credits":u32::from_le_bytes(s.profile[co..co+4].try_into().unwrap()),"cars":cars,"currentCar":format!("g{current}"),"token":s.token,"readOnly":map=="fh2"}))
}

#[cfg(test)]
mod tests {
 use super::*;
 #[test]
 #[ignore = "requires a preconfigured owned Gunsai test library with the expected garage fixture"]
 fn owned_garage_selection_and_paint_preserve_other_data(){
  assert!(std::env::var_os("FHCM_MANAGER_DIR").is_some(), "set FHCM_MANAGER_DIR to the isolated owned test library");
  let actual_config=config().unwrap();let actual_manager=super::super::manager_dir().unwrap();
  let original_root=save_path("gunsai").unwrap();let original=read_session(&original_root).unwrap();
  let tmp=tempfile::tempdir().unwrap();let dev=tmp.path().join("dev");let root=dev.join("runtime/player-1");copy_tree(&original_root,&root).unwrap();
  let manager=tmp.path().join("manager");fs::create_dir_all(&manager).unwrap();
  fs::write(manager.join("profiles.json"),json!({"devRoot":dev,"profiles":{"dev":{"save":root,"game":actual_config["profiles"]["dev"]["game"]}}}).to_string()).unwrap();
  for name in ["Manager.Core.ps1","Manager.Maps.ps1","Manager.Library.ps1"]{if actual_manager.join(name).is_file(){fs::copy(actual_manager.join(name),manager.join(name)).unwrap();}}
  fs::write(dev.join("active-map.json"),json!({"id":"rogue-gunsai"}).to_string()).unwrap();
  let previous=std::env::var_os("FHCM_MANAGER_DIR");std::env::set_var("FHCM_MANAGER_DIR",&manager);
  let run=std::panic::catch_unwind(||{
   let g=catalog("gunsai").unwrap();let cars=g["cars"].as_array().unwrap();assert_eq!(cars.len(),5);
   let invalid=json!({"garageId":999999,"modelId":461,"token":g["token"]});assert!(apply("gunsai",&invalid).is_err());assert_eq!(read_session(&root).unwrap().token,g["token"]);
   let car=cars.iter().find(|c|c["modelId"]==461).unwrap();let paint=car["colors"].as_array().unwrap().last().unwrap()[2].clone();
   let selection=json!({"garageId":car["garageId"],"modelId":461,"paintIndex":paint,"token":g["token"]});
   let before=read_session(&root).unwrap();let (_bt,bd)=open_db(&before).unwrap();
   let dump=|db:&Connection,sql:&str|{let mut s=db.prepare(sql).unwrap();let n=s.column_count();let rows=s.query_map([],|r|Ok((0..n).map(|i|format!("{:?}",r.get_ref(i).unwrap())).collect::<Vec<_>>())).unwrap();rows.map(|r|r.unwrap()).collect::<Vec<_>>()};
   let parts=dump(&bd,"SELECT * FROM Career_PurchasedParts ORDER BY GarageId");let garage=dump(&bd,"SELECT Id,CarId,PerformanceIndex,Drivetrain,Engine,TuneFileName FROM Career_Garage ORDER BY Id");
   apply("gunsai",&selection).unwrap();let after=read_session(&root).unwrap();let (_at,ad)=open_db(&after).unwrap();
   assert_eq!(dump(&ad,"SELECT * FROM Career_PurchasedParts ORDER BY GarageId"),parts);assert_eq!(dump(&ad,"SELECT Id,CarId,PerformanceIndex,Drivetrain,Engine,TuneFileName FROM Career_Garage ORDER BY Id"),garage);
   let offset=career_offset(&before.profile).unwrap();let mut expected=before.profile.clone();expected[offset..offset+4].copy_from_slice(&(car["garageId"].as_u64().unwrap() as u32).to_le_bytes());assert_eq!(after.profile,expected);
   let selected:i64=ad.query_row("SELECT DefaultManufacturerColorIndex FROM Career_Garage WHERE CarId=461",[],|r|r.get(0)).unwrap();assert_eq!(selected,paint.as_i64().unwrap());
   assert_eq!(catalog("gunsai").unwrap()["cars"].as_array().unwrap().len(),5);assert!(apply("gunsai",&selection).is_err());
   let backup=fs::read_dir(dev.join("backups/launcher-vehicles")).unwrap().next().unwrap().unwrap().path();assert_eq!(read_session(&backup.join("original-player-1")).unwrap().token,before.token);
   let personal_token=read_session(&root).unwrap().token;
   let sb=catalog_mode("gunsai","sandbox").unwrap();assert!(sb["cars"].as_array().unwrap().len()>=335);assert_eq!(sb["credits"],2147483647u32);
   assert_eq!(read_session(&root).unwrap().token,personal_token);assert_eq!(catalog("gunsai").unwrap()["cars"].as_array().unwrap().len(),5);
   let owned_sb=sb["cars"].as_array().unwrap().iter().find(|c|c["modelId"]==247).unwrap();
   sandbox::begin("gunsai").unwrap();assert_eq!(catalog("gunsai").unwrap()["token"],personal_token);
   assert!(super::super::powershell(&format!("$ErrorActionPreference='Stop'; . '{}'; Assert-Fh2Stopped",manager.join("Manager.Core.ps1").display())).is_err());
   let select_sb=json!({"garageId":owned_sb["garageId"],"modelId":247,"paintIndex":-1,"token":sb["token"]});apply_mode("gunsai","sandbox",&select_sb).unwrap();
   sandbox::recover().unwrap();assert_eq!(read_session(&root).unwrap().token,personal_token);
   for entry in fs::read_dir(dev.join("fhcm-sandbox/sessions")).unwrap(){let p=entry.unwrap().path().join("restoration.json");if p.exists(){eprintln!("RESTORATION {}",fs::read_to_string(p).unwrap());}}
   let persisted=catalog_mode("gunsai","sandbox").unwrap();assert_eq!(persisted["currentCar"],owned_sb["id"]);assert_eq!(persisted["credits"],2147483647u32);
   sandbox::begin("gunsai").unwrap();let journal=dev.join("fhcm-sandbox/active-session.json");let mut j=super::super::read_json(journal.clone()).unwrap();j["phase"]=json!("prepared");fs::write(&journal,j.to_string()).unwrap();
   fs::write(root.join("interrupted-copy-test"),"partial").unwrap();sandbox::recover().unwrap();assert_eq!(read_session(&root).unwrap().token,personal_token);assert!(!root.join("interrupted-copy-test").exists());
   assert!(catalog_mode("fh2","sandbox").is_err());assert!(catalog_mode("gunsai","unknown").is_err());
  });
  if let Some(p)=previous{std::env::set_var("FHCM_MANAGER_DIR",p);}else{std::env::remove_var("FHCM_MANAGER_DIR");}
  assert_eq!(read_session(&original_root).unwrap().token,original.token);run.unwrap();
 }
}
fn copy_tree(src:&Path,dst:&Path)->Result<()>{
 fs::create_dir_all(dst).map_err(|e|e.to_string())?;
 for entry in fs::read_dir(src).map_err(|e|e.to_string())?{let e=entry.map_err(|e|e.to_string())?;let t=e.file_type().map_err(|e|e.to_string())?;if t.is_symlink(){return Err("Linked save entries unsupported".into());}if t.is_dir(){copy_tree(&e.path(),&dst.join(e.file_name()))?;}else{fs::copy(e.path(),dst.join(e.file_name())).map_err(|e|e.to_string())?;}}Ok(())
}
pub fn apply(map:&str,selection:&Value)->Result<()> {
 if map=="fh2"{return Ok(());}
 apply_at(&save_path(map)?,selection)
}
pub fn apply_mode(map:&str,mode:&str,selection:&Value)->Result<()> {
 if mode=="sandbox"{apply_at(&sandbox::live_path()?,selection)}else{apply(map,selection)}
}
fn apply_at(root:&Path,selection:&Value)->Result<()> {
 let s=read_session(root)?;
 if selection["token"]!=s.token{return Err("The save changed since this garage was loaded. Refresh the launcher before selecting a vehicle.".into());}
 let gid=selection["garageId"].as_i64().ok_or("Missing garage ID")?;let mid=selection["modelId"].as_i64().ok_or("Missing model ID")?;
 let paint=selection["paintIndex"].as_i64().unwrap_or(-1);let off=career_offset(&s.profile)?;
 let (tmp,db)=open_db(&s)?;
 let owned:i64=db.query_row("SELECT CarId FROM Career_Garage WHERE Id=?",[gid],|r|r.get(0)).map_err(|_|"The selected vehicle is not owned by this save")?;
 if owned!=mid{return Err("Garage ownership changed; refresh the launcher".into());}
 let mut profile=s.profile.clone();profile[off..off+4].copy_from_slice(&(u32::try_from(gid).map_err(|_|"Invalid garage ID")?).to_le_bytes());
 if paint>=0 {
  let meta:Value=serde_json::from_str(include_str!("../../vehicle-integration/catalog.json")).map_err(|e|e.to_string())?;let car=&meta[mid.to_string()];
  if !car["colors"].as_array().map(|cs|cs.iter().any(|c|c[2].as_i64()==Some(paint))).unwrap_or(false){return Err("Unsupported factory paint".into());}
  db.execute("UPDATE Career_Garage SET DefaultManufacturerColorIndex=?,LiveryFileName='',VersionedLiveryId=-1,Thumbnail=? WHERE Id=?",params![paint,car["thumbnail"].as_str().unwrap_or(""),gid]).map_err(|e|e.to_string())?;
 }
 drop(db);let raw=bytes(tmp.path())?;let mut database=s.database.clone();
 if paint>=0{if raw.len()>database.len()-8{return Err("Garage edit exceeds save container".into());}database[4..8].copy_from_slice(&(raw.len() as u32).to_le_bytes());database[8..].fill(0);database[8..8+raw.len()].copy_from_slice(&raw);}
 if profile==s.profile&&database==s.database{return Ok(());}
 let c=config()?;let backup=PathBuf::from(c["devRoot"].as_str().ok_or("Missing DevLab path")?).join("backups/launcher-vehicles").join(SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis().to_string());
 let candidate=backup.join("candidate-player-1");copy_tree(&s.root,&candidate)?;
 let rel=s.revision.strip_prefix(&s.root).map_err(|e|e.to_string())?;let mut commit=s.commit.clone();
 for (name,data) in [("ProfileData",profile),("PlayerDatabase",database)]{
  let old=commit.lines().find(|l|l.ends_with(&format!(" {name}"))).ok_or("Missing manifest blob")?.to_owned();
  commit=commit.replace(&old,&format!("{} {} {name}",hash(&data),data.len()));fs::write(candidate.join(rel).join("data").join(name),data).map_err(|e|e.to_string())?;
 }
 fs::write(candidate.join(rel).join("commit"),commit).map_err(|e|e.to_string())?;read_session(&candidate)?;
 if read_session(root)?.token!=s.token{return Err("Save changed during preparation; no changes installed".into());}
 let original=backup.join("original-player-1");fs::rename(&root,&original).map_err(|e|e.to_string())?;
 if let Err(e)=fs::rename(&candidate,&root){let _=fs::rename(&original,&root);return Err(e.to_string());}
 fs::write(backup.join("selection.json"),serde_json::to_vec_pretty(selection).unwrap()).map_err(|e|e.to_string())?;Ok(())
}

pub fn seed_map_slot(slot:&str)->Result<()> {
 if !slot.bytes().all(|b|b.is_ascii_lowercase()||b.is_ascii_digit()||b==b'-'){return Err("Invalid save slot".into());}
 let c=config()?;let dev=PathBuf::from(c["devRoot"].as_str().ok_or("Missing DevLab")?);
 let target=dev.join("save-slots").join(slot).join("player-1");if target.exists(){read_session(&target)?;return Ok(());}
 let source=sandbox::personal_live_override()?.unwrap_or(PathBuf::from(c["profiles"]["dev"]["save"].as_str().ok_or("Missing personal save")?));
 read_session(&source)?;copy_tree(&source,&target)?;read_session(&target)?;Ok(())
}

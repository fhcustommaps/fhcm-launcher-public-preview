use std::{fs,path::PathBuf};
use serde_json::{json,Value};
use tauri_plugin_dialog::DialogExt;
type Result<T> = std::result::Result<T,String>;
pub fn home()->Result<PathBuf>{Ok(PathBuf::from(std::env::var_os("LOCALAPPDATA").ok_or("Local application data is unavailable")?).join("FHCM/Community"))}
pub fn manager()->Result<PathBuf>{let p=home()?.join("installation.json");if !p.exists(){return Ok(home()?.join("manager"));}let v=super::read_json(p)?;Ok(PathBuf::from(v["manager"].as_str().ok_or("Invalid installation configuration")?))}
pub fn ready()->Result<bool>{Ok(super::manager_dir()?.join("profiles.json").is_file())}
pub fn initial()->Result<Value>{Ok(json!({"needsSetup":true,"destination":PathBuf::from(std::env::var_os("USERPROFILE").ok_or("Missing user profile")?).join("Documents/FHCM Library").to_string_lossy()}))}
#[tauri::command]
pub async fn pick_setup_folder(app:tauri::AppHandle)->Result<Option<String>>{tauri::async_runtime::spawn_blocking(move||app.dialog().file().blocking_pick_folder().map(|p|p.into_path().map(|p|p.to_string_lossy().to_string()).map_err(|e|e.to_string())).transpose()).await.map_err(|e|e.to_string())?}
#[tauri::command]
pub fn setup_progress()->Result<String>{Ok(fs::read_to_string(home()?.join("setup-assets/progress.txt")).unwrap_or_else(|_|"Preparing setupâ€¦".into()))}
#[tauri::command]
pub async fn complete_setup(request:Value)->Result<Value>{
 if super::BUSY.compare_exchange(false,true,std::sync::atomic::Ordering::AcqRel,std::sync::atomic::Ordering::Acquire).is_err(){return Err("An operation is already running".into());}let guard=super::BusyGuard;
 tauri::async_runtime::spawn_blocking(move||{let _guard=guard;
  if ready()?{return Err("This launcher already has an installation. Existing configuration was retained.".into());}
  let manager=prepare_library(&request, &home()?.join("setup-assets"))?;
  let staged=home()?.join("installation.new");fs::write(&staged,serde_json::to_vec_pretty(&json!({"schema":1,"manager":manager})).unwrap()).map_err(|e|e.to_string())?;fs::rename(staged,home()?.join("installation.json")).map_err(|e|e.to_string())?;
  Ok(json!({"ready":true}))
 }).await.map_err(|e|e.to_string())?
}
pub fn prepare_library(request: &Value, assets: &std::path::Path) -> Result<PathBuf> {
  for key in ["game","runtime","destination"]{if request[key].as_str().unwrap_or("").trim().is_empty(){return Err(format!("Choose {key} before continuing"));}}
  fs::create_dir_all(assets).map_err(|e|e.to_string())?;
  for (name,bytes) in [
   ("Setup.ps1",include_bytes!("../../installer-integration/Setup.ps1").as_slice()),
   ("Manager.Core.ps1",include_bytes!("../../installer-integration/manager/Manager.Core.ps1").as_slice()),
   ("Manager.Maps.ps1",include_bytes!("../../installer-integration/manager/Manager.Maps.ps1").as_slice()),
   ("Manager.Library.ps1",include_bytes!("../../library-integration/Manager.Library.ps1").as_slice()),
   ("DevLabVisibility.dll",include_bytes!("../../installer-integration/runtime/DevLabVisibility.dll").as_slice()),
   ("DevLabKeyboard.dll",include_bytes!("../../installer-integration/runtime/DevLabKeyboard.dll").as_slice()),
   ("d3dcompiler_47.dll",include_bytes!("../../installer-integration/runtime/d3dcompiler_47.dll").as_slice()),
   ("PLATFORM-LICENSE.txt",include_bytes!("../../installer-integration/runtime/PLATFORM-LICENSE.txt").as_slice())
  ]{fs::write(assets.join(name),bytes).map_err(|e|e.to_string())?;}
  fs::write(assets.join("progress.txt"),"Validating your installation and free spaceâ€¦").map_err(|e|e.to_string())?;
  fs::write(assets.join("request.json"),serde_json::to_vec_pretty(request).unwrap()).map_err(|e|e.to_string())?;
  let quote=|p:PathBuf|format!("'{}'",p.to_string_lossy().replace('\'',"''"));
  super::powershell(&format!("& {} -Request {}",quote(assets.join("Setup.ps1")),quote(assets.join("request.json"))))?;
  let manager=PathBuf::from(request["destination"].as_str().unwrap()).join("manager");
  let config=super::read_json(manager.join("profiles.json"))?;
  if !PathBuf::from(config["profiles"]["dev"]["runtime"].as_str().ok_or("Missing prepared runtime")?).join("azimuth.exe").is_file(){return Err("Prepared runtime is incomplete; configuration was not published".into());}
  Ok(manager)
}

#[cfg(test)]mod tests{
 use super::*;
 #[test]fn setup_refuses_source_overlap_without_creating_destination(){
  let tmp=tempfile::tempdir().unwrap();let source=tmp.path().join("owned");fs::create_dir(&source).unwrap();fs::write(source.join("sentinel"),b"preserve").unwrap();
  let destination=source.join("nested");let request=tmp.path().join("request.json");fs::write(&request,json!({"game":source,"runtime":source,"content":"","destination":destination}).to_string()).unwrap();let script=tmp.path().join("Setup.ps1");fs::write(&script,include_bytes!("../../installer-integration/Setup.ps1")).unwrap();
  let result=super::super::powershell(&format!("& '{}' -Request '{}'",script.display(),request.display()));assert!(result.is_err());assert!(!destination.exists());assert_eq!(fs::read(source.join("sentinel")).unwrap(),b"preserve");
 }
}

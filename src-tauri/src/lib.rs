mod vehicles;
mod conditions;
mod display;
mod library;
mod setup;
mod traffic;
mod map_cli;
use tauri_plugin_dialog::DialogExt;
use std::{fs, path::{Path, PathBuf}, process::Command, os::windows::fs::OpenOptionsExt, os::windows::process::CommandExt, sync::atomic::{AtomicBool, Ordering}};
use serde_json::{json, Value};
use tauri::Manager;
use base64::Engine;
static BUSY: AtomicBool = AtomicBool::new(false);
const MAPS: &[(&str,&str)] = &[("pacific","rogue-pch"),("gunsai","rogue-gunsai")];
fn manager_dir() -> Result<PathBuf,String> {
    if let Some(p)=std::env::var_os("FHCM_MANAGER_DIR") { return Ok(p.into()); }
    setup::manager()
}
fn read_json(path: PathBuf)->Result<Value,String>{
    let text=fs::read_to_string(&path).map_err(|e|format!("{}: {e}",path.display()))?;
    serde_json::from_str(text.trim_start_matches('\u{feff}')).map_err(|e|e.to_string())
}
#[tauri::command]
fn bootstrap()->Result<Value,String>{
    if !setup::ready()?{return setup::initial();}
    vehicles::sandbox::recover()?;
    let config=read_json(manager_dir()?.join("profiles.json"))?;
    let dev=config["devRoot"].as_str().ok_or("Missing DevLab path")?;
    let runtime=config["profiles"]["dev"]["runtime"].as_str().ok_or("Missing runtime path")?;
    let original=json!({"id":"fh2","original":true,"name":"Forza Horizon 2","region":"Original game","img":null,"blurb":"Launch the existing Play profile and its save.","type":"Open world","format":"Original","length":"â€”","size":"â€”","status":"installed","version":"Local","latest":"Local","progress":0,"tone":"#6B6E78","compat":{"traffic":false,"exp":false,"weather":["clear"]}});
    let mut maps=vec![original.clone()];
    let local=library::catalog()?;maps.extend(local["maps"].as_array().ok_or("Invalid library")?.clone());
    let exe=PathBuf::from(runtime).join("azimuth.exe");
    Ok(json!({"maps":maps,"removed":local["removed"],"vehicles":vehicles::catalog("fh2")?,"azimuth":exe.to_string_lossy(),"available":exe.is_file(),"library":PathBuf::from(dev).join("maps").to_string_lossy()}))
}
#[tauri::command]
async fn vehicle_catalog(map:String,mode:Option<String>)->Result<Value,String>{tauri::async_runtime::spawn_blocking(move||vehicles::catalog_mode(&map,mode.as_deref().unwrap_or("personal"))).await.map_err(|e|e.to_string())?}
fn powershell(script:&str)->Result<(),String>{
 let script=format!("$ProgressPreference='SilentlyContinue';try{{& {{\n{}\n}}}}catch{{[Console]::Error.WriteLine($_.Exception.Message);exit 1}}",script);
 let bytes:Vec<u8>=script.encode_utf16().flat_map(u16::to_le_bytes).collect();
 let encoded=base64::engine::general_purpose::STANDARD.encode(bytes);
 let output=Command::new("powershell.exe").args(["-NoProfile","-NonInteractive","-ExecutionPolicy","Bypass","-EncodedCommand",&encoded]).creation_flags(0x08000000).output().map_err(|e|e.to_string())?;
 if !output.status.success(){return Err(String::from_utf8_lossy(&output.stderr).trim().to_string());}Ok(())
}
fn manager_operation(manager: &Path) -> Result<fs::File, String> {
    fs::OpenOptions::new().create(true).truncate(false).write(true).share_mode(0)
        .open(manager.join("fhcm-operation.lock"))
        .map_err(|e| format!("Cannot lock the FHCM library. Close any other FHCM operation and try again: {e}"))
}
struct BusyGuard;
impl Drop for BusyGuard {fn drop(&mut self){BUSY.store(false,Ordering::Release);}}
#[tauri::command]
async fn launch_map(map:String,vehicle:Option<Value>,mode:Option<String>,session:Option<Value>)->Result<Value,String>{
    drive_map(map, vehicle, mode, session, true).await
}

async fn drive_map(map:String,vehicle:Option<Value>,mode:Option<String>,session:Option<Value>,start_game:bool)->Result<Value,String>{
    let ini=if map!="fh2"{Some(conditions::settings(session.as_ref().ok_or("Missing session conditions")?)?)}else{None};
    let display_ini=if map!="fh2"{Some(display::settings(&session.as_ref().ok_or("Missing session")?["display"]) ?)}else{None};
    let mode=mode.unwrap_or_else(||"personal".into());
    if mode!="personal"&&mode!="sandbox"{return Err("Unknown save mode".into());}
    if mode=="sandbox"&&map=="fh2"{return Err("Sandbox supports the custom maps; original FH2 preserves its existing save.".into());}
    let entry=if map=="fh2" {None}else{Some(library::available(&map)?)};
    let id=match &entry {None=>"play".to_owned(),Some(entry)=>entry["packageId"].as_str().ok_or("Missing package ID")?.to_owned()};
    let traffic=entry.as_ref().and_then(traffic::adapter)
        .map(|adapter| traffic::settings(adapter,session.as_ref().and_then(|s|s.get("traffic")))).transpose()?;
    let racer_mode=if entry.as_ref().is_some_and(library::pch_racer_profiles) {
        match session.as_ref().and_then(|s|s["drivatars"].as_str()) {
            Some("experimental")=> "tour",
            Some("standard")|None=> "standard",
            _=>return Err("Select Standard or Experimental Tour for PCH Drivatars".into()),
        }
    }else{""};
    if BUSY.compare_exchange(false,true,Ordering::AcqRel,Ordering::Acquire).is_err(){return Err("A map operation is already running".into());}
    let guard=BusyGuard;
    let manager=manager_dir()?;
    let id=id.to_owned();
    tauri::async_runtime::spawn_blocking(move ||{
        let _guard=guard;
        let _operation=manager_operation(&manager)?;
        // Embed the bridge so installed builds do not depend on the source checkout.
        let quote=|s:&str|format!("'{}'",s.replace('\'',"''"));
        vehicles::sandbox::recover()?;
        let mut owner=String::new();
        if map!="fh2" {
            powershell(&format!("$ErrorActionPreference='Stop'; . {}; Assert-Fh2Stopped",quote(&manager.join("Manager.Core.ps1").to_string_lossy())))?;
            library::prepare(&map)?;
            let prepare=format!("$ErrorActionPreference='Stop'; . {}; Assert-Fh2Stopped; if((Get-Fh2ActiveMap).id -ne {}){{Set-Fh2Map {} | Out-Null}}",quote(&manager.join("Manager.Core.ps1").to_string_lossy()),quote(&id),quote(&id));
            powershell(&prepare)?;
            let config=read_json(manager.join("profiles.json"))?;
            let game=PathBuf::from(config["profiles"]["dev"]["game"].as_str().ok_or("Missing game")?);
            let dev=PathBuf::from(config["devRoot"].as_str().ok_or("Missing DevLab")?);
            conditions::apply(&manager,&dev,&game,&map,ini.as_deref().ok_or("Missing conditions")?,traffic==Some("off"))?;
            let runtime=PathBuf::from(config["profiles"]["dev"]["runtime"].as_str().ok_or("Missing runtime")?);
            display::apply(&runtime,display_ini.as_deref().ok_or("Missing display settings")?)?;
            if mode=="sandbox"{owner=vehicles::sandbox::begin(&map)?;}
            if let Some(selection)=vehicle {if let Err(e)=vehicles::apply_mode(&map,&mode,&selection){let _=vehicles::sandbox::recover();return Err(e);}}
        }
        let racer_class = if id.starts_with("fhcm-pacific-estate36-racers-") {
            vehicles::active_dev_car_class().ok().filter(|&(_, class)| class <= 5)
                .map(|(model, class)| format!("{model},{class}")).unwrap_or_default()
        } else { String::new() };
        let mut script=format!("$env:FHCM_SANDBOX_OWNER={}; $env:FHCM_PCH_TRAFFIC_SETTINGS={}; $env:FHCM_PCH_RACER_CLASS={}; $env:FHCM_PCH_RACER_MODE={}; & {{\n{}\n}} -ManagerDirectory {} -MapId {}",quote(&owner),quote(traffic.unwrap_or("")), quote(&racer_class),quote(racer_mode),include_str!("../../bridge.ps1"),quote(&manager.to_string_lossy()),quote(&id));
        if !start_game {
            script.push_str(" -PrepareOnly");
        }
        if let Err(e)=powershell(&script){let _=vehicles::sandbox::recover();return Err(e);}
        Ok(json!({"message":if !start_game {"Map and spawn prepared. No game was started."} else if mode=="sandbox"{"FHCM Sandbox started. Your personal save will be restored when the game closes."}else{"Azimuth started with the selected vehicle, conditions and HUD."}}))
    }).await.map_err(|e|e.to_string())?
}
#[tauri::command]
async fn map_library()->Result<Value,String>{tauri::async_runtime::spawn_blocking(library::catalog).await.map_err(|e|e.to_string())?}
#[tauri::command]
async fn library_action(map:String,action:String,before:Option<String>)->Result<Value,String>{
 if BUSY.compare_exchange(false,true,Ordering::AcqRel,Ordering::Acquire).is_err(){return Err("A map operation is already running".into());}let guard=BusyGuard;
 tauri::async_runtime::spawn_blocking(move||{let _guard=guard;let _operation=manager_operation(&manager_dir()?)?;library::update(&map,&action,before.as_deref())}).await.map_err(|e|e.to_string())?
}
#[tauri::command]
async fn import_map_zip(app:tauri::AppHandle)->Result<Value,String>{
 if BUSY.compare_exchange(false,true,Ordering::AcqRel,Ordering::Acquire).is_err(){return Err("A map operation is already running".into());}let guard=BusyGuard;
 tauri::async_runtime::spawn_blocking(move||{let _guard=guard;let Some(file)=app.dialog().file().add_filter("FHCM map ZIP",&["zip"]).blocking_pick_file() else{return Ok(serde_json::json!({"cancelled":true}));};let path=file.into_path().map_err(|e|e.to_string())?;let _operation=manager_operation(&manager_dir()?)?;library::import_zip(&path)}).await.map_err(|e|e.to_string())?
}
#[tauri::command]
async fn export_map_zip(app:tauri::AppHandle,map:String)->Result<Value,String>{
 if BUSY.compare_exchange(false,true,Ordering::AcqRel,Ordering::Acquire).is_err(){return Err("A map operation is already running".into());}let guard=BusyGuard;
 tauri::async_runtime::spawn_blocking(move||{let _guard=guard;let Some(file)=app.dialog().file().set_file_name(format!("FHCM-{map}.zip")).add_filter("FHCM map ZIP",&["zip"]).blocking_save_file() else{return Ok(serde_json::json!({"cancelled":true}));};let _operation=manager_operation(&manager_dir()?)?;library::export_zip(&map,&file.into_path().map_err(|e|e.to_string())?)}).await.map_err(|e|e.to_string())?
}
pub fn run_desktop(){
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_single_instance::init(|app,_,_|{if let Some(w)=app.get_webview_window("main"){let _=w.unminimize();let _=w.show();let _=w.set_focus();}}))
        .invoke_handler(tauri::generate_handler![bootstrap,setup::pick_setup_folder,setup::setup_progress,setup::complete_setup,vehicle_catalog,launch_map,map_library,library_action,import_map_zip,export_map_zip])
        .run(tauri::generate_context!()).expect("FHCM Launcher could not start");
}

#[cfg(test)]mod script_tests{
    use super::*;
    #[test]fn embedded_powershell_parses_before_shipping(){
        for source in [include_str!("../../bridge.ps1"),include_str!("../../sandbox-integration/watch-session.ps1"),include_str!("../../library-integration/Manager.Library.ps1"),include_str!("../../installer-integration/Setup.ps1"),include_str!("../../installer-integration/manager/Manager.Core.ps1"),include_str!("../../installer-integration/manager/Manager.Maps.ps1")]{
            let dir=tempfile::tempdir().unwrap();let file=dir.path().join("embedded.ps1");fs::write(&file,source).unwrap();let path=file.to_string_lossy().replace('\'',"''");
            powershell(&format!("$errors=$null;$null=[System.Management.Automation.Language.Parser]::ParseFile('{}',[ref]$null,[ref]$errors);if($errors){{throw ($errors.Message -join '; ')}}",path)).unwrap();
        }
    }
}


pub use map_cli::run as run_map_cli;

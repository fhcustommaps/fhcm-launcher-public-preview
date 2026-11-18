use std::{ffi::OsString, path::{Path, PathBuf}};
use serde_json::json;

const MAP: &str = "pacific-estate36-racers";
const USAGE: &str = "FHCM Map Tool\n\
  fhcm-map setup <library> <owned-game> <azimuth> [storm-island]\n\
  fhcm-map import <library> <map.zip>\n\
  fhcm-map prepare <library> <off|light|medium|heavy> <standard|tour>\n\
  fhcm-map drive <library> <off|light|medium|heavy> <standard|tour>";

pub fn run(args: impl IntoIterator<Item = OsString>) -> Result<(), String> {
    let args: Vec<_> = args.into_iter().collect();
    match args.as_slice() {
        [] => println!("{USAGE}"),
        [help] if help == "--help" || help == "-h" => println!("{USAGE}"),
        [action, library, game, runtime, rest @ ..] if action == "setup" && rest.len() <= 1 => {
            let content = rest.first().map(|p| absolute(Path::new(p))).transpose()?;
            let request = json!({
                "destination": absolute(Path::new(library))?,
                "game": absolute(Path::new(game))?,
                "runtime": absolute(Path::new(runtime))?,
                "content": content,
            });
            let assets = tempfile::tempdir().map_err(|e| e.to_string())?;
            let manager = super::setup::prepare_library(&request, assets.path())?;
            println!("Created FHCM library at {}", manager.parent().unwrap().display());
        }
        [action, library, archive] if action == "import" => {
            let manager = select_library(Path::new(library))?;
            let _operation = super::manager_operation(&manager)?;
            let script = format!("$ErrorActionPreference='Stop'; . '{}'; Assert-Fh2Stopped",
                manager.join("Manager.Core.ps1").to_string_lossy().replace('\'', "''"));
            super::powershell(&script)?;
            let catalog = super::library::import_zip(Path::new(archive))?;
            if let Some(maps) = catalog["maps"].as_array() {
                for map in maps {
                    println!("{}  {}", map["name"].as_str().unwrap_or("Map"), map["version"].as_str().unwrap_or(""));
                }
            }
        }
        [action, library, traffic, racer] if action == "drive" || action == "prepare" => {
            let traffic = traffic.to_str().filter(|v| matches!(*v, "off" | "light" | "medium" | "heavy"))
                .ok_or("Choose Off, Light, Medium, or Heavy traffic (lowercase).")?;
            let drivatars = match racer.to_str() {
                Some("standard") => "standard",
                Some("tour") => "experimental",
                _ => return Err("Choose standard or tour.".into()),
            };
            select_library(Path::new(library))?;
            let entry = super::library::available(MAP)?;
            if !super::library::pch_racer_profiles(&entry) || super::traffic::adapter(&entry).is_none() {
                return Err("Import the compatible PCH release ZIP first.".into());
            }
            let start_game = action == "drive";
            let mode = if start_game { "sandbox" } else { "personal" };
            let session = json!({
                "time": "noon", "weather": "clear", "traffic": traffic, "drivatars": drivatars,
                "display": {"enabled": true, "speedometer": true, "minimap": false}
            });
            let result = tauri::async_runtime::block_on(super::drive_map(MAP.into(), None,
                Some(mode.into()), Some(session), start_game))?;
            println!("{}", result["message"].as_str().unwrap_or("Done"));
        }
        _ => return Err(USAGE.into()),
    }
    Ok(())
}

fn absolute(path: &Path) -> Result<PathBuf, String> {
    std::path::absolute(path).map_err(|e| e.to_string())
}

fn select_library(path: &Path) -> Result<PathBuf, String> {
    let manager = absolute(path)?.join("manager");
    if !manager.join("profiles.json").is_file() {
        return Err(format!("FHCM library missing at {}. Run setup first.", path.display()));
    }
    std::env::set_var("FHCM_MANAGER_DIR", &manager);
    Ok(manager)
}

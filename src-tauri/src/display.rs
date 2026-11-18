use std::{fs,path::Path};
use serde_json::Value;
pub fn settings(value:&Value)->Result<String,String>{
    let enabled=value["enabled"].as_bool().ok_or("HUD enabled must be true or false")?;
    let speedometer=value["speedometer"].as_bool().ok_or("Speedometer must be true or false")?;
    let minimap=value["minimap"].as_bool().ok_or("Minimap must be true or false")?;
    Ok(format!("[Display]\r\nHUD={}\r\nSpeedometer={}\r\nMinimap={}\r\n",enabled as u8,speedometer as u8,minimap as u8))
}
pub fn apply(runtime:&Path,ini:&str)->Result<(),String>{
    let dir=runtime.join("fh2-mods");
    if !dir.is_dir()||!runtime.join("d3dcompiler_47.dll").is_file(){return Err("HUD controls require the configured FH2 Mod Platform runtime".into());}
    let dll=dir.join("FHCMDisplay.dll");
    let bytes=include_bytes!("../../display-integration/native/build/Release/FHCMDisplay.dll");
    if fs::read(&dll).ok().as_deref()!=Some(bytes.as_slice()){
        if dll.exists(){fs::copy(&dll,dir.join(format!("FHCMDisplay-{}.backup",std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis()))).map_err(|e|e.to_string())?;}
        fs::write(dll,bytes).map_err(|e|e.to_string())?;
    }
    let path=dir.join("FHCMDisplay.ini");
    if path.exists(){fs::copy(&path,dir.join("FHCMDisplay.previous.ini")).map_err(|e|e.to_string())?;}
    fs::write(path,ini).map_err(|e|e.to_string())
}
#[cfg(test)]mod tests{
    use super::*;use serde_json::json;
    #[test]fn display_choices_are_strict_and_independent(){
        assert_eq!(settings(&json!({"enabled":false,"speedometer":true,"minimap":false})).unwrap(),"[Display]\r\nHUD=0\r\nSpeedometer=1\r\nMinimap=0\r\n");
        assert!(settings(&json!({"enabled":"false","speedometer":true,"minimap":true})).is_err());
        assert!(settings(&json!({"enabled":true})).is_err());
    }
}

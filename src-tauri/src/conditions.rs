use std::{fs,path::Path};
use serde_json::Value;
pub fn settings(value:&Value)->Result<String,String>{
    let time=value["time"].as_str().ok_or("Missing session time")?;
    let minutes=match time {
        "morning"=>345,"noon"=>720,"sunset"=>1070,"night"=>1380,
        "custom"=>value["customTime"].as_u64().filter(|m|*m<1440&&*m%15==0).ok_or("Custom time must use 15-minute steps within one day")?,
        _=>return Err("Unknown session time".into())
    };
    let weather=match value["weather"].as_str(){Some("clear")=>"Sunny",Some("wet")=>"Wet",Some("rain")=>"Rain",Some("storm")=>"Storm",_=>return Err("Unknown session weather".into())};
    Ok(format!("; FHCM launcher session conditions. Free roam only.\r\n[TimeLock]\r\nEnabled=1\r\nHour={:02}:{:02}\r\n[Weather]\r\nMode={}\r\n",minutes/60,minutes%60,weather)+if value["display"]["enabled"]==false{"[Display]\r\nHUD=0\r\n"}else{""})
}
#[cfg(test)]
pub fn render(source:&str,ini:&str,map:&str)->Result<String,String>{
    let state=match map {"pacific"=>"pch_dry","gunsai"=>"gunsai_dry_daylight",_=>return Err("Unsupported session map".into())};
    render_state(source,ini,state)
}
pub fn render_state(source:&str,ini:&str,state:&str)->Result<String,String>{
    let hour=ini.lines().find_map(|l|l.strip_prefix("Hour=")).ok_or("Missing time")?;
    let (h,m)=hour.split_once(':').ok_or("Invalid time")?;
    let mode=ini.lines().find_map(|l|l.strip_prefix("Mode=")).ok_or("Missing weather")?;
    let (severity,preset,wet,rain)=match mode {
        "Sunny"=>("Clear","Clear","0","0"),
        "Wet"=>("Threatening","ThreateningStormCCloudyA","0.6","0"),
        "Rain"=>("Threatening","ThreateningStormACloudyB","1","0.8"),
        "Storm"=>("Storm","StormBCloudyA","1","1"),
        _=>return Err("Invalid weather".into())};
    let marker=format!("<state id=\"{}\">",state);
    let start=source.find(&marker).ok_or("Map condition state unavailable")?;
    let end=start+source[start..].find("</state>").ok_or("Invalid map condition state")?+8;
    let body=format!("{}<behavior id=\"SetWeatherControl\" enable=\"true\" atmospheric_severity=\"{}\" atmospheric_preset=\"{}\" atmospheric_wetness=\"{}\" environmental_wetness=\"{}\" precipitation_level=\"{}\" /></state>",marker,severity,preset,rain,wet,rain);
    let mut result=source.to_owned();result.replace_range(start..end,&body);
    // Only the map-authored free-roam initialization nodes are changed.
    // Event scripts and shared game states retain their own controls.
    let prefix=format!("<node state=\"{}\" /><node state=\"global.set_time_of_day\" ",state);
    let mut offset=0;let mut count=0;
    while let Some(at)=result[offset..].find(&prefix){
        let start=offset+at;let time_start=start+prefix.len();
        let end=time_start+result[time_start..].find("/>").ok_or("Invalid map clock node")?+2;
        let replacement=format!("{}hours=\"{}\" minutes=\"{}\" speed=\"0\" targeted=\"false\" realtime=\"false\" gameoption=\"false\" /><node state=\"global.stop_time\" />",prefix,h,m);
        result.replace_range(start..end,&replacement);offset=start+replacement.len();count+=1;
    }
    if count!=2{return Err("Map free-roam clock layout changed; conditions were not applied".into());}
    if ini.contains("HUD=0"){result=result.replace("<node state=\"ui_screens.show_hud_and_continue\" />","<node state=\"ui_screens.show_hud_and_continue\" /><node state=\"global.suspend_all_huds\" />");}
    Ok(result)
}
pub fn apply(manager:&Path,dev:&Path,game:&Path,map:&str,ini:&str,traffic_off:bool)->Result<(),String>{
    let entry=super::library::resolve(map)?;
    let id=entry["packageId"].as_str().ok_or("Missing map package")?;
    let relative="Mount/mediaoverride/StateFlow/free_roam_start.xml";
    let source=fs::read_to_string(dev.join("maps").join(id).join("files").join(relative)).map_err(|e|e.to_string())?;
    let rendered=render_state(&source,ini,entry["conditionState"].as_str().ok_or("Missing condition state")?)?;
    let rendered=if traffic_off {super::traffic::suppress_startup(&rendered)?}else{rendered};
    install_manager_support(manager,dev)?;
    let destination=game.join(relative);
    let backup=dev.join("fhcm-conditions/backups").join(id);
    fs::create_dir_all(&backup).map_err(|e|e.to_string())?;
    let original=backup.join("free_roam_start.xml");
    if !original.exists(){fs::copy(&destination,&original).map_err(|e|e.to_string())?;}
    use sha2::{Digest,Sha256};
    let original_hash=format!("{:x}",Sha256::digest(source.as_bytes()));
    let session_hash=format!("{:x}",Sha256::digest(rendered.as_bytes()));
    let record=serde_json::json!({"map":id,"originalHash":original_hash,"sessionHash":session_hash});
    fs::write(dev.join("fhcm-conditions/active.json"),record.to_string()).map_err(|e|e.to_string())?;
    fs::write(&destination,rendered).map_err(|e|e.to_string())?;
    fs::write(backup.join("last-session.ini"),ini).map_err(|e|e.to_string())
}
fn install_manager_support(manager:&Path,dev:&Path)->Result<(),String>{
    let path=manager.join("Manager.Maps.ps1");
    let source=fs::read_to_string(&path).map_err(|e|e.to_string())?;
    if source.contains("# FHCM_SESSION_CONDITIONS_V1"){return Ok(());}
    let old="elseif((Get-Fh2Hash -LiteralPath $target -Algorithm SHA256).Hash -ne $change.sha256){throw \"Untracked map change: $target\"}";
    if !source.contains(old){return Err("Map manager validation layout changed; conditions support was not installed".into());}
    let replacement=r#"elseif((Get-Fh2Hash -LiteralPath $target -Algorithm SHA256).Hash -ne $change.sha256){
   # FHCM_SESSION_CONDITIONS_V1: one tracked generated StateFlow variant only.
   $accepted=$false
   $conditionsFile=Join-Path $script:Config.devRoot 'fhcm-conditions\active.json'
   if($Map.id -in @('rogue-pch','rogue-gunsai') -and $change.target -ceq 'Mount/mediaoverride/StateFlow/free_roam_start.xml' -and (Test-Path -LiteralPath $conditionsFile)){
    $conditions=Get-Content -LiteralPath $conditionsFile -Raw | ConvertFrom-Json
    $accepted=$conditions.map -eq $Map.id -and $conditions.originalHash -eq $change.sha256 -and $conditions.sessionHash -eq (Get-Fh2Hash -LiteralPath $target -Algorithm SHA256).Hash
   }
   if(-not $accepted){throw "Untracked map change: $target"}
  }"#;
    let backup=dev.join("fhcm-conditions/Manager.Maps-before-conditions.ps1");
    fs::create_dir_all(backup.parent().unwrap()).map_err(|e|e.to_string())?;
    if !backup.exists(){fs::copy(&path,backup).map_err(|e|e.to_string())?;}
    fs::write(path,source.replace(old,replacement)).map_err(|e|e.to_string())
}
#[cfg(test)]mod tests{
    use super::*;use serde_json::json;
    #[test]fn validates_conditions(){
        assert!(settings(&json!({"time":"night","weather":"rain"})).unwrap().contains("Hour=23:00"));
        assert!(settings(&json!({"time":"custom","customTime":1425,"weather":"wet"})).unwrap().contains("Hour=23:45"));
        for bad in [json!({"time":"custom","customTime":1440,"weather":"clear"}),json!({"time":"custom","customTime":-1,"weather":"clear"}),json!({"time":"noon","weather":"overcast"}),json!({"time":"oops","weather":"clear"})]{assert!(settings(&bad).is_err());}
    }
    #[test]fn renders_map_conditions_without_changing_events(){
        for (map,source) in [("pacific",include_str!("../../conditions-integration/fixtures/pacific.xml")),("gunsai",include_str!("../../conditions-integration/fixtures/gunsai.xml"))]{
            for weather in ["clear","wet","rain","storm"]{
                let ini=settings(&json!({"time":"custom","customTime":1425,"weather":weather})).unwrap();
                let output=render(source,&ini,map).unwrap();
                assert_eq!(output.matches("hours=\"23\" minutes=\"45\" speed=\"0\"").count(),2);
                assert_eq!(output.matches("<state ").count(),source.matches("<state ").count());
                assert!(output.contains("<node state=\"global.unpause_weather\" />"));
                assert_eq!(output.matches("<behavior ").count(),source.matches("<behavior ").count());
            }
            assert!(render("<stategroup />",&settings(&json!({"time":"noon","weather":"clear"})).unwrap(),map).is_err());
        }
    }
}

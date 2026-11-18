import {build} from 'esbuild';
import fs from 'node:fs';
const native=process.argv.includes('--native');
const output=native?'dist-native':'dist';
const read=p=>fs.readFileSync(p,'utf8');
let app=read('src/app.jsx');
app=app.replace("const AZ_DEFAULT='C:\\\\Games\\\\Azimuth\\\\azimuth.exe';",'const AZ_DEFAULT=window.FHCM_RUNTIME.azimuth;');
app=app.replace(/const azOk=.*?;/,'const azOk=p=>p===window.FHCM_RUNTIME.azimuth && window.FHCM_RUNTIME.available;');
app=app.replace(/function WinCtl\(\)\{[\s\S]*?\n\}/,'function WinCtl(){return null;}');
app=app.replace(/  useEffect\(\(\)=>\{const t=setInterval\(\(\)=>setMaps[\s\S]*?\},\[\]\);/,'');
app=app.replace(/  const drive=\(\)=>\{[\s\S]*?\},160\)\};/,`  const drive=async()=>{setLaunching(1);try {const r=await fetch('/api/launch',{method:'POST',headers:{'Content-Type':'application/json','X-FHCM-Token':window.FHCM_RUNTIME.token},body:JSON.stringify({map:cfg.map})});const result=await r.json();if(!r.ok)throw Error(result.error);toast({msg:result.message,tone:'ok'});setRecent(r=>[{when:'Just now',cfg},...r].slice(0,3));}catch(e){toast({msg:e.message,tone:'err'});}finally{setLaunching(0);}};`);
app=app.replace('Ready. Launches through Azimuth.','Ready. Selected owned vehicle will be applied; session controls are previews.');
app=app.replace('Azimuth detected â€” build 1.4.2 (sample).','Managed Azimuth executable found.');
app=app.replace('41.2 GB free','disk space not measured');
app=app.replace("library:'D:\\\\FHCM\\\\Maps'",'library:window.FHCM_RUNTIME.library');
app=app.replace('Vehicle</','Vehicle Â· preview</').replace('Session</','Session Â· preview</');
if(native){
 app=app.replace('function WinCtl(){return null;}',`function WinCtl(){return <div style={{display:'flex',alignSelf:'stretch'}}>{[['minus','Minimise',()=>getCurrentWindow().minimize()],['square','Maximise',()=>getCurrentWindow().toggleMaximize()],['x','Close',()=>getCurrentWindow().close()]].map(([icon,label,action])=><button key={icon} aria-label={label} title={label} onClick={action} style={{width:42,border:0,background:'transparent',color:'var(--l-ink)'}}><LIcon name={icon} size={14}/></button>)}</div>}`);
 app=app.replace("const r=await fetch('/api/launch',{method:'POST',headers:{'Content-Type':'application/json','X-FHCM-Token':window.FHCM_RUNTIME.token},body:JSON.stringify({map:cfg.map})});const result=await r.json();if(!r.ok)throw Error(result.error);", "const result=await invoke('launch_map',{map:cfg.map,mode:effectiveSaveMode,session:{time:cfg.time,customTime:cfg.customTime,weather:cfg.weather,traffic:cfg.traffic,drivatars:cfg.drivatars,display:displayFor(cfg)},vehicle:{garageId:car.garageId,modelId:car.modelId,paintIndex:car.colors.find(p=>p[0]===cfg.paint)?.[2]??-1,token:garage.token}}); const fresh=await window.FHCM_vehicleCatalog(cfg.map,effectiveSaveMode);setGarage(fresh);garageRef.current=fresh;FD.cars=fresh.cars;setCfg(c=>({...c,car:fresh.currentCar,paint:'Saved appearance'}));");
 app=app.replace('<header style=', '<header data-tauri-drag-region style=');
 app=app.replace('<div style={{flex:1}}></div>','<div data-tauri-drag-region style={{flex:1,height:"100%"}}></div>');
 app=app.replace('msg:e.message','msg:String(e.message||e)');
}
let panels=read('src/panels.jsx').replace(/const FD=window.FHL;/,'const FD=window.FHL;');
const switchSource=read('src/switch.jsx').replace("import React from 'react';",'').replace('export function Switch','function Switch');
const pre=`import React from 'react';import * as ReactDOM from 'react-dom/client';import * as Icons from 'lucide-react';
${native?"import {invoke} from '@tauri-apps/api/core';import {getCurrentWindow} from '@tauri-apps/api/window';"+read('src/setup.jsx')+"async function start(){window.FHCM_RUNTIME=await invoke('bootstrap');if(window.FHCM_RUNTIME.needsSetup){ReactDOM.createRoot(document.getElementById('root')).render(<FirstRunSetup/>);return;}window.FHCM_vehicleCatalog=(map,mode='personal')=>invoke('vehicle_catalog',{map,mode});window.FHCM_library={load:()=>invoke('map_library'),change:(map,action,before=null)=>invoke('library_action',{map,action,before}),importZip:()=>invoke('import_map_zip'),exportZip:map=>invoke('export_map_zip',{map})};":''}
${switchSource}
function Icon({name,...props}){const key=name.split('-').map(s=>s[0].toUpperCase()+s.slice(1)).join('');const C=Icons[key];return C?<C aria-hidden="true" {...props}/>:null;}
window.FHCMDesignSystem_b88bf6={Icon,Switch};
`;
let data=read('src/data.js');
data+=`\nwindow.FHL.maps=window.FHCM_RUNTIME.maps;window.FHL.version='0.2.1-preview';window.FHL.recent=[];window.FHL.presets=window.FHL.presets.map(p=>({...p,cfg:{...p.cfg,map:window.FHCM_RUNTIME.maps.some(m=>m.id===p.cfg.map)?p.cfg.map:window.FHCM_RUNTIME.maps[1]?.id||'fh2'}}));`;
data+=`\nif(window.FHCM_RUNTIME.vehicles){const g=window.FHCM_RUNTIME.vehicles;window.FHL.cars=g.cars;window.FHL.garageMap=g.map;window.FHL.presets=window.FHL.presets.map(p=>({...p,cfg:{...p.cfg,car:g.currentCar,paint:'Saved appearance',cls:'Stock',build:'Saved build',drive:g.cars.find(c=>c.id===g.currentCar)?.drive||g.cars[0].drive}}));}`;
fs.mkdirSync(output,{recursive:true});
await build({stdin:{contents:pre+data+read('src/ui.jsx')+panels+app+(native?`\n}start().catch(e=>{document.getElementById('root').textContent='FHCM could not start: '+String(e);});`:''),resolveDir:process.cwd(),loader:'jsx'},bundle:true,outfile:output+'/app.js',minify:false});
let html=read('src/index.html').replace(/<script[\s\S]*?<\/script>/g,'');
html=html.replace('</head>',`${native?'':'<script src="/api/bootstrap.js"></script>'}<script defer src="/app.js"></script></head>`);
fs.cpSync('ds',output+'/ds',{recursive:true});fs.cpSync('src/img',output+'/img',{recursive:true});
fs.writeFileSync(output+'/index.html',html.replace('../ds/','/ds/'));
console.log('Built offline launcher assets.');





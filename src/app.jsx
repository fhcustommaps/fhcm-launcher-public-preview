/* FHCM Launcher — shell, bars, drawers, app state */
const {Switch:DSwitch}=window.FHCMDesignSystem_b88bf6;
const AZ_DEFAULT='C:\\Games\\Azimuth\\azimuth.exe';
const piCls=pi=>pi>=999?'X':pi>900?'S2':pi>800?'S1':pi>700?'A':pi>600?'B':pi>500?'C':'D';
const azOk=p=>/azimuth\.exe$/i.test(p.trim());


const displayFor=c=>({enabled:c.hud!=='off',speedometer:c.hud!=='minimal',minimap:c.hud!=='minimal'&&c.hud!=='nomap',...(c.display||{})});
const hudChoice=d=>!d.enabled?'off':d.speedometer?(d.minimap?'full':'nomap'):(d.minimap?'custom':'minimal');
function restoreConfig(c){if(!c||!FD.maps.some(m=>m.id===c.map))return normalize({...FD.presets[0].cfg,map:FD.maps[0].id},FD.maps);return normalize(c,FD.maps);}
function storedSession(){try{return JSON.parse(localStorage.getItem('fhcm-session-v1'))||{};}catch{return {};}}
function storedPresets(){try{const p=JSON.parse(localStorage.getItem('fhcm-presets-v1'));if(Array.isArray(p)&&p.length)return p.filter(x=>typeof x.id==='string'&&typeof x.name==='string'&&x.cfg).slice(0,64).map(x=>({...x,cfg:FD.maps.some(m=>m.id===x.cfg.map)?restoreConfig(x.cfg):x.cfg}));}catch{}return FD.presets;}

function normalize(c,maps){
  const car=carById(c.car);const map=maps.find(m=>m.id===c.map)||maps[0];const n={...c,map:map.id};
  if(!car.drives.includes(n.drive))n.drive=car.drive;
  if(!car.colors.some(x=>x[0]===n.paint))n.paint=car.colors[0][0];
  if(!map.compat.weather.includes(n.weather))n.weather='clear';
  if(n.drivatars==='experimental'&&!map.compat.exp)n.drivatars='standard';
  if(map.compat.pchRacers){
    n.drivatars=n.drivatars==='experimental'?'experimental':'standard';
    n.count=n.drivatars==='experimental'?10:6;
    if(n.drivatars==='experimental')n.profile='convoy';
  }
  const max=FD.limits[n.drivatars];if(max&&n.count>max)n.count=max;
  if(map.compat.pchRacers&&n.drivatars==='experimental')n.count=10;
  n.display=displayFor(n);n.hud=hudChoice(n.display);
  return n;
}

function WinCtl(){
  return <div style={{display:'flex',alignSelf:'stretch',marginRight:-16,marginLeft:8,borderLeft:'1px solid var(--border-subtle)'}}>
    {['minus','square','x'].map(i=><button key={i} aria-label={{minus:'Minimise',square:'Maximise',x:'Close'}[i]} style={{width:46,border:'none',background:'transparent',color:'var(--l-ink2)',display:'inline-flex',alignItems:'center',justifyContent:'center',cursor:'default'}}
      onMouseEnter={e=>e.currentTarget.style.background=i==='x'?'var(--red-500)':'var(--l-hov)'} onMouseLeave={e=>e.currentTarget.style.background='transparent'}><LIcon name={i} size={i==='square'?12:15}/></button>)}
  </div>;
}

function DownloadsPopover({maps,open,onClose,anchor,onPause,onCancel,onDownload,library}){
  const items=maps.filter(m=>m.status==='downloading'||m.status==='paused');
  const updates=maps.filter(m=>m.status==='update');
  const gb=parseFloat;
  return <Popover anchorRef={anchor} open={open} onClose={onClose} width={360}>
    <div style={{display:'flex',justifyContent:'space-between',alignItems:'center',padding:'12px 14px 8px'}}><span style={{...LB,color:'var(--l-ink)'}}>Downloads</span><span style={{...MONO,fontSize:11}}>{items.length} active</span></div>
    {items.length===0&&updates.length===0&&<div style={{padding:'6px 14px 16px',font:'400 13px/1.5 var(--font-body)',color:'var(--text-muted)'}}>Nothing downloading. Installs and updates appear here.</div>}
    {items.map(m=>{const tot=gb(m.size);const done=(tot*m.progress/100).toFixed(2);
      return <div key={m.id} style={{padding:'10px 14px',borderTop:'1px solid var(--border-subtle)',display:'flex',flexDirection:'column',gap:8}}>
        <div style={{display:'flex',alignItems:'center',gap:8}}>
          <span style={{flex:1,minWidth:0,font:'italic 700 13px/1.2 var(--font-display)',textTransform:'uppercase',...ELL}}>{m.name}</span>
          <IconBtn size={26} iconSize={13} icon={m.status==='paused'?'play':'pause'} label={m.status==='paused'?'Resume':'Pause'} onClick={()=>onPause(m.id)}/>
          <IconBtn size={26} iconSize={13} icon="x" label="Cancel" onClick={()=>onCancel(m.id)}/>
        </div>
        <Bar v={m.progress} h={3} paused={m.status==='paused'}/>
        <div style={{display:'flex',justifyContent:'space-between',...MONO,fontSize:11}}><span>{done} / {m.size}</span><span>{m.status==='paused'?'Paused':`${Math.floor(m.progress)}% · 6.2 MB/s`}</span></div>
      </div>})}
    {updates.map(m=><div key={m.id} style={{padding:'10px 14px',borderTop:'1px solid var(--border-subtle)',display:'flex',alignItems:'center',gap:10}}>
      <div style={{flex:1,minWidth:0}}><div style={{font:'italic 700 13px/1.2 var(--font-display)',textTransform:'uppercase'}}>{m.name}</div><div style={{...MONO,fontSize:11,marginTop:3}}>v{m.version} → v{m.latest}</div></div>
      <SmallBtn tone="gold" icon="refresh-cw" onClick={()=>onDownload(m.id)}>Update</SmallBtn></div>)}
    <div style={{padding:'10px 14px',borderTop:'1px solid var(--border-subtle)',...MONO,fontSize:11,display:'flex',gap:6,alignItems:'center'}}><LIcon name="hard-drive" size={12}/>{library} · 41.2 GB free</div>
  </Popover>;
}

function SettingsDrawer({open,onClose,settings,setSettings,version}){
  const [path,setPath]=useState(settings.azimuth);
  useEffect(()=>{if(open)setPath(settings.azimuth)},[open]);
  if(!open)return null;
  const ok=azOk(path);
  const inp={flex:1,minWidth:0,height:32,padding:'0 10px',background:'var(--surface-sunken)',border:'1px solid var(--border-default)',color:'var(--l-ink)',font:'500 12px var(--font-mono)',outline:'none'};
  const commit=v=>{setPath(v);setSettings(s=>({...s,azimuth:v}))};
  return <div style={{position:'fixed',inset:'52px 0 0 0',zIndex:60}}>
    <div onClick={onClose} style={{position:'absolute',inset:0,background:'rgba(14,15,17,0.5)'}}></div>
    <div role="dialog" aria-label="Settings" style={{position:'absolute',top:0,right:0,bottom:0,width:'min(420px,100%)',background:'var(--l-bg1)',borderLeft:'1px solid var(--border-default)',boxShadow:'var(--shadow-lg)',display:'flex',flexDirection:'column'}}>
      <div style={{display:'flex',alignItems:'center',justifyContent:'space-between',padding:'14px 12px 14px 20px',borderBottom:'1px solid var(--border-subtle)'}}>
        <span style={{font:'600 18px/1.2 var(--font-display)'}}>Settings</span><IconBtn icon="x" label="Close settings" onClick={onClose}/>
      </div>
      <div style={{flex:1,overflow:'auto',padding:20,display:'flex',flexDirection:'column',gap:24}}>
        <Field label="Azimuth executable" note={ok?<Note tone="ok">Azimuth detected — build 1.4.2 (sample).</Note>:<Note tone="err">azimuth.exe not found at this path. FHCM Launcher needs Azimuth to start the game.</Note>}>
          <div style={{display:'flex',gap:8}}>
            <input value={path} onChange={e=>commit(e.target.value)} aria-label="Azimuth path" spellCheck={false} style={{...inp,borderColor:ok?'var(--border-default)':'var(--red-300)'}}/>
            <SmallBtn icon="folder-open" onClick={()=>commit(AZ_DEFAULT)}>Browse</SmallBtn>
          </div>
        </Field>
        <Field label="Map library folder">
          <div style={{display:'flex',gap:8}}><input value={settings.library} onChange={e=>setSettings(s=>({...s,library:e.target.value}))} aria-label="Map library folder" spellCheck={false} style={inp}/><SmallBtn icon="folder-open">Browse</SmallBtn></div>
        </Field>
        <div style={{display:'flex',flexDirection:'column',borderTop:'1px solid var(--border-subtle)',paddingTop:12}}>
          <DSwitch label="Check for map updates on start" checked={settings.autoUpdate} onChange={v=>setSettings(s=>({...s,autoUpdate:v}))}/>
          <DSwitch label="Close launcher when the game starts" checked={settings.closeOnLaunch} onChange={v=>setSettings(s=>({...s,closeOnLaunch:v}))}/>
        </div>
        <div style={{borderTop:'1px solid var(--border-subtle)',paddingTop:16,display:'flex',flexDirection:'column',gap:8}}>
          <span style={LB}>About</span>
          <span style={{font:'500 14px var(--font-body)'}}>FHCM Launcher <span style={MONO}>v{version}</span></span>
          <span style={{font:'400 12px/1.5 var(--font-body)',color:'var(--text-muted)',textWrap:'pretty'}}>FHCM is an independent, non-commercial fan project. Not affiliated with or endorsed by Microsoft, Xbox Game Studios, Playground Games or Turn 10. Players use their own game files.</span>
        </div>
      </div>
    </div>
  </div>;
}

function OverlayBtn({icon,label,active,onClick}){
  const [h,hp]=useHover();
  return <button aria-label={label} title={label} aria-pressed={active} onClick={onClick} {...hp}
    style={{position:'relative',width:44,height:44,display:'inline-flex',alignItems:'center',justifyContent:'center',border:'none',background:active?'var(--l-sel)':h?'var(--l-hov)':'transparent',color:active||h?'var(--l-ink)':'var(--l-ink2)',cursor:'pointer'}}>
    <LIcon name={icon} size={18}/>{active&&<span style={{position:'absolute',top:0,bottom:0,left:-1,width:2,background:'var(--gradient-horizon)'}}></span>}</button>;
}
function DriveButton({disabled,launching,onClick}){
  const [h,hp]=useHover();const on=!disabled&&!launching;
  return <button onClick={onClick} disabled={disabled||launching} {...hp} aria-label="Drive"
    style={{position:'relative',width:232,height:60,flexShrink:0,border:'none',clipPath:'var(--clip-cut-md)',overflow:'hidden',
      background:disabled?'var(--neutral-800)':'var(--gradient-horizon-h)',backgroundSize:'160% 100%',backgroundPosition:h&&on?'100% 0':'0 0',
      color:disabled?'var(--neutral-500)':'#fff',cursor:on?'pointer':disabled?'not-allowed':'progress',display:'flex',alignItems:'center',justifyContent:'center',gap:12,
      transition:'background-position var(--dur-slow) var(--ease-out)'}}>
    <span style={{font:'italic 800 26px/1 var(--font-display)',letterSpacing:'0.02em'}}>{launching?'LAUNCHING':'DRIVE'}</span>
    {!launching&&<LIcon name="chevron-right" size={26} strokeWidth={4} style={{transform:h&&on?'translateX(4px)':'none',transition:'transform var(--dur-base) var(--ease-out)'}}/>}
    {!!launching&&<span style={{position:'absolute',left:0,bottom:0,height:3,width:launching+'%',background:'#fff',transition:'width 160ms linear'}}></span>}
  </button>;
}

function Summary({label,a,b}){
  return <div style={{minWidth:0,display:'flex',flexDirection:'column',gap:4,padding:'0 20px',borderLeft:'1px solid var(--border-subtle)'}}>
    <span style={{...LB,fontSize:10}}>{label}</span>
    <span style={{font:'600 14px/1.2 var(--font-display)',color:'var(--l-ink)',...ELL}}>{a}</span>
    <span style={{...MONO,fontSize:11,...ELL}}>{b}</span>
  </div>;
}

function App(){
  const [maps,setMaps]=useState(()=>FD.maps.map(m=>({...m})));
  const [presets,setPresets]=useState(storedPresets);
  const [removedMaps,setRemovedMaps]=useState(window.FHCM_RUNTIME.removed||[]);
  const [libraryBusy,setLibraryBusy]=useState(false);
  const [libraryError,setLibraryError]=useState(null);
  const [libraryOpen,setLibraryOpen]=useState(false);
  const applyLibrary=r=>{const next=[FD.maps.find(m=>m.original),...r.maps];FD.maps=next;setMaps(next);setRemovedMaps(r.removed||[]);setCfg(c=>normalize(c,next));setRecent(rs=>rs.filter(x=>next.some(m=>m.id===x.cfg.map)));};
  const libraryAction=async(map,action,before=null)=>{if(libraryBusy||launching)return;setLibraryBusy(true);setLibraryError(null);try{const r=await window.FHCM_library.change(map,action,before);applyLibrary(r);if(action==='remove')toast({msg:'Removed from the library. Files and saves retained; restore from Manage maps.',tone:'ok'});}catch(e){setLibraryError(String(e.message||e));toast({msg:String(e.message||e),tone:'err'});}finally{setLibraryBusy(false);}};
  const importZip=async()=>{if(libraryBusy||launching)return;setLibraryBusy(true);setLibraryError(null);try{const r=await window.FHCM_library.importZip();if(!r.cancelled){applyLibrary(r);toast({msg:'Map ZIP validated and added. Select the map and click Drive.',tone:'ok'});}}catch(e){setLibraryError(String(e.message||e));toast({msg:String(e.message||e),tone:'err'});}finally{setLibraryBusy(false);}};
  const exportZip=async map=>{if(libraryBusy||launching)return;setLibraryBusy(true);setLibraryError(null);try{const r=await window.FHCM_library.exportZip(map);if(!r.cancelled)toast({msg:'Portable ZIP exported: '+r.path,tone:'ok'});}catch(e){setLibraryError(String(e.message||e));toast({msg:String(e.message||e),tone:'err'});}finally{setLibraryBusy(false);}};
  const [presetId,setPresetId]=useState(()=>storedSession().presetId||'p1');
  const [cfg,setCfg]=useState(()=>restoreConfig(storedSession().cfg||FD.presets[0].cfg));
  useEffect(()=>{localStorage.setItem('fhcm-presets-v1',JSON.stringify(presets));},[presets]);
  useEffect(()=>{localStorage.setItem('fhcm-session-v1',JSON.stringify({cfg,presetId}));},[cfg,presetId]);
  const [recent,setRecent]=useState(FD.recent);
  const [settings,setSettings]=useState({azimuth:new URLSearchParams(location.search).get('azimuth')==='missing'?'C:\\Games\\azimuth-old\\':AZ_DEFAULT,library:'D:\\FHCM\\Maps',autoUpdate:true,closeOnLaunch:false});
  const [panel,setPanel]=useState(null);
  const [mode,setModeS]=useState(()=>localStorage.getItem('fhl-mode')||'dark');
  const setMode=v=>{setModeS(v);localStorage.setItem('fhl-mode',v);setOverlay(null)};
  const [sysDark,setSysDark]=useState(()=>matchMedia('(prefers-color-scheme: dark)').matches);
  useEffect(()=>{const q=matchMedia('(prefers-color-scheme: dark)');const f=e=>setSysDark(e.matches);q.addEventListener('change',f);return()=>q.removeEventListener('change',f)},[]);
  const view=mode==='hero'?'immersive':'standard';
  const theme=mode==='light'||(mode==='system'&&!sysDark)?'light':'dark';
  const [mapsOpen,setMapsOpenS]=useState(()=>localStorage.getItem('fhl-maps')!=='0');
  const setMapsOpen=f=>setMapsOpenS(o=>{const n=typeof f==='function'?f(o):f;localStorage.setItem('fhl-maps',n?'1':'0');return n});
  const [overlay,setOverlay]=useState(null);
  useEffect(()=>{if(!overlay)return;const k=e=>{if(e.key==='Escape'&&!document.querySelector('[aria-expanded="true"]'))setOverlay(null)};document.addEventListener('keydown',k);return()=>document.removeEventListener('keydown',k)},[overlay]);
  const [toastMsg,setToastMsg]=useState(null);
  const [launching,setLaunching]=useState(0);
  const [garage,setGarage]=useState(window.FHCM_RUNTIME.vehicles||null);
  const garageRef=useRef(window.FHCM_RUNTIME.vehicles||null);
  const [garageBusy,setGarageBusy]=useState(false);
  const [saveMode,setSaveModeState]=useState(()=>localStorage.getItem('fhcm-save-mode')||'personal');
  const setSaveMode=v=>{setSaveModeState(v);localStorage.setItem('fhcm-save-mode',v)};
  const effectiveSaveMode=cfg.map==='fh2'?'personal':saveMode;
  useEffect(()=>{
    if(!window.FHCM_vehicleCatalog)return;
    let cancelled=false;
    const load=async()=>{
      setGarageBusy(true);
      try{
        let g;try{g=await window.FHCM_vehicleCatalog(cfg.map,effectiveSaveMode)}catch(e){await new Promise(r=>setTimeout(r,500));g=await window.FHCM_vehicleCatalog(cfg.map,effectiveSaveMode)}
        if(cancelled)return;
        const previous=garageRef.current;FD.cars=g.cars;FD.garageMap=g.map;garageRef.current=g;setGarage(g);
        if(previous?.map!==g.map||previous?.mode!==g.mode||previous?.token!==g.token)setCfg(c=>normalize({...c,car:g.currentCar,paint:'Saved appearance',cls:'Stock',build:'Saved build'},maps));
        setGarageBusy(false);
      }catch(e){if(!cancelled){setGarageBusy(false);setGarage(null);toast({msg:'Garage unavailable: '+String(e),tone:'err'});}}
    };
    load();window.addEventListener('focus',load);
    return()=>{cancelled=true;window.removeEventListener('focus',load)};
  },[cfg.map,effectiveSaveMode]);
  const dlRef=useRef();

  const toast=t=>{setToastMsg({...t,k:Date.now()})};
  useEffect(()=>{if(!toastMsg)return;const t=setTimeout(()=>setToastMsg(null),3400);return()=>clearTimeout(t)},[toastMsg]);

  const set=(k,v)=>setCfg(c=>normalize({...c,[k]:v,...(k==='hud'?{display:displayFor({hud:v})}:{}),...(k==='car'?{paint:'Saved appearance',cls:'Stock',build:'Saved build'}:{})},maps));
  const applyCfg=c=>setCfg(previous=>normalize({...c,...(window.FHCM_vehicleCatalog?{car:previous.car,paint:previous.paint,drive:previous.drive,cls:'Stock',build:'Saved build'}:{})},maps));
  const preset=presets.find(p=>p.id===presetId);
  const dirty=!!preset&&JSON.stringify(normalize(preset.cfg,maps))!==JSON.stringify(cfg);

  useEffect(()=>{const t=setInterval(()=>setMaps(ms=>ms.some(m=>m.status==='downloading')?ms.map(m=>{
    if(m.status!=='downloading')return m;const p=m.progress+1.6+Math.random()*1.2;
    return p>=100?{...m,status:'installed',progress:0,version:m.latest,done:true}:{...m,progress:p}}):ms),500);return()=>clearInterval(t)},[]);
  useEffect(()=>{const d=maps.find(m=>m.done);if(d){toast({msg:`${d.name} installed — v${d.latest}`,tone:'ok'});setMaps(ms=>ms.map(m=>m.done?{...m,done:false}:m))}},[maps]);

  const onDownload=id=>setMaps(ms=>ms.map(m=>m.id===id?{...m,prev:m.status,status:'downloading',progress:0}:m));
  const onPause=id=>setMaps(ms=>ms.map(m=>m.id===id?{...m,status:m.status==='paused'?'downloading':'paused'}:m));
  const onCancel=id=>setMaps(ms=>ms.map(m=>m.id===id?{...m,status:m.prev||(m.version?'update':'none'),progress:0}:m));

  const map=maps.find(m=>m.id===cfg.map);const car=carById(cfg.car);
  const ok=azOk(settings.azimuth);const playable=map.status==='installed'||map.status==='update';
  const canDrive=ok&&playable&&!libraryBusy&&!garageBusy&&(!window.FHCM_vehicleCatalog||(garage?.map===cfg.map&&garage?.mode===effectiveSaveMode));
  const active=maps.filter(m=>m.status==='downloading'||m.status==='paused');
  const avg=active.length?active.reduce((s,m)=>s+m.progress,0)/active.length:0;

  let msg;
  if(!ok)msg=<Note tone="err">Azimuth not found. <a href="#" onClick={e=>{e.preventDefault();setPanel('settings')}} style={{color:'inherit',textDecoration:'underline'}}>Set the path in Settings</a>.</Note>;
  else if(map.status==='none')msg=<Note tone="warn">{map.name} isn't installed. <a href="#" onClick={e=>{e.preventDefault();onDownload(map.id)}} style={{color:'inherit',textDecoration:'underline'}}>Download {map.size}</a></Note>;
  else if(!playable)msg=<Note tone="info">Downloading {map.name} — {Math.floor(map.progress)}%. You can drive when it's installed.</Note>;
  else if(map.status==='update')msg=<Note tone="info">Update available (v{map.latest}). You can still drive v{map.version}.</Note>;
  else if(cfg.drivatars==='experimental')msg=<Note tone="warn">Experimental AI profiles may behave unpredictably.</Note>;
  else msg=null;

  const drive=()=>{let p=0;setLaunching(1);const t=setInterval(()=>{p+=7;if(p>=100){clearInterval(t);setLaunching(0);
    toast({msg:'Demo only — the game was not launched.',tone:'info'});setRecent(r=>[{when:'Just now',cfg},...r].slice(0,3))}else setLaunching(p)},160)};

  const timeL=cfg.time==='custom'?fmtTime(cfg.customTime):FD.times.find(t=>t[0]===cfg.time)[1];
  const wL=FD.weather.find(w=>w[0]===cfg.weather)[1];
  const trL=map.compat.traffic?(cfg.traffic==='off'?'No traffic':FD.traffic.find(t=>t[0]===cfg.traffic)[1]+' traffic'):'Saved traffic';
  const dvL=cfg.drivatars==='off'?'No Drivatars':`${cfg.count} ${cfg.drivatars==='experimental'?(map.compat.pchRacers?'Tour':FD.profiles.find(p=>p[0]===cfg.profile)[1]):'Standard'} Drivatars`;

  return <div data-theme={theme} data-lt={theme} style={{colorScheme:theme,height:'100vh',minHeight:600,minWidth:1024,display:'grid',gridTemplateRows:'52px minmax(0,1fr) auto 26px',background:'var(--l-bg0)',color:'var(--text-primary)',font:'400 14px/1.5 var(--font-body)',overflow:'hidden'}}>
    <header style={{display:'flex',alignItems:'center',gap:16,padding:'0 16px',borderBottom:'1px solid var(--border-subtle)',background:'var(--l-bg0)',position:'relative',zIndex:20}}>
      <img src={theme==='light'?FD.logoDark:FD.logo} alt="FHCM" style={{height:22,width:'auto',display:'block'}}/>
      <span style={{width:1,height:18,background:'var(--border-default)'}}></span>
      <span style={{...LB,color:'var(--l-ink2)',letterSpacing:'0.16em'}}>Launcher</span>
      <div style={{flex:1}}></div>
      <div style={{width:124}}><Seg height={28} ariaLabel="Appearance" value={mode} onChange={setMode} options={[{v:'dark',label:'Dark',icon:'moon',title:'Dark'},{v:'light',label:'Light',icon:'sun',title:'Light'},{v:'system',label:'System',icon:'monitor',title:'Match system'},{v:'hero',label:'Hero',icon:'earth',title:'World view'}]}/></div>
      <span style={{width:1,height:18,background:'var(--border-default)'}}></span>
      <div style={{position:'relative'}} ref={dlRef}>
        <button onClick={()=>setPanel(p=>p==='downloads'?null:'downloads')} aria-label="Downloads" aria-expanded={panel==='downloads'}
          style={{height:32,display:'inline-flex',alignItems:'center',gap:8,padding:'0 10px',border:'none',background:panel==='downloads'?'var(--l-sel)':'transparent',color:active.length?'var(--l-ink)':'var(--l-ink2)',cursor:'pointer',position:'relative'}}>
          <LIcon name="download" size={16}/>{active.length>0&&<span style={{...MONO,fontSize:11,color:'var(--l-ink)'}}>{active.length} · {Math.floor(avg)}%</span>}
          {active.length>0&&<span style={{position:'absolute',left:10,right:10,bottom:2}}><Bar v={avg} h={2}/></span>}
        </button>
        <DownloadsPopover maps={maps} open={panel==='downloads'} onClose={()=>setPanel(null)} anchor={dlRef} onPause={onPause} onCancel={onCancel} onDownload={onDownload} library={settings.library}/>
      </div>
      <IconBtn icon="settings" label="Settings" active={panel==='settings'} onClick={()=>setPanel(p=>p==='settings'?null:'settings')}/>
      <WinCtl/>
    </header>

    {view==='standard'?<div style={{display:'grid',gridTemplateColumns:(mapsOpen?'clamp(250px,22vw,320px)':'56px')+' minmax(0,1fr) clamp(320px,25vw,360px)',minHeight:0}}>
      {mapsOpen?<MapPanel onLibraryAction={libraryAction} onImportZip={importZip} onManage={()=>setLibraryOpen(true)} libraryBusy={libraryBusy||!!launching} maps={maps} sel={cfg.map} onSelect={id=>set('map',id)} onDownload={onDownload} onPause={onPause} toast={toast} onCollapse={()=>setMapsOpen(false)}/>
        :<MapRail maps={maps} sel={cfg.map} onSelect={id=>set('map',id)} onExpand={()=>setMapsOpen(true)}/>}
      <main style={{minWidth:0,minHeight:0,overflow:'auto',padding:20,display:'flex',flexDirection:'column',gap:24,background:'var(--l-bg1)'}}>
        <MapHero m={map} onDownload={onDownload} onPause={onPause}/>
        <VehiclePanel cfg={cfg} set={set} saveMode={effectiveSaveMode} setSaveMode={setSaveMode} garageBusy={garageBusy} garage={garage}/>
      </main>
      <SessionPanel cfg={cfg} set={set} map={map} presets={presets} presetId={presetId} dirty={dirty}
        onLoadPreset={p=>{if(!maps.some(m=>m.id===p.cfg.map)){toast({msg:'Restore this map from Manage maps before loading its preset.',tone:'err'});return;}setPresetId(p.id);applyCfg(p.cfg)}}
        onSavePreset={()=>{setPresets(ps=>ps.map(p=>p.id===presetId?{...p,cfg}:p));toast({msg:`Saved “${preset.name}”`,tone:'ok'})}}
        onSaveAs={name=>{const id='u'+Date.now();setPresets(ps=>[...ps,{id,name,cfg}]);setPresetId(id);toast({msg:`Saved “${name}”`,tone:'ok'})}}
        recent={recent} onLoadRecent={r=>{applyCfg(r.cfg);setPresetId(null)}}/>
    </div>
    :<div style={{position:'relative',minHeight:0,overflow:'hidden'}}>
      <MapHero full m={map} onDownload={onDownload} onPause={onPause}/>
      <div role="toolbar" aria-label="Panels" aria-orientation="vertical" style={{position:'absolute',top:16,right:16,zIndex:10,display:'flex',flexDirection:'column',background:'var(--l-bg0)',border:'1px solid var(--border-default)'}}>
        {[['maps','map','Maps'],['vehicle','car','Vehicle'],['session','sliders-horizontal','Session']].map(([k,ic,l])=><OverlayBtn key={k} icon={ic} label={l} active={overlay===k} onClick={()=>setOverlay(o=>o===k?null:k)}/>)}
      </div>
      {overlay&&<div role="dialog" aria-label={overlay} style={{position:'absolute',top:16,right:68,bottom:16,zIndex:9,width:overlay==='vehicle'?400:340,maxWidth:'calc(100% - 100px)',display:'flex',flexDirection:'column',background:'var(--l-bg0)',border:'1px solid var(--border-default)',boxShadow:'var(--shadow-lg)'}}>
        {overlay==='maps'&&<MapPanel onLibraryAction={libraryAction} onImportZip={importZip} onManage={()=>setLibraryOpen(true)} libraryBusy={libraryBusy||!!launching} maps={maps} sel={cfg.map} onSelect={id=>set('map',id)} onDownload={onDownload} onPause={onPause} toast={toast} onCollapse={()=>setOverlay(null)} collapseIcon="x"/>}
        {overlay==='vehicle'&&<div style={{flex:1,minHeight:0,overflow:'auto',padding:16,position:'relative'}}><div style={{position:'absolute',top:8,right:8}}><IconBtn icon="x" label="Close" onClick={()=>setOverlay(null)}/></div><VehiclePanel cfg={cfg} set={set} saveMode={effectiveSaveMode} setSaveMode={setSaveMode} garageBusy={garageBusy} garage={garage}/></div>}
        {overlay==='session'&&<SessionPanel cfg={cfg} set={set} map={map} presets={presets} presetId={presetId} dirty={dirty}
        onLoadPreset={p=>{setPresetId(p.id);applyCfg(p.cfg)}}
        onSavePreset={()=>{setPresets(ps=>ps.map(p=>p.id===presetId?{...p,cfg}:p));toast({msg:`Saved “${preset.name}”`,tone:'ok'})}}
        onSaveAs={name=>{const id='u'+Date.now();setPresets(ps=>[...ps,{id,name,cfg}]);setPresetId(id);toast({msg:`Saved “${name}”`,tone:'ok'})}}
        recent={recent} onLoadRecent={r=>{applyCfg(r.cfg);setPresetId(null)}}/>}
      </div>}
    </div>}

    <footer style={{display:'flex',alignItems:'center',gap:20,padding:'14px 16px 14px 0',borderTop:'1px solid var(--border-default)',background:'var(--l-bg0)'}}>
      <div style={{flex:1,minWidth:0,display:'grid',gridTemplateColumns:'repeat(3,minmax(0,1fr))'}}>
        <Summary label="Map" a={map.name} b={map.original?'Original game · Azimuth':map.region}/>
        <Summary label="Vehicle" a={carName(car)} b=""/>
        <Summary label="Session" a={`${timeL} · ${wL}`} b={`${trL} · ${dvL} · HUD ${FD.hud.find(h=>h[0]===cfg.hud)[1]}`}/>
      </div>
      {msg&&<div style={{width:250,flexShrink:0}}>{msg}</div>}
      <DriveButton disabled={!canDrive} launching={launching} onClick={drive}/>
    </footer>

    <div style={{display:'flex',alignItems:'center',justifyContent:'space-between',gap:16,padding:'0 16px',borderTop:'1px solid var(--border-subtle)',background:'var(--l-chrome)',...MONO,fontSize:11}}>
      <span style={{marginLeft:'auto'}}>FHCM v{FD.version}</span>
    </div>

    <SettingsDrawer open={panel==='settings'} onClose={()=>setPanel(null)} settings={settings} setSettings={setSettings} version={FD.version}/>
    {libraryOpen&&<LibraryManager error={libraryError} maps={maps} removed={removedMaps} busy={libraryBusy||!!launching} onAction={libraryAction} onExport={exportZip} onImport={importZip} onClose={()=>setLibraryOpen(false)}/>}
    {libraryBusy&&<div role="status" style={{position:'fixed',top:62,left:'50%',transform:'translateX(-50%)',padding:'10px 16px',background:'var(--l-bg2)',border:'1px solid var(--border-default)',zIndex:100}}>Map library operation in progress…</div>}
    {toastMsg&&<div key={toastMsg.k} role="status" style={{position:'fixed',left:'50%',bottom:118,transform:'translateX(-50%)',zIndex:70,display:'flex',alignItems:'center',gap:10,padding:'10px 16px',background:'var(--l-bg2)',border:'1px solid var(--border-default)',boxShadow:'var(--shadow-md)'}}>
      <Note tone={toastMsg.tone} style={{fontSize:13,color:'var(--l-ink)'}}>{toastMsg.msg}</Note></div>}
  </div>;
}
ReactDOM.createRoot(document.getElementById('root')).render(<App/>);



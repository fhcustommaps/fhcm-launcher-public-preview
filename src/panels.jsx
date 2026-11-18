/* FHCM Launcher — map, vehicle and session panels */
const FD=window.FHL;
const carById=id=>FD.cars.find(c=>c.id===id)||FD.cars[0];
const carName=c=>`${c.year} ${c.make} ${c.model}`;
const PI_FALLBACK=[['D',100,500,'var(--pi-d)','#0E0F11'],['C',501,600,'var(--pi-c)','#0E0F11'],['B',601,700,'var(--pi-b)','#0E0F11'],['A',701,800,'var(--pi-a)','#fff'],['S1',801,900,'var(--pi-s1)','#fff'],['S2',901,998,'var(--pi-s2)','#fff'],['X',999,999,'var(--pi-x)','#fff']];
function LocalPIBadge({cls,pi,size='md'}){
  const c=PI_FALLBACK.find(x=>cls?x[0]===cls:pi>=x[1]&&pi<=x[2])||PI_FALLBACK[0];const s={sm:[18,11,22,30],md:[24,14,28,38],lg:[32,19,38,50]}[size];
  return <span style={{display:'inline-flex',alignItems:'stretch',height:s[0],flexShrink:0,boxSizing:'border-box',border:'1px solid '+c[3],background:'#0E0F11'}}>
    <span style={{minWidth:s[2]-2,display:'inline-flex',alignItems:'center',justifyContent:'center',padding:'0 3px',background:c[3],color:c[4],font:`italic 800 ${s[1]}px/1 var(--font-display)`}}>{c[0]}</span>
    {pi!=null&&<span style={{minWidth:s[3],display:'inline-flex',alignItems:'center',justifyContent:'center',padding:'0 5px',color:'#fff',font:`600 ${s[1]-1}px/1 var(--font-display)`,fontVariantNumeric:'tabular-nums'}}>{pi}</span>}
  </span>;
}
const PIBadge=window.FHCMDesignSystem_b88bf6.PIBadge||LocalPIBadge;
const effPI=(car,cls)=>cls==='Stock'?{pi:car.pi}:{cls,pi:FD.classMax[cls]};
const opt=(arr)=>arr.map(([v,label])=>({v,label}));

/* ---------- Maps ---------- */
function MapStatus({m,onDownload,onPause}){
  const stop=f=>e=>{e.stopPropagation();f()};
  if(m.original||m.status==='installed')return <span title={m.original?'Original game':'Installed · v'+m.version} style={{width:26,display:'inline-flex',justifyContent:'center'}}><LIcon name="circle-check" size={14} color="var(--status-success)"/></span>;
  if(m.status==='update')return <IconBtn size={26} iconSize={14} icon="refresh-cw" color="var(--fh-gold)" label={'Update to v'+m.latest} onClick={stop(()=>onDownload(m.id))}/>;
  if(m.status==='downloading'||m.status==='paused')return <IconBtn size={26} iconSize={13} icon={m.status==='paused'?'play':'pause'} label={m.status==='paused'?'Resume':'Pause'} onClick={stop(()=>onPause(m.id))}/>;
  return <IconBtn size={26} iconSize={14} icon="download" label={'Download '+m.name+' · '+m.size} onClick={stop(()=>onDownload(m.id))}/>;
}
function mapSub(m){
  if(m.original)return ['Via Azimuth'];
  if(m.status==='update')return ['Update · v'+m.latest,'var(--fh-gold)'];
  if(m.status==='downloading')return ['Downloading · '+Math.floor(m.progress)+'%','var(--l-ink)'];
  if(m.status==='paused')return ['Paused · '+Math.floor(m.progress)+'%'];
  if(m.status==='none')return ['Not installed · '+m.size];
  return [m.region];
}
function MapRow({m,sel,onSelect,onDownload,onPause,onLibraryAction,libraryBusy}){
  const [h,hp]=useHover();const dragStart=useRef(null);const dl=m.status==='downloading'||m.status==='paused';
  return <div data-map-id={m.id} role="option" aria-selected={sel} title={m.name+' | '+m.region} tabIndex={0} onClick={()=>onSelect(m.id)} onKeyDown={e=>{if(e.key==='Enter'||e.key===' '){e.preventDefault();onSelect(m.id)}}} {...hp}
    style={{position:'relative',display:'grid',gridTemplateColumns:'60px minmax(0,1fr) auto',columnGap:10,rowGap:8,alignItems:'center',padding:'10px 12px 10px 16px',background:sel?'var(--l-bg2)':h?'var(--l-hov)':'transparent',cursor:'pointer',outlineOffset:-2}}>
    {sel&&<span style={{position:'absolute',left:0,top:0,bottom:0,width:3,background:'var(--gradient-horizon)'}}></span>}
    <Thumb tone={m.tone} img={m.img} style={{width:60,height:38,display:'flex',alignItems:'center',justifyContent:'center',color:'rgba(255,255,255,0.6)'}}>{m.original&&<LIcon name="gamepad-2" size={16}/>}</Thumb>
    <div style={{minWidth:0}}>
      <div style={{font:'italic 700 14px/1.2 var(--font-display)',textTransform:'uppercase',color:m.status==='none'&&!sel?'var(--l-ink2)':'var(--l-ink)',...ELL}}>{m.name}</div>
      <div style={{font:'400 12px/1.35 var(--font-body)',color:mapSub(m)[1]||'var(--text-muted)',...ELL}}>{mapSub(m)[0]}</div>
    </div>
    <MapStatus m={m} onDownload={onDownload} onPause={onPause}/>
    {!m.original&&onLibraryAction&&<div style={{gridColumn:'1 / -1',display:'flex',alignItems:'center',justifyContent:'flex-end',gap:3}}><button type="button" aria-label={'Drag to reorder '+m.name} disabled={libraryBusy} onClick={e=>e.stopPropagation()} onPointerDown={e=>{e.preventDefault();e.stopPropagation();dragStart.current={x:e.clientX,y:e.clientY};e.currentTarget.setPointerCapture(e.pointerId);}} onPointerUp={e=>{e.preventDefault();e.stopPropagation();const start=dragStart.current;dragStart.current=null;try{e.currentTarget.releasePointerCapture(e.pointerId);}catch{}if(!start||Math.hypot(e.clientX-start.x,e.clientY-start.y)<8)return;const target=document.elementFromPoint(e.clientX,e.clientY)?.closest('[data-map-id]')?.dataset.mapId;if(target&&target!=='fh2'&&target!==m.id)onLibraryAction(m.id,'before',target);}} onPointerCancel={()=>{dragStart.current=null;}} style={{marginRight:'auto',...MONO,fontSize:10,border:0,padding:0,background:'transparent',color:'var(--text-muted)',cursor:'grab',touchAction:'none'}}>Drag to reorder</button>{[['star',m.favorite?'Unfavorite map':'Favorite map','favorite'],['arrow-up','Move map up','up'],['arrow-down','Move map down','down'],['package-minus','Remove from installed maps','remove']].map(([icon,label,action])=><button key={action} type="button" aria-label={label+' · '+m.name} title={label} disabled={libraryBusy} onClick={e=>{e.stopPropagation();onLibraryAction(m.id,action);}} style={{display:'inline-flex',alignItems:'center',justifyContent:'center',width:26,height:24,border:0,background:'transparent',color:action==='favorite'&&m.favorite?'var(--fh-gold)':'var(--l-ink2)',cursor:'pointer'}}><LIcon name={icon} size={13}/></button>)}</div>}
    {dl&&<div style={{gridColumn:'1 / -1'}}><Bar v={m.progress} paused={m.status==='paused'}/></div>}
  </div>;
}
function MapPanel({maps,sel,onSelect,onDownload,onPause,toast,onCollapse,collapseIcon='panel-left-close',onLibraryAction,onImportZip,onManage,libraryBusy}){
  const [q,setQ]=useState('');const [f,setF]=useState('all');
  const orig=maps.find(m=>m.original);
  const list=maps.filter(m=>!m.original&&(f==='favorites'?m.favorite:(f==='all'||m.status==='installed'||m.status==='update'))&&(m.name+' '+m.region).toLowerCase().includes(q.trim().toLowerCase()));
  const installed=maps.filter(m=>!m.original&&(m.status==='installed'||m.status==='update')).length;
  return <aside aria-label="Maps" style={{display:'flex',flexDirection:'column',minHeight:0,flex:1,borderRight:'1px solid var(--border-subtle)',background:'var(--l-bg0)'}}>
    <div style={{padding:'18px 16px 12px',display:'flex',flexDirection:'column',gap:12}}>
      <div style={{display:'flex',alignItems:'center',justifyContent:'space-between',gap:8}}>
        <span style={{display:'flex',alignItems:'center',gap:2,marginLeft:-8}}>{onCollapse&&<IconBtn size={28} iconSize={15} icon={collapseIcon} label={collapseIcon==='x'?'Close':'Collapse maps'} onClick={onCollapse}/>}<span style={{...LB,color:'var(--l-ink)',whiteSpace:'nowrap'}}>Maps <span style={{...MONO,fontSize:11,marginLeft:4}}>{installed}/{maps.length-1}</span></span></span>
        <div style={{width:150}}><Seg height={26} ariaLabel="Filter maps" value={f} onChange={setF} options={[{v:'all',label:'All'},{v:'favorites',label:'Favorites'}]}/></div>
      </div>
      {onImportZip&&<div style={{display:'flex',gap:8}}><SmallBtn icon="package-plus" disabled={libraryBusy} onClick={onImportZip}>Import ZIP</SmallBtn><SmallBtn icon="library" onClick={onManage}>Manage maps</SmallBtn></div>}
      <SearchInput value={q} onChange={setQ} placeholder="Search maps"/>
    </div>
    <div role="listbox" aria-label="Map library" style={{flex:1,overflow:'auto',minHeight:0}}>
      <MapRow m={orig} sel={sel===orig.id} onSelect={onSelect}/>
      <div style={{...LB,padding:'16px 16px 6px',borderTop:'1px solid var(--border-subtle)',marginTop:6}}>FHCM maps</div>
      {list.map(m=><MapRow onLibraryAction={onLibraryAction} libraryBusy={libraryBusy} key={m.id} m={m} sel={sel===m.id} onSelect={onSelect} onDownload={onDownload} onPause={onPause}/>)}
      {list.length===0&&<div style={{padding:'12px 16px',font:'400 13px/1.5 var(--font-body)',color:'var(--text-muted)'}}>No maps match{q?` “${q}”`:' this filter'}.</div>}
    </div>
    <div style={{display:'flex',alignItems:'center',justifyContent:'space-between',gap:8,padding:'10px 12px 10px 16px',borderTop:'1px solid var(--border-subtle)'}}>
      <span style={{...MONO,fontSize:11}}>Local map library</span>
      <IconBtn size={28} iconSize={14} icon="refresh-cw" label="Local library info" onClick={onManage||(()=>toast({msg:'Local library loaded.',tone:'ok'}))}/>
    </div>
  </aside>;
}

function MapRail({maps,sel,onSelect,onExpand}){
  return <aside aria-label="Maps" style={{display:'flex',flexDirection:'column',alignItems:'center',minHeight:0,borderRight:'1px solid var(--border-subtle)',background:'var(--l-bg0)'}}>
    <div style={{padding:'12px 0 8px'}}><IconBtn icon="panel-left-open" label="Expand maps" onClick={onExpand}/></div>
    <div role="listbox" aria-label="Map library" style={{flex:1,minHeight:0,overflow:'auto',display:'flex',flexDirection:'column',alignItems:'center',gap:6,padding:'4px 0 12px',width:'100%'}}>
      {maps.map((m,i)=>{const s=sel===m.id;const busy=m.status==='downloading'||m.status==='paused';
        return <React.Fragment key={m.id}>{i===1&&<span style={{width:24,height:1,background:'var(--border-subtle)',margin:'4px 0'}}></span>}
        <button role="option" aria-selected={s} title={m.name+(m.status==='none'?' · Not installed':m.status==='update'?' · Update available':busy?' · '+Math.floor(m.progress)+'%':'')} onClick={()=>onSelect(m.id)}
          style={{position:'relative',width:36,height:36,padding:0,border:'none',outline:s?'2px solid var(--l-ink)':'none',outlineOffset:2,cursor:'pointer',background:'none',opacity:m.status==='none'&&!s?0.5:1}}>
          <Thumb tone={m.tone} img={m.img} style={{width:36,height:36,display:'flex',alignItems:'center',justifyContent:'center',font:'italic 800 12px/1 var(--font-display)',color:'#fff'}}>{m.original?<LIcon name="gamepad-2" size={15}/>:!m.img&&m.name.slice(0,2).toUpperCase()}</Thumb>
          {m.status==='update'&&<span style={{position:'absolute',top:-2,right:-2,width:7,height:7,background:'var(--fh-gold)'}}></span>}
          {busy&&<span style={{position:'absolute',left:0,right:0,bottom:0}}><Bar v={m.progress} h={3} paused={m.status==='paused'}/></span>}
        </button></React.Fragment>})}
    </div>
  </aside>;
}

/* ---------- Map hero ---------- */
function MapHero({m,onDownload,onPause,full}){
  const meta=m.original?['Open world','Your existing save']:[m.length,m.format,m.type,m.status==='installed'||m.status==='update'?'v'+m.version:'v'+m.latest,m.size];
  let action=null;
  if(m.status==='none')action=<SmallBtn icon="download" onClick={()=>onDownload(m.id)}>Download · {m.size}</SmallBtn>;
  if(m.status==='update')action=<SmallBtn tone="gold" icon="refresh-cw" onClick={()=>onDownload(m.id)}>Update to v{m.latest}</SmallBtn>;
  if(m.status==='downloading'||m.status==='paused')action=<div style={{display:'flex',alignItems:'center',gap:10,width:220}}><div style={{flex:1}}><Bar v={m.progress} h={3} paused={m.status==='paused'}/></div><span style={{...MONO,color:'#fff'}}>{Math.floor(m.progress)}%</span><IconBtn size={28} iconSize={14} icon={m.status==='paused'?'play':'pause'} label={m.status==='paused'?'Resume':'Pause'} onClick={()=>onPause(m.id)}/></div>;
  return <section aria-label="Selected map" style={full?{position:'absolute',inset:0}:{position:'relative',flex:'1 1 240px',minHeight:210,clipPath:'var(--clip-cut-md)'}}>
    <Thumb tone={m.tone} img={m.img} label="MAP STILL · PLACEHOLDER" style={{position:'absolute',inset:0}}></Thumb>
    <div style={{position:'absolute',inset:0,background:'var(--scrim-left)'}}></div>
    <div style={{position:'absolute',left:full?32:28,right:full?84:28,bottom:full?32:24,display:'flex',alignItems:'flex-end',justifyContent:'space-between',gap:24,flexWrap:'wrap'}}>
      <div style={{display:'flex',flexDirection:'column',gap:8,maxWidth:520}}>
        <span style={{...LB,color:'var(--neutral-300)'}}>{m.original?'Original game':m.region}</span>
        <h1 style={{margin:0,font:'italic 800 clamp(30px,3.2vw,44px)/0.95 var(--font-display)',textTransform:'uppercase',letterSpacing:'-0.01em',color:'#fff'}}>{m.name}</h1>
        <p style={{margin:0,font:'400 14px/1.5 var(--font-body)',color:'var(--neutral-200)',textWrap:'pretty'}}>{m.blurb}</p>
        <div style={{display:'flex',flexWrap:'wrap',gap:'4px 0',marginTop:2}}>{meta.filter(Boolean).map((t,i)=><span key={i} style={{...MONO,whiteSpace:'nowrap',color:'var(--neutral-300)',padding:'0 10px',borderLeft:i?'1px solid rgba(255,255,255,0.2)':'none',paddingLeft:i?10:0}}>{t}</span>)}</div>
      </div>
      {action}
    </div>
  </section>;
}

/* ---------- Vehicle ---------- */
function VehiclePicker({car,onPick}){
  const [open,setOpen]=useState(false);const [q,setQ]=useState('');const [ai,setAi]=useState(0);const ref=useRef();
  const rect=ref.current?.getBoundingClientRect();const bounds=ref.current?.closest('main,[role="dialog"]')?.getBoundingClientRect();
  const below=(bounds?.bottom||window.innerHeight-114)-(rect?.bottom||0)-64;const above=(rect?.top||0)-(bounds?.top||52)-64;
  const menuAbove=below<150&&above>below;const menuHeight=Math.max(80,Math.min(300,menuAbove?above:below));
  useOutside(ref,open,()=>setOpen(false));
  const list=FD.cars.filter(c=>(carName(c)).toLowerCase().includes(q.trim().toLowerCase())).sort((a,b)=>a.make.localeCompare(b.make)||a.year-b.year);
  useEffect(()=>setAi(0),[q,open]);
  const pick=c=>{onPick(c.id);setOpen(false);setQ('')};
  const key=e=>{if(e.key==='ArrowDown'){e.preventDefault();setAi(i=>Math.min(list.length-1,i+1))}else if(e.key==='ArrowUp'){e.preventDefault();setAi(i=>Math.max(0,i-1))}else if(e.key==='Enter'&&list[ai])pick(list[ai])};
  return <div ref={ref} style={{position:'relative'}}>
    <button onClick={()=>setOpen(o=>!o)} aria-haspopup="listbox" aria-expanded={open}
      style={{width:'100%',height:56,display:'flex',alignItems:'center',gap:12,padding:'0 12px 0 14px',background:'var(--surface-sunken)',border:'1px solid '+(open?'var(--border-strong)':'var(--border-default)'),color:'var(--l-ink)',cursor:'pointer',textAlign:'left'}}>
      <span style={{flex:1,minWidth:0,display:'flex',flexDirection:'column',gap:4}}>
        <span style={{...LB,fontSize:10.5}}>{car.year} · {car.make}</span>
        <span style={{font:'600 17px/1.1 var(--font-display)',...ELL}}>{car.model}</span>
      </span>
      <PIBadge pi={car.pi}/>
      <LIcon name="chevron-down" size={16} style={{color:'var(--text-muted)',transform:open?'rotate(180deg)':'none',transition:'transform var(--dur-fast)'}}/>
    </button>
    {open&&<Menu style={{padding:0,...(menuAbove?{top:'auto',bottom:'calc(100% + 4px)'}:{})}}>
      <div style={{padding:8,borderBottom:'1px solid var(--border-subtle)'}}><SearchInput autoFocus value={q} onChange={setQ} onKeyDown={key} placeholder="Search make, model or year"/></div>
      <div style={{maxHeight:menuHeight,overflow:'auto',padding:4}}>
        {list.map((c,i)=>{const head=i===0||list[i-1].make!==c.make;return <React.Fragment key={c.id}>
          {head&&<div style={{...LB,fontSize:10,padding:'10px 8px 4px'}}>{c.make}</div>}
          <MenuItem sel={c.id===car.id} active={i===ai} hint={<span style={{display:'inline-flex',alignItems:'center',gap:10}}><PIBadge pi={c.pi} size="sm"/></span>} onClick={()=>pick(c)}><span style={{display:'inline-flex',alignItems:'center',gap:8}}>{c.img&&<img src={c.img} alt="" style={{width:54,height:32,objectFit:'contain',flexShrink:0,display:'block'}}/>}<span>{c.model} <span style={{color:'var(--text-muted)'}}>{c.year}{c.garageId?' · #'+c.garageId:''}</span></span></span></MenuItem>
        </React.Fragment>})}
        {list.length===0&&<div style={{padding:'10px 8px',font:'400 13px var(--font-body)',color:'var(--text-muted)'}}>No vehicles match “{q}”.</div>}
      </div>
    </Menu>}
  </div>;
}
function Swatch({c,sel,onClick}){
  return <button aria-label={c[0]} title={c[0]} aria-pressed={sel} onClick={onClick}
    style={{width:20,height:20,padding:0,background:c[1],border:'1px solid var(--border-strong)',outline:sel?'1.5px solid var(--fh-pink)':'none',outlineOffset:2,cursor:'pointer'}}></button>;
}
function VehiclePanelContent({cfg,set,saveMode='personal',setSaveMode,garageBusy=false,garage}){
  const [paletteOpen,setPaletteOpen]=useState(false);
  useEffect(()=>setPaletteOpen(false),[cfg.car,cfg.map]);
  if(window.FHCM_vehicleCatalog&&FD.garageMap!==cfg.map)return <section aria-label="Vehicle">{setSaveMode&&<Seg ariaLabel="Save mode" value={saveMode} onChange={setSaveMode} disabled={garageBusy||cfg.map==='fh2'} options={[{v:'personal',label:'Personal garage'},{v:'sandbox',label:'FHCM Sandbox'}]}/>}<Note>{garageBusy?'Loading the selected save’s garage…':'Garage unavailable. Choose Personal garage or reopen the launcher to retry.'}</Note></section>;
  const car=carById(cfg.car);const paint=car.colors.find(c=>c[0]===cfg.paint)||car.colors[0];
  return <section aria-label="Vehicle" style={{display:'flex',flexDirection:'column',gap:14}}>
    <span style={{...LB,color:'var(--l-ink)'}}>Vehicle · {saveMode==='sandbox'?'FHCM Sandbox':'personal garage'}</span>
    {setSaveMode&&<div style={{display:'flex',flexDirection:'column',gap:6}}><Seg ariaLabel="Save mode" value={saveMode} onChange={setSaveMode} disabled={garageBusy||cfg.map==='fh2'} options={[{v:'personal',label:'Personal garage'},{v:'sandbox',label:'FHCM Sandbox',title:'Separate save with supported installed cars and maximum credits'}]}/></div>}
    <div style={{display:'grid',gridTemplateColumns:'repeat(auto-fit,minmax(280px,1fr))',gap:20}}>
      <div style={{minWidth:0}}><Thumb tone="#6B6E78" label={car.img?null:"VEHICLE RENDER · PLACEHOLDER"} style={{aspectRatio:'1 / 1',minWidth:0,border:'1px solid var(--border-subtle)',display:'flex',flexDirection:'column'}}>
        <div style={{flex:'1 1 0',minHeight:0,position:'relative',margin:'12px 16px 0',overflow:'hidden'}}>
          {car.img&&<img src={car.img} alt={carName(car)} style={{position:'absolute',inset:0,width:'100%',height:'100%',objectFit:'contain',objectPosition:'center',display:'block'}}/>}
        </div>
        <div style={{display:'flex',flexDirection:'column',alignItems:'stretch',gap:10,padding:'12px 16px 18px',position:'relative'}}>
          <div style={{display:'flex',flexDirection:'column',gap:4,minWidth:0}}>
            <span style={{...MONO,fontSize:11,color:'var(--neutral-300)'}}>{car.year} {car.make}</span>
            <span style={{font:'italic 800 24px/1 var(--font-display)',textTransform:'uppercase',color:'#fff',...ELL}}>{car.model}</span>
            <span style={{display:'flex',alignItems:'center',gap:10,marginTop:4}}><PIBadge size="lg" {...effPI(car,cfg.cls)}/></span>
          </div>
          <div style={{display:'flex',alignItems:'center',justifyContent:'space-between',gap:8,minWidth:0}}>
            <span style={{font:'500 12px/1 var(--font-body)',color:'var(--neutral-200)',...ELL}}>{cfg.paint==='Saved appearance'?'':paint[0]}</span>
            {car.colors.length>1&&<button type="button" aria-expanded={paletteOpen} aria-controls="vehicle-palette" onClick={()=>setPaletteOpen(v=>!v)} style={{display:'flex',alignItems:'center',flexShrink:0,gap:6,border:'1px solid #666',background:'#15161A',color:'#fff',padding:'5px 8px',cursor:'pointer'}}><span style={{width:14,height:14,background:paint[1],border:'1px solid #888'}}/><span>Paint colors</span><LIcon name={paletteOpen?'chevron-up':'chevron-down'} size={12}/></button>}
          </div>
        </div>
        <div style={{height:4,background:paint[1],transition:'background var(--dur-base)'}}></div>
      </Thumb>
      {paletteOpen&&car.colors.length>1&&<div id="vehicle-palette" style={{padding:12,border:'1px solid var(--border-subtle)',background:'var(--l-bg1)',maxHeight:180,overflowY:'auto'}}><div style={{...LB,marginBottom:10}}>Factory paint · {paint[0]}</div><div role="radiogroup" aria-label="Factory paint" style={{display:'grid',gridTemplateColumns:'repeat(auto-fill,minmax(28px,1fr))',gap:10,padding:3}}>{car.colors.map(c=><Swatch key={c[0]} c={c} sel={c===paint} onClick={()=>set('paint',c[0])}/>)}</div></div>}
      </div>
      <div style={{display:'flex',flexDirection:'column',minWidth:0}}>
        <VehiclePicker car={car} onPick={id=>set('car',id)}/>
        <div style={{marginTop:6}}>
          <Row label="Performance"><PIBadge pi={car.pi}/></Row>
        </div>
      </div>
    </div>
  </section>;
}

/* ---------- Session ---------- */
const lbl=(arr,v)=>(arr.find(x=>x[0]===v)||[,''])[1];
const TIME_IC={morning:'sunrise',noon:'sun',sunset:'sunset',night:'moon',custom:'clock'};
const WX_IC={clear:'sun',wet:'cloud',rain:'cloud-rain',storm:'cloud-lightning'};
const TR_IC={off:'signal-zero',light:'signal-low',medium:'signal-medium',heavy:'signal-high'};
const DV_IC={off:'user-x',standard:'users',experimental:'flask-conical'};
const HUD_IC={full:'gauge',minimal:'minimize-2',nomap:'map-pin-off',off:'eye-off'};
function PresetMenu({presets,presetId,dirty,onLoad,onSave,onSaveAs}){
  const [open,setOpen]=useState(false);const [naming,setNaming]=useState(false);const [name,setName]=useState('');const anchor=useRef();
  const cur=presets.find(p=>p.id===presetId);
  const close=()=>{setOpen(false);setNaming(false);setName('')};
  const submit=()=>{if(name.trim()){onSaveAs(name.trim());close()}};
  return <div style={{position:'relative'}}>
    <button ref={anchor} onClick={()=>open?close():setOpen(true)} aria-haspopup="menu" aria-expanded={open}
      style={{height:28,display:'inline-flex',alignItems:'center',gap:6,maxWidth:200,padding:'0 8px 0 10px',border:'1px solid var(--border-default)',background:open?'var(--l-sel)':'transparent',color:'var(--l-ink)',font:'500 12px/1 var(--font-body)',cursor:'pointer'}}>
      <LIcon name="bookmark" size={13} style={{color:'var(--text-muted)'}}/><span style={ELL}>{cur?cur.name:'Unsaved session'}</span>
      {dirty&&cur&&<span title="Modified" style={{width:6,height:6,background:'var(--fh-gold)',flexShrink:0}}></span>}
      <LIcon name="chevron-down" size={13} style={{color:'var(--text-muted)'}}/>
    </button>
    <Popover anchorRef={anchor} open={open} onClose={close} width={280}>
      <div style={{...LB,padding:'12px 12px 6px'}}>Presets</div>
      <div style={{padding:'0 4px 4px'}}>{presets.map(p=>{const m=FD.maps.find(x=>x.id===p.cfg.map),c=carById(p.cfg.car);
        return <MenuItem disabled={!m} key={p.id} sel={p.id===presetId} onClick={()=>{onLoad(p);close()}}><div style={{display:'flex',flexDirection:'column',gap:2}}><span>{p.name}</span><span style={{...MONO,fontSize:11}}>{m?.name||'Removed map'} · {c.make} {c.model}</span></div></MenuItem>})}</div>
      <div style={{borderTop:'1px solid var(--border-subtle)',padding:4}}>
        {cur&&<MenuItem icon="save" disabled={!dirty} onClick={()=>{onSave();close()}}>Save changes to “{cur.name}”</MenuItem>}
        {!naming?<MenuItem icon="plus" onClick={()=>setNaming(true)}>Save as new preset…</MenuItem>:
        <div style={{display:'flex',gap:6,padding:4}}>
          <input autoFocus value={name} onChange={e=>setName(e.target.value)} onKeyDown={e=>e.key==='Enter'&&submit()} placeholder="Preset name" aria-label="Preset name"
            style={{flex:1,minWidth:0,height:28,padding:'0 8px',background:'var(--surface-sunken)',border:'1px solid var(--border-strong)',color:'var(--l-ink)',font:'400 13px var(--font-body)',outline:'none'}}/>
          <SmallBtn onClick={submit} disabled={!name.trim()}>Save</SmallBtn>
        </div>}
      </div>
    </Popover>
  </div>;
}
function RecentRow({r,onLoad}){
  const [h,hp]=useHover();const m=FD.maps.find(x=>x.id===r.cfg.map),c=carById(r.cfg.car);
  const t=r.cfg.time==='custom'?fmtTime(r.cfg.customTime):FD.times.find(x=>x[0]===r.cfg.time)[1];
  const w=FD.weather.find(x=>x[0]===r.cfg.weather)[1];
  return <button onClick={()=>onLoad(r)} {...hp} title="Load this configuration"
    style={{display:'grid',gridTemplateColumns:'minmax(0,1fr) auto',alignItems:'center',gap:8,width:'100%',padding:'8px 16px',border:'none',background:h?'var(--l-hov)':'transparent',color:'var(--l-ink)',cursor:'pointer',textAlign:'left'}}>
    <span style={{minWidth:0,display:'flex',flexDirection:'column',gap:3}}>
      <span style={{font:'500 13px/1.2 var(--font-body)',...ELL}}>{m.name} · {c.make} {c.model}</span>
      <span style={{...MONO,fontSize:11,...ELL}}>{t} · {w} · {r.when}</span>
    </span>
    <LIcon name="arrow-right" size={14} style={{color:h?'var(--l-ink)':'var(--l-ink3)',transform:h?'translateX(2px)':'none',transition:'transform var(--dur-base) var(--ease-out)'}}/>
  </button>;
}
function SessionPanelContent({cfg,set,map,presets,presetId,dirty,onLoadPreset,onSavePreset,onSaveAs,recent,onLoadRecent}){
  const C=map.compat;const max=FD.limits[cfg.drivatars]||12;const exp=cfg.drivatars==='experimental';
  return <aside aria-label="Session" style={{display:'flex',flexDirection:'column',minHeight:0,borderLeft:'1px solid var(--border-subtle)',background:'var(--l-bg0)'}}>
    <div style={{display:'flex',alignItems:'center',justifyContent:'space-between',gap:12,padding:'14px 16px',borderBottom:'1px solid var(--border-subtle)'}}>
      <span style={{...LB,color:'var(--l-ink)'}}>Session</span>
      <PresetMenu presets={presets} presetId={presetId} dirty={dirty} onLoad={onLoadPreset} onSave={onSavePreset} onSaveAs={onSaveAs}/>
    </div>
    <div style={{flex:1,minHeight:0,overflow:'auto',padding:'0 16px 16px'}}>
      <Group label={C.conditions?"Conditions · applied at launch":"Conditions · saved configuration"}>
        <Row label="Time" value={cfg.time==='custom'?fmtTime(cfg.customTime):lbl(FD.times,cfg.time)}>
          <IconSeg ariaLabel="Time of day" disabled={!C.conditions} value={cfg.time} onChange={v=>set('time',v)} options={FD.times.map(([v,label])=>({v,label,title:label,icon:TIME_IC[v]}))}/>
        </Row>
        {C.conditions&&cfg.time==='custom'&&<Row indent label="Custom"><input type="range" min={0} max={1425} step={15} value={cfg.customTime} onChange={e=>set('customTime',+e.target.value)} aria-label="Custom time" style={{width:150,accentColor:'var(--fh-pink)',margin:0}}/></Row>}
        <Row label="Weather" value={lbl(FD.weather,cfg.weather)}>
          <IconSeg ariaLabel="Weather" disabled={!C.conditions} value={cfg.weather} onChange={v=>set('weather',v)} options={FD.weather.map(([v,label])=>({v,label,icon:WX_IC[v],disabled:!C.weather.includes(v),title:C.weather.includes(v)?label:label+' — not supported on this map'}))}/>
        </Row>
      </Group>
      <Group label={C.traffic?"Traffic · applied at launch":"Traffic & AI · preview"}>
        <Row label="Traffic" value={C.traffic?lbl(FD.traffic,cfg.traffic):'Saved configuration'} title={C.traffic?undefined:'Traffic is controlled by the saved map configuration'}>
          <IconSeg ariaLabel="Traffic" disabled={!C.traffic} value={C.traffic?cfg.traffic:'off'} onChange={v=>set('traffic',v)} options={FD.traffic.map(([v,label])=>({v,label,title:label,icon:TR_IC[v]}))}/>
        </Row>
        <Row label="Drivatars" value={lbl(FD.drivatars,cfg.drivatars)}>
          <IconSeg ariaLabel="Drivatars" value={cfg.drivatars} onChange={v=>set('drivatars',v)} options={FD.drivatars.filter(([v])=>!C.pchRacers||v!=='off').map(([v,label])=>({v,label,icon:DV_IC[v],disabled:v==='experimental'&&!C.exp,title:v==='experimental'&&!C.exp?'Experimental — needs an FHCM map':label}))}/>
        </Row>
        {C.pchRacers?<Row indent label="Profile" value={exp?'Tour · 10 racers · lower lot':'Standard · estate parking'}/>:<>
          {cfg.drivatars!=='off'&&<Row indent label="Count"><Stepper ariaLabel="Drivatar count" width={96} value={Math.min(cfg.count,max)} onChange={v=>set('count',v)} options={Array.from({length:max},(_,i)=>({v:i+1,label:String(i+1)}))}/></Row>}
          {exp&&<Row indent label="Profile" value={<Tag tone="exp">Exp</Tag>}><Stepper ariaLabel="AI profile" value={cfg.profile} onChange={v=>set('profile',v)} options={opt(FD.profiles)}/></Row>}
        </>}
      </Group>
      <Group label={C.conditions?'Display · applied at launch':'Display · saved configuration'}>
        <Row label="HUD preset" value={lbl(FD.hud,cfg.hud)}><IconSeg ariaLabel="HUD preset" disabled={!C.conditions} value={cfg.hud} onChange={v=>set('hud',v)} options={FD.hud.filter(([v])=>v!=='custom').map(([v,label])=>({v,label,title:label,icon:HUD_IC[v]}))}/></Row>
        <DSwitch label="Disable HUD" checked={!displayFor(cfg).enabled} disabled={!C.conditions} onChange={v=>set('display',{...displayFor(cfg),enabled:!v})}/>
        <DSwitch label="Speedometer" checked={displayFor(cfg).speedometer} disabled={!C.conditions||!displayFor(cfg).enabled} onChange={v=>set('display',{...displayFor(cfg),speedometer:v})}/>
        <DSwitch label="Minimap" checked={displayFor(cfg).minimap} disabled={!C.conditions||!displayFor(cfg).enabled} onChange={v=>set('display',{...displayFor(cfg),minimap:v})}/>
        <Note>Save changes or save a new session preset to keep these HUD choices. Presets and your last selection survive launcher restarts.</Note>
      </Group>
    </div>
    <div style={{borderTop:'1px solid var(--border-subtle)',padding:'12px 0 8px'}}>
      <div style={{...LB,padding:'0 16px 4px'}}>Recent</div>
      {recent.slice(0,3).map((r,i)=><RecentRow key={i+r.when} r={r} onLoad={onLoadRecent}/>)}
    </div>
  </aside>;
}
function MapControls({original,children}){return <fieldset disabled={original} inert={original?'':undefined} style={{border:0,padding:0,margin:0,minWidth:0,minHeight:0,display:'flex',flexDirection:'column',opacity:original?0.38:1,pointerEvents:original?'none':undefined}}>{children}</fieldset>;}
function VehiclePanel(props){return <MapControls original={props.cfg.map==='fh2'}><VehiclePanelContent {...props}/></MapControls>;}
function SessionPanel(props){return <MapControls original={props.cfg.map==='fh2'}><SessionPanelContent {...props}/></MapControls>;}
Object.assign(window,{carById,carName,MapPanel,MapRail,MapHero,VehiclePanel,SessionPanel});



function LibraryManager({error,maps,removed,busy,onAction,onExport,onImport,onClose}){
 return <div role="dialog" aria-label="Manage map library" style={{position:'fixed',inset:0,zIndex:90,background:'rgba(0,0,0,.45)',display:'flex',alignItems:'center',justifyContent:'center'}}><section style={{width:580,maxWidth:'calc(100vw - 48px)',maxHeight:'calc(100vh - 80px)',overflow:'auto',padding:24,background:'var(--l-bg0)',color:'var(--l-ink)',border:'1px solid var(--border-default)'}}><div style={{display:'flex',justifyContent:'space-between',alignItems:'center'}}><h2 style={{margin:0}}>Manage maps</h2><IconBtn icon="x" label="Close map library" onClick={onClose}/></div><p style={{fontSize:13,lineHeight:1.6,color:'var(--text-muted)'}}>Import an FHCM map ZIP, then select it and Drive. Removing a map hides it from the installed list; source ZIPs, retained packages and saves stay on disk.</p><SmallBtn icon="package-plus" disabled={busy} onClick={onImport}>Import ZIP</SmallBtn><h3>Installed</h3>{error&&<p role="alert" style={{color:'var(--red-500)',fontSize:13,lineHeight:1.6,overflowWrap:'anywhere'}}>{error}</p>}{maps.filter(m=>!m.original).map(m=><div key={m.id} style={{display:'flex',alignItems:'center',gap:10,padding:'12px 0',borderBottom:'1px solid var(--border-subtle)'}}><div style={{flex:1,minWidth:0}}><strong>{m.name}</strong><div style={{fontSize:12,color:'var(--text-muted)'}}>v{m.version}{m.favorite?' · Favorite':''}</div></div><SmallBtn disabled={busy} onClick={()=>onExport(m.id)}>Export ZIP</SmallBtn><SmallBtn disabled={busy} onClick={()=>onAction(m.id,'remove')}>Remove</SmallBtn></div>)}<h3>Removed · files retained</h3>{removed.length?removed.map(m=><div key={m.id} style={{display:'flex',alignItems:'center',gap:10,padding:'12px 0',borderBottom:'1px solid var(--border-subtle)'}}><strong style={{flex:1}}>{m.name}</strong><SmallBtn disabled={busy} onClick={()=>onAction(m.id,'restore')}>Restore</SmallBtn></div>):<p style={{fontSize:13,color:'var(--text-muted)'}}>No removed maps.</p>}<p style={{fontSize:12,lineHeight:1.6,color:'var(--text-muted)'}}>FHCM Git downloads can use this same versioned package format. Online acquisition will be added later.</p></section></div>;
}

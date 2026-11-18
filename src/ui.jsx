/* FHCM Launcher — compact desktop controls built on FHCM tokens */
const {Icon:LIcon}=window.FHCMDesignSystem_b88bf6;
const {useState,useRef,useEffect}=React;
const LB={font:'600 11px/1.2 var(--font-display)',letterSpacing:'0.1em',textTransform:'uppercase',color:'var(--text-muted)'};
const MONO={font:'500 12px/1.3 var(--font-mono)',color:'var(--text-muted)',whiteSpace:'nowrap'};
const ELL={whiteSpace:'nowrap',overflow:'hidden',textOverflow:'ellipsis'};

function useOutside(ref,open,close){
  useEffect(()=>{if(!open)return;
    const h=e=>{if(ref.current&&!ref.current.contains(e.target))close()};
    const k=e=>{if(e.key==='Escape')close()};
    document.addEventListener('mousedown',h);document.addEventListener('keydown',k);
    return()=>{document.removeEventListener('mousedown',h);document.removeEventListener('keydown',k)}},[open]);
}
function useHover(){const [h,setH]=useState(false);return [h,{onMouseEnter:()=>setH(true),onMouseLeave:()=>setH(false)}]}

function Seg({options,value,onChange,disabled,ariaLabel,height=32}){
  return <div role="radiogroup" aria-label={ariaLabel} style={{display:'flex',border:'1px solid var(--border-default)',height,opacity:disabled?0.4:1,boxSizing:'border-box'}}>
    {options.map((o,i)=><SegBtn key={o.v} o={o} sel={o.v===value} dis={disabled||o.disabled} first={i===0} onClick={()=>onChange(o.v)}/>)}
  </div>;
}
function SegBtn({o,sel,dis,first,onClick}){
  const [h,hp]=useHover();
  return <button role="radio" aria-checked={sel} disabled={dis} title={o.title} onClick={onClick} {...hp}
    style={{flex:o.w?`0 0 ${o.w}px`:'1 1 0',minWidth:0,display:'flex',alignItems:'center',justifyContent:'center',position:'relative',border:'none',borderLeft:first?'none':'1px solid var(--border-subtle)',padding:'0 2px',
      background:sel?'var(--l-sel)':h&&!dis?'var(--l-hov)':'transparent',
      color:sel?'var(--l-ink)':dis?'var(--l-ink3)':'var(--l-ink2)',font:'600 11px/1 var(--font-display)',letterSpacing:'0.03em',textTransform:'uppercase',
      cursor:dis?'not-allowed':'pointer',transition:'background var(--dur-fast),color var(--dur-fast)',...ELL}}>
    {o.icon?<LIcon name={o.icon} size={14}/>:o.label}{sel&&<span style={{position:'absolute',left:0,right:0,bottom:-1,height:2,background:'var(--gradient-horizon-h)'}}></span>}
  </button>;
}

function Menu({children,style,role='listbox'}){
  return <div role={role} style={{position:'absolute',top:'calc(100% + 4px)',left:0,right:0,zIndex:40,background:'var(--l-bg2)',border:'1px solid var(--border-default)',boxShadow:'var(--shadow-lg)',padding:4,...style}}>{children}</div>;
}
function MenuItem({children,sel,disabled,hint,onClick,icon,active}){
  const [h,hp]=useHover();
  return <button role="option" aria-selected={!!sel} disabled={disabled} onClick={onClick} {...hp}
    style={{display:'flex',width:'100%',alignItems:'center',gap:8,minHeight:32,padding:'6px 8px',border:'none',background:(h||active)&&!disabled?'var(--l-sel)':'transparent',color:disabled?'var(--l-ink3)':'var(--text-primary)',font:'500 13px/1.25 var(--font-body)',textAlign:'left',cursor:disabled?'not-allowed':'pointer'}}>
    <span style={{width:14,display:'inline-flex',flexShrink:0}}>{icon?<LIcon name={icon} size={14}/>:sel&&<LIcon name="check" size={14} color="var(--fh-coral)"/>}</span>
    <span style={{flex:1,minWidth:0}}>{children}</span>
    {hint&&<span style={{...MONO,fontSize:11,flexShrink:0}}>{hint}</span>}
  </button>;
}

function Dropdown({value,options,onChange,disabled,ariaLabel}){
  const [open,setOpen]=useState(false);const ref=useRef();useOutside(ref,open,()=>setOpen(false));
  const cur=options.find(o=>o.v===value);
  return <div ref={ref} style={{position:'relative'}}>
    <button disabled={disabled} aria-label={ariaLabel} onClick={()=>setOpen(o=>!o)} aria-haspopup="listbox" aria-expanded={open}
      style={{width:'100%',height:32,display:'flex',alignItems:'center',justifyContent:'space-between',gap:8,padding:'0 10px',background:'var(--surface-sunken)',border:'1px solid '+(open?'var(--border-strong)':'var(--border-default)'),color:'var(--text-primary)',font:'500 13px/1 var(--font-body)',cursor:disabled?'not-allowed':'pointer',opacity:disabled?0.4:1}}>
      <span style={ELL}>{cur?cur.label:'—'}</span><LIcon name="chevron-down" size={15} style={{transform:open?'rotate(180deg)':'none',transition:'transform var(--dur-fast) var(--ease-out)'}}/>
    </button>
    {open&&<Menu>{options.map(o=><MenuItem key={o.v} sel={o.v===value} disabled={o.disabled} hint={o.hint} onClick={()=>{onChange(o.v);setOpen(false)}}>{o.label}</MenuItem>)}</Menu>}
  </div>;
}

function Field({label,right,note,children}){
  return <div style={{display:'flex',flexDirection:'column',gap:8}}>
    <div style={{display:'flex',alignItems:'center',justifyContent:'space-between',gap:8,minHeight:16}}><span style={LB}>{label}</span>{right}</div>
    {children}{note}
  </div>;
}
const TONES={muted:['var(--text-muted)','info'],warn:['var(--status-warning)','triangle-alert'],err:['var(--status-danger)','circle-alert'],ok:['var(--status-success)','circle-check'],info:['var(--status-info)','info']};
function Note({tone='muted',children,style}){
  const [c,ic]=TONES[tone];
  return <div style={{display:'flex',gap:6,alignItems:'flex-start',font:'400 12px/1.4 var(--font-body)',color:c,...style}}><LIcon name={ic} size={13} style={{marginTop:2}}/><span style={{textWrap:'pretty'}}>{children}</span></div>;
}
function Tag({children,tone}){
  const c=tone==='exp'?'var(--fh-gold)':tone==='ok'?'var(--status-success)':'var(--l-ink2)';
  return <span style={{display:'inline-flex',alignItems:'center',height:18,padding:'0 6px',border:'1px solid currentColor',color:c,font:'600 10px/1 var(--font-display)',letterSpacing:'0.1em',textTransform:'uppercase',whiteSpace:'nowrap'}}>{children}</span>;
}
function IconBtn({icon,label,onClick,active,size=32,iconSize=16,color,disabled}){
  const [h,hp]=useHover();
  return <button aria-label={label} title={label} onClick={onClick} disabled={disabled} {...hp}
    style={{width:size,height:size,display:'inline-flex',alignItems:'center',justifyContent:'center',border:'none',background:active?'var(--l-sel)':h&&!disabled?'var(--l-hov)':'transparent',color:color||(active||h?'var(--l-ink)':'var(--l-ink2)'),cursor:disabled?'not-allowed':'pointer',opacity:disabled?0.4:1,flexShrink:0,transition:'background var(--dur-fast)'}}>
    <LIcon name={icon} size={iconSize}/></button>;
}
function SmallBtn({children,icon,onClick,variant='outline',disabled,tone}){
  const [h,hp]=useHover();
  const c=tone==='gold'?'var(--fh-gold)':'var(--text-primary)';
  return <button onClick={onClick} disabled={disabled} {...hp}
    style={{height:28,display:'inline-flex',alignItems:'center',gap:6,padding:'0 10px',flexShrink:0,
      border:variant==='outline'?'1px solid '+(tone==='gold'?'rgba(250,178,22,0.5)':h?'var(--border-strong)':'var(--border-default)'):'none',
      background:h&&!disabled?'var(--l-hov)':'transparent',color:c,font:'600 11px/1 var(--font-display)',letterSpacing:'0.08em',textTransform:'uppercase',cursor:disabled?'not-allowed':'pointer',opacity:disabled?0.4:1,whiteSpace:'nowrap'}}>
    {icon&&<LIcon name={icon} size={13}/>}{children}</button>;
}
function SearchInput({value,onChange,placeholder,autoFocus,onKeyDown}){
  return <label style={{display:'flex',alignItems:'center',gap:8,height:32,padding:'0 10px',background:'var(--surface-sunken)',border:'1px solid var(--border-default)',color:'var(--text-muted)'}}>
    <LIcon name="search" size={14}/>
    <input value={value} autoFocus={autoFocus} onKeyDown={onKeyDown} onChange={e=>onChange(e.target.value)} placeholder={placeholder} aria-label={placeholder}
      style={{flex:1,minWidth:0,background:'none',border:'none',outline:'none',color:'var(--text-primary)',font:'400 13px/1 var(--font-body)'}}/>
    {value&&<button aria-label="Clear" onClick={()=>onChange('')} style={{border:'none',background:'none',color:'var(--text-muted)',cursor:'pointer',display:'inline-flex',padding:0}}><LIcon name="x" size={14}/></button>}
  </label>;
}
function Thumb({tone,style,children,label,img}){
  if(img)return <div style={{position:'relative',overflow:'hidden',flexShrink:0,background:`#0E0F11 url("${img}") center/cover no-repeat`,...style}}>{children}</div>;
  return <div style={{position:'relative',overflow:'hidden',flexShrink:0,background:`linear-gradient(160deg,color-mix(in oklch,${tone} 30%,#15161A) 0%,color-mix(in oklch,${tone} 10%,#0E0F11) 100%)`,...style}}>
    {label&&<span style={{position:'absolute',top:12,right:14,...MONO,fontSize:10,letterSpacing:'0.08em',color:'rgba(255,255,255,0.45)'}}>{label}</span>}
    {children}</div>;
}
function Bar({v,h=2,paused}){
  return <div style={{height:h,background:'var(--l-track)',overflow:'hidden'}}><div style={{height:'100%',width:v+'%',background:paused?'var(--neutral-500)':'var(--gradient-horizon-h)',transition:'width 500ms linear'}}></div></div>;
}
function Popover({anchorRef,open,onClose,children,width=340,align='right'}){
  const ref=useRef();
  useEffect(()=>{if(!open)return;
    const h=e=>{if(ref.current&&!ref.current.contains(e.target)&&!(anchorRef.current&&anchorRef.current.contains(e.target)))onClose()};
    const k=e=>{if(e.key==='Escape')onClose()};
    document.addEventListener('mousedown',h);document.addEventListener('keydown',k);
    return()=>{document.removeEventListener('mousedown',h);document.removeEventListener('keydown',k)}},[open]);
  if(!open)return null;
  return <div ref={ref} style={{position:'absolute',top:'calc(100% + 6px)',[align]:0,width,zIndex:50,background:'var(--l-bg2)',border:'1px solid var(--border-default)',boxShadow:'var(--shadow-lg)'}}>{children}</div>;
}
const fmtTime=m=>String(Math.floor(m/60)).padStart(2,'0')+':'+String(m%60).padStart(2,'0');

function Row({label,value,children,indent,title}){
  return <div title={title} style={{display:'flex',alignItems:'center',gap:12,minHeight:42,paddingLeft:indent?14:0,borderBottom:'1px solid var(--border-subtle)'}}>
    <span style={{flex:1,minWidth:0,display:'flex',alignItems:'baseline',gap:8,...ELL}}>
      <span style={{font:'400 13px/1.2 var(--font-body)',color:indent?'var(--text-muted)':'var(--l-ink2)',flexShrink:0}}>{label}</span>
      {value!=null&&<span style={{font:'500 12px/1.2 var(--font-body)',color:'var(--l-ink)',...ELL}}>{value}</span>}
    </span>{children}
  </div>;
}
function Group({label,children}){
  return <div style={{display:'flex',flexDirection:'column'}}><div style={{...LB,fontSize:10,padding:'18px 0 4px'}}>{label}</div>{children}</div>;
}
function IconSeg({options,value,onChange,disabled,ariaLabel}){
  return <Seg height={28} ariaLabel={ariaLabel} value={value} onChange={onChange} disabled={disabled} options={options.map(o=>({...o,w:30}))}/>;
}
function Stepper({options,value,onChange,disabled,width=128,ariaLabel}){
  const en=options.filter(o=>!o.disabled);const i=en.findIndex(o=>o.v===value);const cur=options.find(o=>o.v===value);
  const go=d=>{const n=en[i+d];if(n)onChange(n.v)};
  return <div role="group" aria-label={ariaLabel} style={{display:'flex',alignItems:'center',width,height:28,flexShrink:0,border:'1px solid var(--border-default)',boxSizing:'border-box',opacity:disabled?0.4:1}}>
    <IconBtn size={26} iconSize={14} icon="chevron-left" label="Previous" disabled={disabled||i<=0} onClick={()=>go(-1)}/>
    <span style={{flex:1,minWidth:0,textAlign:'center',font:'500 12.5px/1 var(--font-body)',color:'var(--l-ink)',...ELL}}>{cur?cur.label:'—'}</span>
    <IconBtn size={26} iconSize={14} icon="chevron-right" label="Next" disabled={disabled||i<0||i>=en.length-1} onClick={()=>go(1)}/>
  </div>;
}

Object.assign(window,{LIcon,LB,MONO,ELL,useOutside,useHover,Seg,Menu,MenuItem,Dropdown,Field,Note,Tag,IconBtn,SmallBtn,SearchInput,Thumb,Bar,Popover,fmtTime,Row,Group,IconSeg,Stepper});

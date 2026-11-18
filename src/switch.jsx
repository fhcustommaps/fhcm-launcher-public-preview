import React from 'react';
export function Switch({label,checked=false,onChange,disabled=false,description}){
  return <label style={{display:'inline-flex',alignItems:'center',gap:12,minHeight:44,cursor:disabled?'not-allowed':'pointer',opacity:disabled?0.5:1,color:'var(--text-primary)',position:'relative'}}>
    <input type="checkbox" role="switch" aria-checked={checked} checked={checked} onChange={e=>onChange&&onChange(e.target.checked)} disabled={disabled} style={{position:'absolute',opacity:0,width:44,height:24,margin:0}}/>
    <span style={{width:44,height:24,flexShrink:0,borderRadius:'var(--radius-pill)',background:checked?'var(--gradient-horizon-h)':'var(--border-strong)',position:'relative',transition:'background var(--dur-base) var(--ease-out)'}}>
      <span style={{position:'absolute',top:3,left:checked?23:3,width:18,height:18,borderRadius:'50%',background:'#fff',boxShadow:'var(--shadow-sm)',transition:'left var(--dur-base) var(--ease-out)'}}/>
    </span><span style={{display:'flex',flexDirection:'column',gap:2}}><span style={{fontSize:15,fontWeight:500}}>{label}</span>{description&&<span style={{fontSize:13,color:'var(--text-muted)'}}>{description}</span>}</span>
  </label>;
}

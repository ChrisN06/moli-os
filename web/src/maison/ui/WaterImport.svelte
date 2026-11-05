<script>
 import {login} from '../../lib/hub.svelte.js';
 import {t} from '../../lib/i18n.svelte.js';
 let file=$state(null),unit=$state('m3'),busy=$state(false),needPin=$state(false),pin=$state(''),message=$state('');
 async function send(){if(busy||!file)return;busy=true;message='';try{
  if(file.size>250000){message=t('water.invalid');return;}
  if(needPin){const error=await login(pin);pin='';if(error){message=error;return;}}
  const r=await fetch('/api/water',{method:'PUT',headers:{'Content-Type':'application/json'},body:JSON.stringify({csv:await file.text(),unit})});
  if(r.status===403)needPin=true;
  if(!r.ok){message=(await r.json()).error??t('water.failed');return;}
  message=t('water.saved');needPin=false;
 }catch{message=t('water.failed');}finally{busy=false;}}
</script>
<section class="card"><h2>{t('water.setup')}</h2><p>{t('water.setup_note')}</p>
 <form onsubmit={e=>{e.preventDefault();send();}}>
 <label>{t('water.file')}<input type="file" accept=".csv,text/csv" onchange={e=>file=e.currentTarget.files?.[0]??null} required/></label>
 <label>{t('water.unit')}<select bind:value={unit}><option value="m3">m³</option><option value="L">L</option></select></label>
 {#if needPin}<label>{t('water.pin')}<input type="password" autocomplete="off" bind:value={pin} required/></label>{/if}
 <button type="submit" disabled={busy||!file}>{t('water.import')}</button>
 </form>{#if message}<p role="status">{message}</p>{/if}
</section>
<style>h2{margin:0}form{display:flex;flex-wrap:wrap;gap:16px;align-items:end}label{display:grid;gap:8px;max-width:100%}input,select,button{min-height:44px;background:var(--surface);color:var(--ink);max-width:100%}p{line-height:1.5}</style>

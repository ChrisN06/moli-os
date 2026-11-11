<script>
 import {onMount} from 'svelte';
 import {home,num} from '../lib/home.svelte.js';
 import {barIndex} from '../lib/readings.js';
 import {waterMonth,waterTotal,localMonth} from '../lib/water.js';
 import {t} from '../../lib/i18n.svelte.js';
 let {chart=false}=$props();
 let history=$state(null),failed=$state(false),month=$state(localMonth()),selected=$state(null),monthly=$state(false);
 const current=$derived(localMonth(new Date(home.now)));
 const months=$derived.by(()=>{
  const options=[];
  const cursor=new Date(Number(current.slice(0,4)),Number(current.slice(5))-1,1);
  const first=Object.keys(history?.days??{}).sort()[0]?.slice(0,7)??current;
  while(localMonth(cursor)>=first){options.push(localMonth(cursor));cursor.setMonth(cursor.getMonth()-1);}
  return options;
 });
 const monthName=(key)=>new Date(key+'-01T12:00:00').toLocaleDateString(undefined,{month:'long',year:'numeric'});
 const rows=$derived(monthly&&chart ? months.filter(key=>key.slice(0,4)===month.slice(0,4)).reverse().map(date=>{
  const daily=waterMonth(history?.days,date);
  return {date,value:waterTotal(daily),partial:daily.some(row=>row.value==null)};
 }) : waterMonth(history?.days,chart?month:current));
 const total=$derived(waterTotal(rows));
 const last=$derived(rows.filter(r=>r.value!=null).at(-1)?.date);
 const max=$derived(Math.max(.001,...rows.map(r=>r.value??0)));
 const chosen=$derived(rows.find(r=>r.date===selected));
 function select(event) {
  if(event.type==='pointerdown'){if(event.pointerType!=='mouse')event.preventDefault();event.currentTarget.setPointerCapture(event.pointerId);}
  if(event.type==='pointermove'&&event.pointerType!=='mouse'&&!event.buttons)return;
  const box=event.currentTarget.getBoundingClientRect();
  const i=barIndex(event.clientX,box.left,box.width,rows.length);
  if(i!=null)selected=rows[i].date;
 }
 onMount(()=>{
  const controller=new AbortController();
  const refresh=async()=>{if(document.visibilityState!=='visible')return;try{const r=await fetch('/api/water',{signal:controller.signal});if(!r.ok)throw Error();history=await r.json();failed=false;}catch{if(!controller.signal.aborted)failed=true;}};
  refresh();const timer=setInterval(refresh,60000);return()=>{controller.abort();clearInterval(timer);};
 });
</script>
{#if chart || Object.keys(history?.days??{}).length}
<section class="card water">
 <header>
  <h2>{t(chart?(monthly?'orbite.monthly':'water.daily'):'water.month')}</h2>
  {#if chart}
   <select aria-label={t('water.period')} value={monthly ? months.find(key=>key.startsWith(month.slice(0,4))) : month} onchange={event=>{month=event.currentTarget.value;selected=null;}}>
    {#each monthly ? months.filter((key,i)=>i===0||key.slice(0,4)!==months[i-1].slice(0,4)) : months as key}
     <option value={key}>{monthly?key.slice(0,4):monthName(key)}</option>
    {/each}
   </select>
   <button onclick={()=>{monthly=!monthly;selected=null;}}>{t(monthly?'orbite.daily':'orbite.monthly')}</button>
  {/if}
 </header>
 {#if failed}<p role="status">{t('water.failed')}</p>{/if}
 <strong class="total">{num(total,3)} m³</strong>
 {#if last}<p class="muted">{t('water.through',{date:last})}</p>{:else}<p class="muted">{t('water.empty')}</p>{/if}
 {#if chart && total!=null}
 <div class="bars" onpointerdown={select} onpointermove={select} role="group" aria-label={t('water.daily')}>
 {#each rows as row,i}<button class:missing={row.value==null} class:partial={row.partial&&row.value!=null} class:selected={selected===row.date} onfocus={()=>selected=row.date} onclick={()=>selected=row.date} aria-label={`${row.date} : ${row.value==null?t('water.missing'):`${num(row.value*1000,0)} L`}`} title={`${row.date} : ${row.value==null?t('water.missing'):`${num(row.value*1000,0)} L`}`}><i style:height={`${row.value==null?0:Math.max(1,row.value/max*100)}%`}></i><small>{monthly ? row.date.slice(5) : (i===0||(i+1)%5===0)?i+1:''}</small></button>{/each}
 </div>
 <p class="readout" aria-live="polite">{chosen?`${chosen.date} : ${chosen.value==null?t('water.missing'):`${num(chosen.value*1000,0)} L`}`:t('water.inspect')}{#if chosen?.partial} · {t('orbite.partial')}{/if}</p>
 {/if}
 <p class="muted">{t(history?.sync?.configured?'water.automatic_note':'water.import_note')}</p>
 {#if history?.sync?.error}<p role="status">{history.sync.error}</p>{/if}
 <a href={chart?'#/parametres':'#/energie'}>{t(chart?'water.setup':'water.view')}</a>
</section>
{/if}
<style>
 header{display:flex;justify-content:space-between;gap:12px;flex-wrap:wrap}h2{margin:0}.total{display:block;font-size:28px;margin-top:16px}select,header button{min-height:44px;background:var(--surface);color:var(--ink)}.water{min-width:0}.bars{touch-action:pan-y;display:flex;height:180px;gap:2px;margin-top:24px;padding-bottom:24px}.bars button{position:relative;display:flex;align-items:end;flex:1;min-width:0;padding:0;border:0;border-bottom:1px solid var(--line);background:transparent;cursor:pointer}.bars i{display:block;width:100%;background:var(--cool);border-radius:3px 3px 0 0}.bars small{position:absolute;top:100%;width:100%;font-size:10px;padding-top:4px}.bars .missing{background:repeating-linear-gradient(135deg,transparent,transparent 5px,var(--line) 5px,var(--line) 6px)}.bars .partial i{opacity:.55}.bars .selected i{background:var(--ink)}.readout{min-height:24px}
</style>

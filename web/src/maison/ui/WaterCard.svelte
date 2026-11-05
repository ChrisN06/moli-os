<script>
 import {onMount} from 'svelte';
 import {home,num} from '../lib/home.svelte.js';
 import {barIndex} from '../lib/readings.js';
 import {waterMonth,waterTotal,localMonth} from '../lib/water.js';
 import {t} from '../../lib/i18n.svelte.js';
 let {chart=false}=$props();
 let history=$state(null),failed=$state(false),month=$state(localMonth()),selected=$state(null);
 const current=$derived(localMonth(new Date(home.now)));
 const rows=$derived(waterMonth(history?.days,chart?month:current));
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
 <header><h2>{t(chart?'water.daily':'water.month')}</h2>{#if chart}<input aria-label={t('water.period')} type="month" max={current} bind:value={month}/>{/if}</header>
 {#if failed}<p role="status">{t('water.failed')}</p>{/if}
 <strong class="total">{num(total,3)} m³</strong>
 {#if last}<p class="muted">{t('water.through',{date:last})}</p>{:else}<p class="muted">{t('water.empty')}</p>{/if}
 {#if chart && total!=null}
 <div class="bars" onpointerdown={select} onpointermove={select} role="group" aria-label={t('water.daily')}>
 {#each rows as row,i}<button class:missing={row.value==null} class:selected={selected===row.date} onfocus={()=>selected=row.date} onclick={()=>selected=row.date} aria-label={`${row.date} : ${row.value==null?t('water.missing'):`${num(row.value*1000,0)} L`}`} title={`${row.date} : ${row.value==null?t('water.missing'):`${num(row.value*1000,0)} L`}`}><i style:height={`${row.value==null?0:Math.max(1,row.value/max*100)}%`}></i><small>{(i===0||(i+1)%5===0)?i+1:''}</small></button>{/each}
 </div>
 <p class="readout" aria-live="polite">{chosen?`${chosen.date} : ${chosen.value==null?t('water.missing'):`${num(chosen.value*1000,0)} L`}`:t('water.inspect')}</p>
 {/if}
 <p class="muted">{t('water.import_note')}</p>
 <a href={chart?'#/parametres':'#/energie'}>{t(chart?'water.import':'water.view')}</a>
</section>
{/if}
<style>
 header{display:flex;justify-content:space-between;gap:12px;flex-wrap:wrap}h2{margin:0}.total{display:block;font-size:28px;margin-top:16px}input{min-height:44px;background:var(--surface);color:var(--ink)}.water{min-width:0}.bars{touch-action:pan-y;display:flex;height:180px;gap:2px;margin-top:24px;padding-bottom:24px}.bars button{position:relative;display:flex;align-items:end;flex:1;min-width:0;padding:0;border:0;border-bottom:1px solid var(--line);background:transparent;cursor:pointer}.bars i{display:block;width:100%;background:var(--cool);border-radius:3px 3px 0 0}.bars small{position:absolute;top:100%;width:100%;font-size:10px;padding-top:4px}.bars .missing{background:repeating-linear-gradient(135deg,transparent,transparent 5px,var(--line) 5px,var(--line) 6px)}.bars .selected i{background:var(--ink)}.readout{min-height:24px}
</style>

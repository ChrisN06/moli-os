<script>
  import { hub,home,value,act } from '../lib/home.svelte.js';
  import {t} from '../../lib/i18n.svelte.js';
  import {at,rectangle,overlay,picture} from '../lib/vacuum-map.js';
  let {id}=$props();
  const device=$derived(hub.devices[id]), instance=$derived(device?.instance), writable=$derived(device?.points?.some(p=>p.key==='action' && p.access?.write));
  let map=$state(null), image=$state(''), bounds=$state(null), saved=$state(false), busy=$state(false), message=$state(''), start=$state(null);
  const box=$derived(map && bounds ? overlay(map,bounds) : null);
  async function request(path,method='GET',body=null) {
    const r=await fetch(`/api/xiaomi/${encodeURIComponent(instance)}/${path}`,{method,headers:{'Content-Type':'application/json'},...(body?{body:JSON.stringify(body)}:{})});
    const result=await r.json();
    if(r.status===403) home.held={label:t('xiaomi.zone.title'),reason:result.error,custom:method==='PUT'?save:load};
    if(!r.ok) throw new Error(result.error??t('xiaomi.zone.map_failed'));
    return result;
  }
  async function load() {
    if(busy)return;busy=true;message='';
    try {
      const result=await request('map'); image=picture(result);map=result;
      const stored=(await request('zone')).zone;
      saved=!!stored && stored.map_index===map.index && stored.frame===map.frame && stored.dock.every((p,i)=>Math.abs(p-map.dock[i])<=100);
      bounds=saved?stored.bounds:rectangle(at(map,Math.max(0,(map.width-94)/2),Math.max(0,(map.height-66)/2)),at(map,Math.min(map.width,(map.width+94)/2),Math.min(map.height,(map.height+66)/2)));
    }catch(e){message=e.message;}finally{busy=false;}
  }
  function point(e) {const svg=e.currentTarget,p=svg.createSVGPoint();p.x=e.clientX;p.y=e.clientY;const xy=p.matrixTransform(svg.getScreenCTM().inverse());return at(map,xy.x,xy.y);}
  function down(e) {if(busy)return;e.preventDefault();e.currentTarget.setPointerCapture(e.pointerId);start=point(e);bounds=rectangle(start,start);saved=false;}
  function move(e) {if(start){e.preventDefault();bounds=rectangle(start,point(e));}}
  function up(e) {if(start){move(e);start=null;}}
  async function save() {
    if(!map || !bounds || busy)return;busy=true;message='';
    try {await request('zone','PUT',{bounds,dock:map.dock,map_index:map.index,frame:map.frame,confirmed_at:0});saved=true;message=t('xiaomi.zone.saved');}
    catch(e){message=e.message;}finally{busy=false;}
  }
  async function order(action) {message='';const result=await act(`${id}/action`,action,t(`xiaomi.vacuum.${action}`));if(result?.error)message=result.error;}
</script>
<section class="card">
  <h2>{t('xiaomi.vacuum.title')}</h2>
  <p>{device?.online ? ((value(id,'state')??0)<=18 ? t(`xiaomi.state.${value(id,'state')??0}`) : `${t('xiaomi.vacuum.state')} ${value(id,'state')}`) : t('xiaomi.vacuum.offline')}</p>
  <dl>
    <div><dt>{t('xiaomi.vacuum.battery')}</dt><dd>{value(id,'battery')??'—'} %</dd></div>
    <div><dt>{t('xiaomi.vacuum.area')}</dt><dd>{value(id,'clean_area')?.toFixed(1)??'—'} m²</dd></div>
    <div><dt>{t('xiaomi.vacuum.time')}</dt><dd>{value(id,'clean_time')!=null?Math.round(value(id,'clean_time')/60):'—'} min</dd></div>
  </dl>
  {#if value(id,'error')}<p role="alert">{t('xiaomi.vacuum.error')} : {value(id,'error')}</p>{/if}
  <div class="actions">
    <button disabled={!writable || !device?.online} onclick={()=>order('pause')}>{t('xiaomi.vacuum.pause')}</button>
    <button disabled={!writable || !device?.online} onclick={()=>order('dock')}>{t('xiaomi.vacuum.dock')}</button>
    <button disabled={!writable || !device?.online || (!value(id,'kitchen_ready') || (map && !saved))} onclick={()=>order('kitchen')}>{t('xiaomi.vacuum.kitchen')}</button>
  </div>
  {#if !writable}<p class="muted">{t('xiaomi.vacuum.read_only')}</p>{/if}
  <h3>{t('xiaomi.zone.title')}</h3>
  <p class="muted">{t('xiaomi.zone.note')}</p>
  <button disabled={busy || !device?.online} onclick={load}>{t(busy?'xiaomi.zone.loading':'xiaomi.zone.load')}</button>
  {#if map}
    <p>{t('xiaomi.zone.draw')}</p>
    <svg viewBox={`0 0 ${map.width} ${map.height}`} role="img" aria-label={t('xiaomi.zone.title')} onpointerdown={down} onpointermove={move} onpointerup={up} onpointercancel={()=>{start=null;}}>
      <image href={image} width={map.width} height={map.height}/>
      <circle cx={(map.dock[0]-map.left)/50} cy={map.height-((map.dock[1]-map.bottom)/50)} r="3" fill="#f7b731"/>
      {#if map.robot}<circle cx={(map.robot[0]-map.left)/50} cy={map.height-((map.robot[1]-map.bottom)/50)} r="3" fill="#2638b8"/>{/if}
      {#if box}<rect x={box.x} y={box.y} width={box.width} height={box.height} fill="#f7b73144" stroke="#bd6c00" stroke-width="1"/>{/if}
    </svg>
    {#if bounds}<p>{((bounds[2]-bounds[0])/1000).toFixed(2)} × {((bounds[3]-bounds[1])/1000).toFixed(2)} m · {t(saved?'xiaomi.zone.confirmed':'xiaomi.zone.unconfirmed')}</p>{/if}
    {#if bounds}<details><summary>{t('xiaomi.zone.coordinates')}</summary><div class="coordinates">{#each ['X min','Y min','X max','Y max'] as label,i}<label>{label}<input type="number" step="0.05" min="0" max="51.2" value={bounds[i]/1000} onchange={e=>{bounds=bounds.map((v,j)=>j===i?Math.round(Number(e.currentTarget.value)*1000):v);saved=false;}}/></label>{/each}</div></details>{/if}
    <button disabled={busy || !bounds || bounds[2]-bounds[0]<500 || bounds[3]-bounds[1]<500} onclick={save}>{t('xiaomi.zone.confirm')}</button>
  {/if}
  {#if message}<p role="status">{message}</p>{/if}
</section>
<style>
 .coordinates{display:flex;flex-wrap:wrap;gap:12px}.coordinates label{display:grid;gap:4px}.coordinates input{width:100px;min-height:44px}h2{margin-top:0}h3{margin-top:24px}dl,.actions{display:flex;gap:16px;flex-wrap:wrap}dt{font-size:13px;color:var(--muted)}dd{margin:4px 0;font-weight:600}button{min-height:44px;padding:8px 14px;border:1px solid var(--surface-3);border-radius:8px;background:var(--surface);color:var(--ink);cursor:pointer}button:disabled{opacity:.5;cursor:default}svg{display:block;max-width:100%;max-height:520px;width:100%;margin-top:16px;touch-action:none;user-select:none;-webkit-user-select:none}p{line-height:1.5}
</style>

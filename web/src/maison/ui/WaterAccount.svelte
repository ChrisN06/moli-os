<script>
  import {onMount} from 'svelte';
  import {login} from '../../lib/hub.svelte.js';
  import {t} from '../../lib/i18n.svelte.js';
  let email=$state(''), password=$state(''), meter=$state(''), pin=$state('');
  let status=$state(null), needPin=$state(false), busy=$state(false), message=$state('');
  async function load() {
    const response=await fetch('/api/water/account');
    if(response.status===403){needPin=true;return;}
    if(!response.ok)throw Error();
    status=await response.json();needPin=false;
  }
  onMount(()=>{load().catch(()=>message=t('water.failed'));});
  async function unlock(){
    busy=true;message='';
    try{const error=await login(pin);pin='';if(error){message=error;return;}await load();}
    catch{message=t('water.failed');}finally{busy=false;}
  }
  async function send(connect){
    if(busy)return;
    busy=true;message='';
    const body=connect?{email,password,meter}:{};
    password='';
    try{
      const response=await fetch(connect?'/api/water/account':'/api/water/sync',{
        method:'POST',headers:{'Content-Type':'application/json'},body:JSON.stringify(body)
      });
      if(response.status===403)needPin=true;
      const reply=await response.json();
      if(!response.ok){message=reply.error??t('water.failed');return;}
      await load();message=t('water.connected');
    }catch{message=t('water.failed');}finally{busy=false;}
  }
</script>
<div class="account">
  <h3>{t('water.account')}</h3>
  <p>{t('water.account_note')}</p>
  {#if needPin}
    <form onsubmit={e=>{e.preventDefault();unlock();}}>
      <label>{t('water.pin')}<input type="password" autocomplete="off" bind:value={pin} required/></label>
      <button type="submit" disabled={busy}>{t('water.unlock')}</button>
    </form>
  {:else if status}
    {#if status.configured}
      <p>{t('water.automatic')}</p>
      {#if status.synchronized_at}<p>{t('water.synced',{date:new Date(status.synchronized_at).toLocaleString()})}</p>{/if}
      {#if status.error}<p role="status">{status.error}</p>{/if}
      <button type="button" disabled={busy} onclick={()=>send(false)}>{t('water.sync_now')}</button>
    {/if}
    <details open={!status.configured}>
      <summary>{t(status.configured?'water.reconnect':'water.connect')}</summary>
      <form onsubmit={e=>{e.preventDefault();send(true);}}>
        <label>{t('water.email')}<input type="email" autocomplete="username" bind:value={email} maxlength="254" required/></label>
        <label>{t('water.password')}<input type="password" autocomplete="current-password" bind:value={password} maxlength="1024" required/></label>
        <label>{t('water.meter')}<input type="text" autocomplete="off" bind:value={meter} maxlength="64"/></label>
        <button type="submit" disabled={busy}>{t('water.connect')}</button>
      </form>
    </details>
  {/if}
  {#if busy}<p role="status">{t('water.connecting')}</p>{/if}
  {#if message}<p role="status">{message}</p>{/if}
</div>
<style>
  .account{margin:20px 0;padding-bottom:20px;border-bottom:1px solid var(--line)}
  h3{margin:0}p{line-height:1.5}form{display:flex;flex-wrap:wrap;gap:16px;align-items:end;margin:16px 0}
  label{display:grid;gap:8px;max-width:100%}input,button{min-height:44px;background:var(--surface);color:var(--ink);max-width:100%}
  summary{padding:12px 0;cursor:pointer}
</style>

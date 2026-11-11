<script>
  import { onMount } from 'svelte';
  import { login } from '../../lib/hub.svelte.js';
  import { t } from '../../lib/i18n.svelte.js';
  let devices = $state([]), username = $state(''), password = $state(''), pin = $state('');
  let challenge = $state(null), code = $state(''), busy = $state(false), needPin = $state(false), message = $state('');
  async function refresh() {
    const r = await fetch('/api/xiaomi');
    if (r.ok) devices = (await r.json()).devices;
  }
  onMount(() => { refresh().catch(() => {}); });
  async function send(path, method='POST', body=null) {
    if (busy) return;
    busy = true; message = '';
    try {
      if (needPin && !(await login(pin))) { message=t('energie.price.code_failed'); return; }
      const r = await fetch(path, {method, headers:{'Content-Type':'application/json'}, ...(body ? {body:JSON.stringify(body)} : {})});
      const result = await r.json();
      if (r.status === 403) needPin = true;
      if (!r.ok) { message=result.error ?? t('xiaomi.account.failed'); return; }
      needPin=false;
      if (result.challenge) { challenge=result.challenge; code=''; return; }
      challenge=null; username=''; code='';
      message = result.devices?.map(d => `${d.instance} : ${d.configured ? t('xiaomi.account.saved') : d.error}`).join(' · ') ?? t('xiaomi.account.saved');
      await refresh();
    } catch { message=t('xiaomi.account.failed'); }
    finally { password=''; busy=false; }
  }
  function connect() {
    const body = challenge ? (challenge.captcha ? {captcha:code} : {verify:code}) : {username,password};
    return send('/api/xiaomi/login','POST',body);
  }
</script>

{#if devices.length}
<section class="card">
  <h2>{t('xiaomi.account.title')}</h2>
  <p class="muted">{t('xiaomi.account.note')}</p>
  <ul>{#each devices as d}<li>{d.instance} · {d.host} · {t(d.configured ? 'xiaomi.account.connected' : 'xiaomi.account.wait')}
  </li>{/each}</ul>
  <form onsubmit={e=>{e.preventDefault();connect();}}>
    {#if challenge}
      {#if challenge.captcha}<img src={`data:image/jpeg;base64,${challenge.captcha}`} alt={t('xiaomi.account.captcha')} />{:else}<p>{challenge.verify_phone ?? challenge.verify_email}</p>{/if}
      <label>{t(challenge.captcha ? 'xiaomi.account.captcha' : 'xiaomi.account.verify')}<input bind:value={code} required maxlength="32" autocomplete="one-time-code" disabled={busy}/></label>
    {:else}
      <label>{t('xiaomi.account.username')}<input type="text" bind:value={username} required maxlength="256" autocomplete="username" disabled={busy}/></label>
      <label>{t('xiaomi.account.password')}<input type="password" bind:value={password} required maxlength="256" autocomplete="current-password" disabled={busy}/></label>
    {/if}
    {#if needPin}<label>{t('energie.price.code')}<input type="password" bind:value={pin} required inputmode="numeric" autocomplete="off" disabled={busy}/></label>{/if}
    <button disabled={busy} type="submit">{t(busy ? 'xiaomi.account.connecting' : 'xiaomi.account.connect')}</button>
    {#if challenge}<button type="button" disabled={busy} onclick={()=>{challenge=null;code='';}}>{t('xiaomi.account.restart')}</button>{/if}
  </form>
  {#if devices.some(d=>d.configured)}<button disabled={busy} onclick={()=>send('/api/xiaomi/discover')}>{t('xiaomi.account.discover')}</button>{/if}
  {#if message}<p role="status">{message}</p>{/if}
</section>
{/if}
<style>
  h2{margin:0 0 12px;font-size:18px} form{display:flex;flex-wrap:wrap;gap:12px;align-items:end;margin:16px 0} label{display:grid;gap:8px;font-size:13px;min-width:0;max-width:100%} input,button{box-sizing:border-box;max-width:100%;min-height:44px;padding:8px 12px;border-radius:8px;border:1px solid var(--surface-3);background:var(--surface);color:var(--ink)} button{cursor:pointer} li{margin:10px 0} p,li{font-size:13px} img{max-width:180px}
</style>

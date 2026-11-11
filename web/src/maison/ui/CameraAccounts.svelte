<script>
  import { onMount } from 'svelte';
  import { login } from '../../lib/hub.svelte.js';
  import { t } from '../../lib/i18n.svelte.js';
  let cameras = $state([]);
  let instance = $state('');
  let username = $state('');
  let password = $state('');
  let pin = $state('');
  let needPin = $state(false);
  let busy = $state(false);
  let message = $state('');
  let failed = $state(false);
  onMount(() => {
    const abort = new AbortController();
    fetch('/api/cameras/accounts', { signal: abort.signal }).then(async r => {
      if (!r.ok) throw new Error('accounts');
      cameras = (await r.json()).cameras;
      instance = cameras[0]?.instance ?? '';
    }).catch(() => { if (!abort.signal.aborted) failed = true; });
    return () => abort.abort();
  });
  async function save() {
    busy = true; message = '';
    try {
      if (needPin) {
        const error = await login(pin); pin = '';
        if (error) { message = t('camera.account.owner_failed'); return; }
      }
      const r = await fetch(`/api/cameras/${encodeURIComponent(instance)}/account`, { method:'PUT', headers:{'Content-Type':'application/json'}, body:JSON.stringify({username,password}) });
      if (r.status === 403) needPin = true;
      if (!r.ok) { message = (await r.json()).error ?? t('camera.account.failed'); return; }
      const camera = cameras.find(c => c.instance === instance);
      if (camera) camera.configured = true;
      username = ''; password = ''; needPin = false;
      message = t('camera.account.saved');
    } catch { message = t('camera.account.failed'); }
    finally { busy = false; }
  }
</script>

{#if cameras.length}
<section class="card">
  <h2>{t('camera.account.title')}</h2>
  <p class="muted">{t('camera.account.note')}</p>
  <form onsubmit={e => {e.preventDefault();save();}}>
    <label>{t('camera.account.camera')}<select bind:value={instance} onchange={() => {username='';password='';message='';}} disabled={busy}>{#each cameras as camera}<option value={camera.instance}>Tapo {camera.model} · {camera.host}{camera.configured ? ` · ${t('camera.account.configured')}` : ''}</option>{/each}</select></label>
    <label>{t('camera.account.username')}<input type="text" autocomplete="off" maxlength="128" required bind:value={username} disabled={busy}/></label>
    <label>{t('camera.account.password')}<input type="password" autocomplete="off" maxlength="256" required bind:value={password} disabled={busy}/></label>
    {#if needPin}<label>{t('energie.price.code')}<input type="password" autocomplete="off" inputmode="numeric" required bind:value={pin} disabled={busy}/></label>{/if}
    <button type="submit" disabled={busy}>{t(busy ? 'camera.account.testing' : 'camera.account.save')}</button>
  </form>
  {#if message}<p role="status">{message}</p>{/if}
</section>
{:else if failed}<p role="status">{t('camera.account.load_failed')}</p>{/if}

<style>
  h2 { margin:0 0 12px; font-size:18px; }
  form { display:flex; flex-wrap:wrap; gap:12px; align-items:end; }
  label { min-width:0; max-width:100%; display:grid; gap:8px; font-size:13px; }
  input,select,button { box-sizing:border-box; max-width:100%; min-height:44px; padding:8px 12px; border-radius:8px; border:1px solid var(--surface-3); background:var(--surface); color:var(--ink); }
  button { cursor:pointer; background:var(--ink); color:var(--surface); }
  p { font-size:13px; }
</style>

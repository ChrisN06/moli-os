<script>
  import { onMount } from 'svelte';
  import { login } from '../../lib/hub.svelte.js';
  import { t } from '../../lib/i18n.svelte.js';
  let configured=$state(false),pending=$state(false),busy=$state(false),needPin=$state(false),pin=$state(''),message=$state('');
  onMount(()=>{fetch('/api/freebox').then(r=>r.ok?r.json():null).then(v=>configured=!!v?.configured).catch(()=>{})});
  async function send(){if(busy)return;busy=true;message='';try{if(needPin&&!(await login(pin))){message=t('freebox.failed');return}const r=await fetch('/api/freebox/'+(pending?'confirm':'pair'),{method:'POST'});if(r.status===403)needPin=true;if(!r.ok)throw Error();const v=await r.json();pending=!!v.pending;if(v.configured)configured=true;message=t(pending?'freebox.pending':'freebox.saved')}catch{message=t('freebox.failed')}finally{busy=false;pin=''}}
</script>
<section class="card"><h2>{t('freebox.setup')}</h2><p>{t('freebox.setup_note')}</p>
  {#if configured}<p>{t('freebox.saved')}</p>{/if}
  <form onsubmit={event=>{event.preventDefault();send()}}>
    {#if needPin}<label>{t('freebox.pin')}<input type="password" bind:value={pin} autocomplete="off" required /></label>{/if}
    <button disabled={busy} type="submit">{t(pending?'freebox.confirm':'freebox.pair')}</button>
  </form>
  {#if message}<p role="status">{message}</p>{/if}
</section>
<style>h2{font-size:18px}p{margin:12px 0;color:var(--ink-2)}form{display:flex;gap:12px;flex-wrap:wrap;align-items:end}label{display:grid;gap:8px}input,button{min-height:44px;padding:8px 14px;background:var(--surface);color:var(--ink);border:1px solid var(--line);border-radius:10px}</style>

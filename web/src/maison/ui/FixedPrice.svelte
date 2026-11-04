<script>
  import { energy, refreshEnergy } from '../lib/energy-live.svelte.js';
  import { login } from '../../lib/hub.svelte.js';
  import { t } from '../../lib/i18n.svelte.js';
  let price = $state(undefined);
  let fee = $state(undefined);
  let { subscription = false } = $props();
  let pin = $state('');
  let needPin = $state(false);
  let busy = $state(false);
  let message = $state('');
  let initialized = false;
  $effect(() => {
    if (!initialized && energy.summary) {
      price = energy.summary.fixed_price;
      fee = energy.summary.monthly_fee ?? 0;
      initialized = true;
    }
  });
  async function save() {
    if (subscription ? (!Number.isFinite(fee) || fee < 0 || fee > 10000) : (!Number.isFinite(price) || price < 0 || price > 10)) { message = t(subscription ? 'energie.fee.invalid' : 'energie.price.invalid'); return; }
    busy = true;
    message = '';
    try {
      if (needPin) {
        const error = await login(pin);
        pin = '';
        if (error) { message = error; return; }
      }
      const r = await fetch(subscription ? '/api/energy/subscription' : '/api/energy/price', { method: 'PUT', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify(subscription ? {monthly_fee: fee} : { price }) });
      if (r.status === 403) needPin = true;
      if (!r.ok) { message = (await r.json()).error ?? t('energie.price.failed'); return; }
      await refreshEnergy();
      message = t('energie.price.saved');
      needPin = false;
    } catch { message = t('energie.price.failed'); }
    finally { busy = false; }
  }
</script>

<section class="card">
  <h2>{t(subscription ? 'energie.fee.title' : 'energie.price.title')}</h2>
  <form onsubmit={(e) => { e.preventDefault(); save(); }}>
    {#if subscription}
      <label>{t('energie.fee.label')}<input type="number" min="0" max="10000" step="0.01" inputmode="decimal" required bind:value={fee} /></label>
    {:else}
    <label>{t('energie.price.label', { currency: energy.summary?.currency ?? '€' })}
      <input type="number" min="0" max="10" step="any" inputmode="decimal" required bind:value={price} />
    </label>
    {/if}
    {#if needPin}<label>{t('energie.price.code')}<input type="password" inputmode="numeric" autocomplete="off" bind:value={pin} required /></label>{/if}
    <button type="submit" disabled={busy}>{t('energie.price.save')}</button>
  </form>
  <p class="muted">{t(subscription ? 'energie.fee.note' : 'energie.price.estimate')}</p>
  {#if !subscription && energy.summary?.fixed_price_since}<p class="muted">{t('energie.price.since', { date: new Date(energy.summary.fixed_price_since).toLocaleString() })}</p>{/if}
  {#if message}<p role="status">{message}</p>{/if}
</section>

<style>
  h2 { font-size: 17px; margin: 0 0 16px; }
  form { display: flex; flex-wrap: wrap; align-items: end; gap: 12px; }
  label { display: grid; gap: 8px; font-size: 13px; }
  input { max-width: 170px; min-height: 44px; padding: 8px 12px; border: 1px solid var(--ink-3); border-radius: 10px; background: var(--surface); color: var(--ink); }
  button { min-height: 44px; border: 0; border-radius: 10px; padding: 8px 18px; background: var(--ink); color: var(--surface); }
  p { font-size: 13px; margin-bottom: 0; }
</style>

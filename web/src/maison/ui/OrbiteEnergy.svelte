<script>
  import { onMount } from 'svelte';
  import { home } from '../lib/home.svelte.js';
  import { energy, useEnergy, kwh, euros } from '../lib/energy-live.svelte.js';
  import { subscriptionCost } from '../lib/subscription.js';
  import { deviceSeries } from '../lib/energy-series.js';
  import { t } from '../../lib/i18n.svelte.js';
  import Icon from './Icon.svelte';
  let report = $state(null), failed = $state(false);
  const summary = $derived(energy.summary);
  const parts = $derived(deviceSeries(report?.meters ?? []).map(series => ({...series, amount:(report?.buckets ?? []).reduce((total, bucket) => total + series.kwh(bucket), 0)})).filter(part => part.amount > 0));
  const total = $derived(parts.reduce((sum, part) => sum + part.amount, 0));
  onMount(useEnergy);
  onMount(() => {
    const controller = new AbortController();
    const refresh = async () => {
      if (document.visibilityState !== 'visible') return;
      try { const response = await fetch('/api/energy/series?step=day&count=1', {signal:controller.signal}); if (!response.ok) throw new Error(); report = await response.json(); failed = false; }
      catch { if (!controller.signal.aborted) failed = true; }
    };
    refresh(); const timer = setInterval(refresh, 60_000);
    return () => { controller.abort(); clearInterval(timer); };
  });
</script>
<a class="card orbite-energy" href="#/energie" aria-label={t('energie.carte.detail')}>
  <span class="label"><Icon name="bolt" size={20}/>{t('energie.carte.titre')}</span>
  {#if summary}<strong class="number num">{kwh(summary.today.total.kwh)} kWh</strong><span class="cost num">{euros(summary.today.total.cost == null ? null : summary.today.total.cost + subscriptionCost(summary.monthly_fee,summary.today.from,Math.min(home.now,summary.today.to),summary.timezone),summary.currency)}</span>{:else}<span class="muted">{t(energy.missing?'energie.aucun_compteur':'energie.chargement')}</span>{/if}
  {#if total > 0}<span class="stack" role="img" aria-label={t('orbite.period')}>{#each parts as part (part.id)}<i style:width={`${part.amount/total*100}%`} style:background={part.color}></i>{/each}</span><span class="legend">{#each parts as part (part.id)}<span><i style:background={part.color}></i>{part.name}{part.estimated?' ≈':''}</span>{/each}</span>{/if}
  {#if failed}<small>{t('orbite.load_failed')}</small>{/if}
</a>
<style>
.orbite-energy{display:flex;flex-direction:column;gap:12px;min-height:220px;color:var(--ink);text-decoration:none;background:radial-gradient(ellipse at 0 0,color-mix(in srgb,var(--good) 12%,transparent),transparent 70%),var(--surface)}.label{display:flex;gap:8px;align-items:center;color:var(--good);font-size:14px}.number{font-size:38px;font-weight:450;letter-spacing:-1px;line-height:1.1}.cost{color:var(--ink-2)}.stack{height:10px;display:flex;gap:3px;border-radius:10px;overflow:hidden;margin-top:6px}.stack i{height:100%}.legend{display:flex;flex-wrap:wrap;gap:6px 12px;font-size:11px;color:var(--ink-2)}.legend>span{display:inline-flex;gap:5px;align-items:center}.legend i{width:8px;height:8px;border-radius:2px;flex:none}small{color:var(--ink-2)}
@media(min-width:1800px){:global(.orbite:is([data-display=auto],[data-display=tv])) .number{font-size:3.6vw}:global(.orbite:is([data-display=auto],[data-display=tv])) .label,:global(.orbite:is([data-display=auto],[data-display=tv])) .legend{font-size:1.1vw}:global(.orbite:is([data-display=auto],[data-display=tv])) .stack{height:.9vw}}
</style>

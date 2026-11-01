<script>
  import { home, hub, device, reachable, relative, num } from '../lib/home.svelte.js';
  import { t } from '../../lib/i18n.svelte.js';
  import { watts, energy } from '../lib/energy-live.svelte.js';
  import { counterPower } from '../lib/counter-power.js';

  let { point } = $props();
  let series = $state(null);
  const id = $derived(point?.slice(0, point.indexOf('/')));
  const key = $derived(point?.slice(point.indexOf('/') + 1));
  const unit = $derived(device(id)?.points.find((p) => p.key === key)?.unit);
  const estimate = $derived(counterPower(series, unit, home.now, (energy.summary?.max_power_kw ?? 36) * 1000));
  const available = $derived(hub.connected && reachable(id));
  $effect(() => {
    series = null;
    if (!point) return;
    const controller = new AbortController();
    const url = `/api/history?point=${encodeURIComponent(point)}&hours=48&points=2000`;
    const refresh = async () => {
      try {
        const response = await fetch(url, { signal: controller.signal });
        if (!response.ok) throw new Error();
        const next = await response.json();
        if (!controller.signal.aborted) series = next;
      } catch { if (!controller.signal.aborted) series = null; }
    };
    refresh();
    const timer = setInterval(refresh, 60_000);
    return () => { clearInterval(timer); controller.abort(); };
  });
</script>

<div class="estimate" class:stale={estimate?.stale || !available}>
  <small>{t('energie.counter.title')}</small>
  <strong class="num">{estimate && available ? `≈ ${watts(estimate.watts)}` : '—'}</strong>
  {#if estimate && available}
    <span>{t('energie.counter.window', { minutes: num(estimate.duration / 60_000, 1) })}</span>
    <span>{t('energie.counter.received', { age: relative(estimate.end, home.now) })}</span>
    {#if estimate.stale}<b>{t('energie.counter.stale')}</b>{/if}
  {:else}
    <span>{t(available ? 'energie.counter.waiting' : 'energie.counter.offline')}</span>
  {/if}
</div>

<style>
  .estimate { display: grid; gap: 5px; }
  small, span { color: var(--ink-3); font-size: 12px; }
  strong { font-size: 34px; font-weight: 500; }
  b { color: var(--warm); font-size: 12px; }
  .stale strong { color: var(--ink-3); }
</style>

<script>
  import { hub, home, num } from '../lib/home.svelte.js';
  import { housePoint } from '../lib/readings.js';
  import { samples, readingAt, curve } from '../lib/climate-history.js';
  import { t } from '../../lib/i18n.svelte.js';
  let hours = $state(24), series = $state({}), failed = $state(false), loading = $state(true), cursor = $state(1000);
  const keys = ['temperature', 'humidity', 'reported_setpoint'];
  const refs = $derived(keys.map(key => housePoint(hub.devices, home.config, key)));
  const points = $derived(keys.map((key, i) => samples(series[key], i === 2)));
  const from = $derived(Object.values(series)[0]?.from ?? 0);
  const to = $derived(Object.values(series)[0]?.to ?? 1);
  const time = $derived(from + (to-from)*cursor/1000);
  const temperatures = $derived([...points[0], ...points[2]].map(p => p[1]));
  const min = $derived(temperatures.length ? Math.floor(Math.min(...temperatures)) - 1 : 15);
  const max = $derived(temperatures.length ? Math.ceil(Math.max(...temperatures)) + 1 : 25);
  const humidity = $derived(points[1].map(p => p[1]));
  const humidityMin = $derived(humidity.length ? Math.max(0, Math.floor(Math.min(...humidity)) - 1) : 0);
  const humidityMax = $derived(humidity.length ? Math.min(100, Math.ceil(Math.max(...humidity)) + 1) : 100);
  const date = ts => new Date(ts).toLocaleString(undefined, {day:'2-digit',month:'2-digit',hour:'2-digit',minute:'2-digit'});
  $effect(() => {
    const selected = [...refs], span = hours;
    const controller = new AbortController();
    series = {}; loading = true; failed = false;
    async function refresh() {
      if (document.visibilityState !== 'visible') return;
      const end = Date.now(), start = end-span*3600000;
      const results = await Promise.all(selected.map(async (point, i) => {
        if (!point) return [keys[i], null];
        try {
          const response = await fetch(`/api/history?point=${encodeURIComponent(point)}&from=${start}&to=${end}&points=2000`, {signal:controller.signal});
          if (!response.ok) throw new Error();
          return [keys[i], await response.json()];
        } catch { return [keys[i], null]; }
      }));
      if (controller.signal.aborted) return;
      series = Object.fromEntries(results.filter(([, value]) => value));
      failed = results.some(([, value], i) => selected[i] && !value);
      loading = false;
    }
    refresh();
    const timer = setInterval(refresh, 60000);
    return () => { clearInterval(timer); controller.abort(); };
  });
</script>
<section class="card climate-history">
  <header><h2>{t('salon.climate.title')}</h2><select aria-label={t('salon.climate.period')} bind:value={hours}><option value={24}>24 h</option><option value={168}>{t('salon.climate.days', {n:7})}</option><option value={720}>{t('salon.climate.days', {n:30})}</option></select></header>
  <p class="muted">{t('salon.climate.note')}</p>
  {#if failed}<p role="status">{t('moli.historique.indisponible')}</p>{/if}
  {#if loading}<p class="muted">…</p>
  {:else if !points.some(list => list.length)}<p class="muted">{t('moli.historique.aucun')}</p>
  {:else}
    <div class="legend">
      {#each keys as key, i}<span class="reading" style:--series-color={['#ed7d8b','#3296ff','var(--ink)'][i]}><b>{t(`salon.climate.${key}`)}</b><strong>{num(readingAt(points[i],time),i===1?0:1)} {i===1?'%':'°C'}</strong></span>{/each}
    </div>
    <div class="chart">
      <div class="scale"><span>{max} °C</span><span>{(max+min)/2} °C</span><span>{min} °C</span></div>
      <svg viewBox="0 0 800 240" preserveAspectRatio="none" role="img" aria-label={t('salon.climate.title')}>
        {#each [0,120,240] as y}<line x1="0" x2="800" y1={y} y2={y} class="grid"/>{/each}
        {#each points as list,i}<path d={curve(list,from,to,i===1?humidityMin:min,i===1?humidityMax:max,i===2)} stroke={['#ed7d8b','#3296ff','var(--ink)'][i]} />{/each}
        <line x1={cursor*.8} x2={cursor*.8} y1="0" y2="240" class="cursor"/>
      </svg>
      <div class="scale humidity"><span>{humidityMax} %</span><span>{(humidityMax + humidityMin) / 2} %</span><span>{humidityMin} %</span></div>
    </div>
    <input class="scrub" type="range" min="0" max="1000" step="1" bind:value={cursor} aria-label={t('salon.climate.inspect')} aria-valuetext={date(time)} />
    <div class="dates"><span>{date(from)}</span><strong>{date(time)}</strong><span>{date(to)}</span></div>
    {#if series.reported_setpoint?.buckets?.length}<p class="muted">{t('salon.climate.setpoint_dense')}</p>{/if}
  {/if}
</section>
<style>
  header,.legend,.dates { display:flex; justify-content:space-between; gap:16px; flex-wrap:wrap; } h2 { margin:0; } select { min-height:44px; background:var(--surface); color:var(--ink); } .reading { display:grid; gap:4px; border-left:3px solid var(--series-color); padding-left:10px; } .chart { display:grid; grid-template-columns:48px minmax(0,1fr) 48px; gap:8px; margin-top:24px; } .scale { display:flex; flex-direction:column; justify-content:space-between; font-size:11px; color:#ed7d8b; } .humidity { color:#3296ff; text-align:right; } svg { display:block; width:100%; height:240px; overflow:visible; } path { fill:none; stroke-width:2.5; vector-effect:non-scaling-stroke; } .grid { stroke:var(--ink); opacity:.15; vector-effect:non-scaling-stroke; } .cursor { stroke:var(--ink); opacity:.4; stroke-dasharray:4 4; vector-effect:non-scaling-stroke; } .scrub { display:block; width:calc(100% - 112px); margin:0 56px; min-height:44px; } .dates { font-size:11px; } .climate-history { min-width:0; }
</style>

<script>
  import ConsumptionDial from './ConsumptionDial.svelte';
  import { deviceSeries } from '../lib/energy-series.js';
  import { hasElectricity, usualElectricity } from '../lib/consumption.js';
  import CounterPower from './CounterPower.svelte';
  import { subscriptionCost } from '../lib/subscription.js';
  import { onMount } from 'svelte';
  import { hub, home } from '../lib/home.svelte.js';
  import { energy, useEnergy, euros, kwh, watts, power, liveLook, billOf } from '../lib/energy-live.svelte.js';
  import { t } from '../../lib/i18n.svelte.js';
  import Icon from './Icon.svelte';

  onMount(useEnergy);

  const s = $derived(energy.summary);
  const current = (point, fallback) => {
    if (!point) return fallback ?? null;
    const i = point.indexOf('/');
    const v = hub.devices[point.slice(0, i)]?.state?.[point.slice(i + 1)]?.value;
    return typeof v === 'number' ? v : (fallback ?? null);
  };
  const total = $derived(s?.meters.find((m) => m.role === 'total' && m.power));
  const totalNow = $derived(total ? current(total.power, total.power_w) : null);
  const counter = $derived(s?.meters.find((m) => m.role === 'total' && !m.power));

  let report = $state(null);
  onMount(() => {
    const controller = new AbortController();
    async function refresh() {
      if (document.hidden) return;
      try {
        const response = await fetch('/api/energy/series?step=hour&count=217', { signal: controller.signal });
        if (response.ok) report = await response.json();
      } catch { /* The daily total remains available without a reference. */ }
    }
    refresh();
    const timer = setInterval(refresh, 10 * 60_000);
    return () => { controller.abort(); clearInterval(timer); };
  });
  const today = $derived(s && hasElectricity(s.meters, s.today) ? (s.today.consumption ?? s.today.total.kwh) : null);
  const usual = $derived(usualElectricity(report, s?.today, home.now));
  const parts = $derived(s ? deviceSeries(s.meters)
    .map(part => ({ ...part, amount: part.kwh(s.today) }))
    .filter(part => part.amount > 0) : []);
  const partsTotal = $derived(parts.reduce((sum, part) => sum + part.amount, 0));
  const yesterday = $derived(s?.yesterday.total.kwh ?? null);

  // The supplier's meter right now, against the contract.
  const live = $derived(s?.live ? current(s.live.point, s.live.value) : null);
  const liveFrac = $derived(live != null && s?.live ? Math.min(1, Math.max(0, live / s.live.max)) : 0);
  // The colour says how much, not how close to the contract.
  const look = $derived(liveLook(live, s?.live?.warn));
  const liveHue = $derived(look.hue);
  const liveWord = $derived(look.word);

  // Who draws the most right now (circuits and appliances measured), as a
  // legend under the figures, when the meters say it.
  const DOTS = ['var(--warm)', 'var(--cool)', '#9b7bf0'];
  const top = $derived(
    (s?.meters ?? [])
      .filter((m) => (m.role === 'circuit' || m.role === 'appliance') && m.power)
      .map((m) => ({ id: m.id, name: m.estimated ? `${m.name} ≈` : m.name, w: current(m.power, m.power_w) }))
      .filter((m) => m.w != null && m.w >= 30)
      .sort((a, b) => b.w - a.w)
      .slice(0, 3),
  );

  const R = 70;
  const C = Math.PI * R;
  const isHc = $derived(/hc/i.test(s?.period ?? ''));
</script>

<a class="card energy" href="#/energie" aria-label={t('energie.carte.detail')}>
  <div class="card-head">
    <h2><Icon name="bolt" size={18} />{t('energie.carte.titre')}</h2>
    <span class="more" aria-hidden="true"><Icon name="arrow-top-right" size={18} /></span>
  </div>
  {#if s}
    {#if !s.live && counter}<CounterPower point={counter.point} />{/if}
    <div class="gauges" class:pair={s.live}>
      <ConsumptionDial value={today} {usual} unit="kWh" digits={1} reference="orbite.electricity_reference" />
      {#if s.live}
        <div class="gauge">
          <svg viewBox="0 0 180 104" aria-hidden="true">
            <path class="track" d="M 20 92 A {R} {R} 0 0 1 160 92" />
            <path class="fill" class:hued={liveHue != null} style:--h={liveHue} d="M 20 92 A {R} {R} 0 0 1 160 92" stroke-dasharray="{C * liveFrac} {C}" />
          </svg>
          <div class="value">
            <strong class="num" class:hued={liveHue != null} style:--h={liveHue}>{power(live, s.live.unit)}</strong>
            <span title={t('energie.abonnement', { power: power(s.live.max, s.live.unit) })}>{t('energie.carte.en_direct', { level: liveWord })}</span>
          </div>
        </div>
      {/if}
    </div>
    {#if partsTotal > 0}
      <div class="stack" aria-hidden="true">
        {#each parts as part (part.id)}<i style:width={`${part.amount / partsTotal * 100}%`} style:background={part.color}></i>{/each}
      </div>
      <div class="legend">
        {#each parts as part (part.id)}<span><i style:background={part.color}></i>{part.name}{part.estimated ? ' ≈' : ''}</span>{/each}
      </div>
    {/if}
    <div class="facts">
      <div><small>{t('energie.carte.cout_jour')}</small><b class="num">{euros(s.today.total.cost == null ? null : s.today.total.cost + subscriptionCost(s.monthly_fee, s.today.from, Math.min(home.now,s.today.to), s.timezone), s.currency)}</b></div>
      {#if s.live}
        <div title={billOf(s) != null ? t('energie.carte.facture', { bill: euros(billOf(s), s.currency) }) : null}>
          <small>{t('energie.span.month')}</small><b class="num">{euros(s.month.total.cost, s.currency)}</b>
          {#if billOf(s) != null}<em class="num">→ ~{Math.round(billOf(s))} {s.currency}</em>{/if}
        </div>
      {:else}
        <div><small>{t('energie.now.titre')}</small><b class="num">{watts(totalNow)}</b></div>
      {/if}
      <div><small>{t('energie.span.yesterday')}</small><b class="num">{kwh(yesterday)} kWh</b></div>
    </div>
    {#if top.length}
      <p class="top" aria-label={t('energie.carte.plus_gros')}>
        <span class="muted">{t('energie.now.titre')}</span>
        {#each top as who, i (who.id)}
          <span class="who"><i style:background={DOTS[i]}></i>{who.name}<b class="num">{watts(who.w)}</b></span>
        {/each}
      </p>
    {/if}
    {#if s.period}
      <span class="chip {isHc ? 'cool' : 'warm'}">{isHc ? t('energie.hc') : t('energie.hp')}</span>
    {/if}
  {:else}
    <p class="muted">{energy.missing ? t('energie.aucun_compteur') : t('energie.chargement')}</p>
  {/if}
</a>

<style>
  .stack { display: flex; height: 8px; border-radius: 8px; overflow: hidden; margin: 12px 0 6px; }
  .legend { display: flex; flex-wrap: wrap; gap: 4px 12px; font-size: .78em; color: var(--ink-2); }
  .legend span { display: inline-flex; align-items: center; gap: 5px; }
  .legend i { width: 8px; height: 8px; border-radius: 2px; flex: none; }

  .energy {
    display: block;
    color: inherit;
    text-decoration: none;
    transition: transform 0.2s var(--ease);
  }

  .energy:hover {
    transform: translateY(-2px);
  }

  .gauges {
    display: grid;
    gap: 12px;
  }

  .gauges.pair {
    grid-template-columns: 1fr 1fr;
  }

  .gauge {
    position: relative;
    max-width: 240px;
    width: 100%;
    margin: 0 auto;
  }

  svg {
    width: 100%;
    display: block;
  }

  .track,
  .fill {
    fill: none;
    stroke-width: 14;
    stroke-linecap: round;
  }

  .track {
    stroke: var(--surface-3);
  }

  .fill {
    stroke: var(--warm);
    transition: stroke-dasharray 0.8s var(--ease), stroke 0.4s var(--ease);
  }

  /* The live gauge's colour (hue from the script), lighter at night. */
  .fill.hued {
    stroke: hsl(var(--h) 72% 46%);
  }

  strong.hued {
    color: hsl(var(--h) 70% 38%);
  }

  :global(.maison[data-theme='night']) strong.hued {
    color: hsl(var(--h) 70% 62%);
  }

  .value {
    position: absolute;
    left: 0;
    right: 0;
    bottom: 0;
    display: grid;
    justify-items: center;
    text-align: center;
  }

  .value strong {
    font-size: 40px;
    font-weight: 300;
    letter-spacing: -0.03em;
    line-height: 1;
  }

  .pair .value strong {
    font-size: 28px;
  }

  .value span {
    font-size: 12px;
    color: var(--ink-3);
    font-weight: 600;
  }

  .pair .value span {
    font-size: 11px;
    line-height: 1.25;
  }

  .facts {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 8px;
    margin: 18px 0 14px;
    text-align: center;
  }

  .facts small {
    display: block;
    font-size: 11px;
    color: var(--ink-3);
    font-weight: 600;
  }

  .facts b {
    font-size: 15px;
    font-weight: 650;
  }

  .facts em {
    display: block;
    font-style: normal;
    font-size: 11px;
    font-weight: 600;
    color: var(--ink-3);
  }

  .top {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 6px 14px;
    margin-top: 16px;
    padding-top: 12px;
    border-top: 1px solid var(--line, color-mix(in srgb, var(--ink) 10%, transparent));
    font-size: 12.5px;
  }

  .who {
    display: inline-flex;
    align-items: center;
    gap: 6px;
  }

  .who i {
    width: 8px;
    height: 8px;
    border-radius: 50%;
  }

  .who b {
    font-weight: 650;
  }

  .chip {
    position: absolute;
    top: 20px;
    right: 64px;
  }
</style>

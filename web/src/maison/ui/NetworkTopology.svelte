<script>
  import { onMount, untrack } from 'svelte';
  import { freebox, useFreebox } from '../lib/freebox.svelte.js';
  import { appendTraffic, rate, rateText, stationRate, trafficPath, trafficScale, TRAFFIC_WINDOW } from '../lib/freebox-data.js';
  import { t, locale } from '../../lib/i18n.svelte.js';
  import Icon from './Icon.svelte';
  import TrafficWire from './network/TrafficWire.svelte';
  import TrafficRoutes from './network/TrafficRoutes.svelte';

  let samples = $state.raw([]);
  let now = $state(Date.now() / 1000);
  let hidden = $state(false);
  let paused = $state(false);
  let selected = $state(null);
  let selectedDevice = $state(null);
  let reduced = $state(false);
  let compact = $state(false);
  let sheet;
  let map = $state();
  const continuous = typeof CSS !== 'undefined' && CSS.supports('offset-path', 'path("M0 0 L1 1")');
  let scale = $state.raw({ ceiling: 200, lowerSince: null });

  onMount(() => {
    const stop = useFreebox();
    const preference = matchMedia('(prefers-reduced-motion: reduce)');
    const screen = matchMedia('(max-width: 760px)');
    const visibility = () => { hidden = document.hidden; now = Date.now() / 1000; };
    const motion = () => { reduced = preference.matches; };
    const timer = setInterval(() => { if (!document.hidden) now = Date.now() / 1000; }, 1000);
    const layout = () => {
      compact = screen.matches;
      if (!compact && sheet.open) sheet.close();
    };
    visibility();
    motion();
    layout();
    screen.addEventListener('change', layout);
    document.addEventListener('visibilitychange', visibility);
    preference.addEventListener('change', motion);
    return () => {
      clearInterval(timer);
      document.removeEventListener('visibilitychange', visibility);
      preference.removeEventListener('change', motion);
      screen.removeEventListener('change', layout);
      stop();
    };
  });

  $effect(() => {
    const data = freebox.data;
    // Only the Freebox timestamp identifies a new measurement.
    if (data) samples = appendTraffic(untrack(() => samples), data);
  });

  const data = $derived(freebox.data);
  const stale = $derived(!!data && (freebox.failed || now - data.timestamp > 15));
  const fresh = $derived(!!data && !stale);
  const connected = $derived(data?.connection?.state === 'up');
  const lineState = $derived(stale ? 'stale' : freebox.failed ? 'failed' : data ? (connected ? 'connected' : 'disconnected') : 'waiting');
  const down = $derived(fresh ? rate(data.connection?.rate_down) : null);
  const up = $derived(fresh ? rate(data.connection?.rate_up) : null);
  const moving = $derived(!paused && !hidden && !reduced && fresh);

  const orderDevices = devices => [...devices].sort((a, b) => String(a.mac).localeCompare(String(b.mac)));
  function wireless(id, label, devices, extra = {}) {
    return { id, label, icon: 'wifi', devices: orderDevices(devices), active: devices.length > 0,
      down: stationRate(devices, 'tx_rate'), up: stationRate(devices, 'rx_rate'), ...extra };
  }
  const ports = $derived([...data?.ports ?? []].sort((a, b) => a.id - b.id).map(port => ({
    id: port.id, label: t('freebox.ethernet', { id: port.id }), icon: 'lan',
    active: port.link === 'up', devices: orderDevices(port.mac_list),
    speed: port.speed > 0 ? `${port.speed} Mbit/s` : null,
    duplex: port.duplex === 'full' ? t('freebox.full') : port.duplex === 'half' ? t('freebox.half') : null,
    down: rate(port.stats.tx_bytes_rate), up: rate(port.stats.rx_bytes_rate),
  })));
  const repeaters = $derived((data?.repeaters ?? []).map(repeater => {
    const parent = ports.find(port => port.id === repeater.port && port.active);
    return wireless(repeater.id, repeater.hostname || t('freebox.repeater'), repeater.stations, {
      parent: parent?.id,
      logical: !parent,
      active: true,
      speed: repeater.connection === 'ethernet' && repeater.speed > 0 ? `${repeater.speed} Mbit/s` : null,
    });
  }).sort((a, b) => a.id.localeCompare(b.id)));
  const wifi = $derived(wireless('wifi', t('freebox.wifi_pop'), data?.wifi ?? []));
  const interfaces = $derived([...ports, wifi, ...repeaters]);
  const current = $derived(interfaces.find(group => group.id === selected));
  const device = $derived(current?.devices.find(item => item.mac === selectedDevice));

  const chartNow = $derived(Math.max(now, samples.at(-1)?.time ?? 0));
  const recent = $derived(samples.filter(sample => sample.time >= chartNow - TRAFFIC_WINDOW));
  const peak = $derived(Math.max(0, ...recent.flatMap(sample => [sample.down ?? 0, sample.up ?? 0])));
  $effect(() => { scale = trafficScale(untrack(() => scale), peak, now); });
  const chartMax = $derived(scale.ceiling);
  const displayRate = value => rateText(value, locale());
  const groupRate = (group, direction) => fresh && group.active ? group[direction] : null;
  const links = $derived(interfaces.map(group => ({
    id: group.id, parent: group.parent, logical: group.logical,
    down: groupRate(group, 'down'), up: groupRate(group, 'up'),
  })));
  const traffic = group => (groupRate(group, 'down') ?? 0) + (groupRate(group, 'up') ?? 0);
  const busiest = $derived(Math.max(0, ...interfaces.filter(group => !group.logical).map(traffic)));
  const groupState = group => t(group.logical ? 'freebox.unknown_link' : group.active ? 'freebox.connected' : 'freebox.disconnected');
  const groupNote = group => [groupState(group), group.active ? group.speed : null, group.active ? group.duplex : null].filter(Boolean).join(' · ');
  const deviceName = item => item.hostname || item.mac || t('freebox.device');
  const deviceCount = count => t(count === 1 ? 'freebox.one_device' : 'freebox.device_count', { count });

  function select(id) {
    selected = selected === id ? null : id;
    selectedDevice = null;
  }

  function inspect(item) {
    if (compact) {
      selectedDevice = item.mac;
      sheet.showModal();
    } else {
      selectedDevice = selectedDevice === item.mac ? null : item.mac;
    }
  }
</script>

{#snippet rates(downRate, upRate)}
  <span class="rates num">
    <span class="down" title={t('freebox.down')}>↓ {displayRate(downRate)}</span>
    <span class="up" title={t('freebox.up')}>↑ {displayRate(upRate)}</span>
  </span>
{/snippet}

{#snippet devices(group)}
  {#if group.devices.length}
    <div class="devices">
      {#each group.devices as item (item.mac)}
        <button type="button" aria-pressed={compact ? undefined : selectedDevice === item.mac} aria-haspopup={compact ? 'dialog' : undefined} onclick={() => inspect(item)}>
          <Icon name="server" size={16} />
          <span><b>{deviceName(item)}</b>{#if item.ip}<small>{item.ip}</small>{/if}</span>
        </button>
      {/each}
    </div>
  {:else}
    <p class="muted">{t('freebox.no_device')}</p>
  {/if}
{/snippet}

{#snippet deviceDetails(group)}
  {#if device}
    <dl class="device-info">
      <dt>IP</dt><dd>{device.ip ?? '—'}</dd>
      <dt>MAC</dt><dd>{device.mac ?? '—'}</dd>
      <dt>{t('freebox.link')}</dt><dd>{group.label}</dd>
      {#if device.port_speed > 0}<dt>{t('freebox.negotiated')}</dt><dd>{device.port_speed} Mbit/s</dd>{/if}
      {#if device.band}<dt>{t('freebox.band')}</dt><dd>{device.band === '5g' ? '5 GHz' : device.band === '2d4g' ? '2.4 GHz' : device.band}</dd>{/if}
      {#if device.signal != null}<dt>{t('freebox.signal')}</dt><dd>{device.signal} dBm</dd>{/if}
      {#if device.link_down > 0 || device.link_up > 0}
        <dt>{t('freebox.negotiated')}</dt><dd>{@render rates(device.link_down > 0 ? device.link_down * 125_000 : null, device.link_up > 0 ? device.link_up * 125_000 : null)}</dd>
      {/if}
      <dt>{t('freebox.current')}</dt><dd>{#if group.icon === 'wifi'}{@render rates(fresh ? device.tx_rate : null, fresh ? device.rx_rate : null)}{:else}{t('freebox.individual_missing')}{/if}</dd>
    </dl>
  {:else if selectedDevice}
    <p class="muted" role="status">{t('freebox.gone')}</p>
  {/if}
{/snippet}

{#snippet branch(group)}
  <div class="branch" class:busy={!group.logical && busiest > 0 && traffic(group) >= busiest * .7}>
    <div class="branch-wire">
      {#if !continuous}
        <TrafficWire down={group.logical ? null : groupRate(group, 'down')} up={group.logical ? null : groupRate(group, 'up')} paused={!moving} unknown={group.logical} />
      {/if}
    </div>
    <button class="node" data-traffic-link={group.id} type="button" aria-expanded={selected === group.id} onclick={() => select(group.id)}>
      <Icon name={group.icon} size={22} />
      <span><b>{group.label}</b><small>{group.active && group.speed && !group.logical ? group.speed : groupState(group)}</small>{@render rates(groupRate(group, 'down'), groupRate(group, 'up'))}</span>
      <span class="node-count num" title={deviceCount(group.devices.length)} aria-label={deviceCount(group.devices.length)}><Icon name="devices" size={15} />{group.devices.length}</span>
    </button>
    {#if compact && selected === group.id}<div class="inline-detail">{@render devices(group)}</div>{/if}
  </div>
{/snippet}

<div class="network">
  <header class="network-head">
    <div><h1 class="page-title">{t('freebox.title')}</h1><p class="page-sub">{t('freebox.subtitle')}</p></div>
    {#if data && !reduced}
      <button type="button" class="motion" aria-pressed={paused} onclick={() => paused = !paused}>{t(paused ? 'freebox.resume' : 'freebox.pause')}</button>
    {/if}
  </header>

  {#if freebox.configured === false}
    <section class="card"><p>{t('freebox.not_configured')}</p><a href="#/parametres">{t('freebox.settings')}</a></section>
  {:else}
    <div class="metrics">
      <div class="card metric"><span>{t('freebox.line')}</span><strong class:online={fresh && connected}><i></i>{t(`freebox.${lineState}`)}</strong></div>
      <div class="card metric"><span>{t('freebox.down')}</span><strong class="down num">↓ {displayRate(down)}</strong></div>
      <div class="card metric"><span>{t('freebox.up')}</span><strong class="up num">↑ {displayRate(up)}</strong></div>
      <div class="card metric">
        <span>{t('freebox.updated')}</span>
        <strong class="num">{data ? new Date(data.timestamp * 1000).toLocaleTimeString(locale()) : '—'}</strong>
        {#if data?.activeCount != null}<small>{deviceCount(data.activeCount)}</small>{/if}
      </div>
    </div>

    {#if data}
      <div class="workspace">
        <section class="card graph" aria-label={t('freebox.topology')}>
          <div class="card-head"><h2>{t('freebox.topology')}</h2></div>
          <div class="map" bind:this={map}>
            <div class="origin">
              <div class="anchor"><Icon name="internet" size={26} /><b>{t('freebox.internet')}</b><small>{data.connection?.media?.toUpperCase() ?? ''}</small></div>
              <TrafficWire down={connected ? down : null} up={connected ? up : null} paused={!moving} vertical />
              <div class="anchor gateway"><Icon name="router" size={26} /><b>Freebox</b><small>{t('freebox.gateway')}</small></div>
            </div>
            <div class="branches">
              {#each ports as port (port.id)}
                {@render branch(port)}
                {#each repeaters.filter(repeater => repeater.parent === port.id) as repeater (repeater.id)}
                  <div class="child">{@render branch(repeater)}</div>
                {/each}
              {/each}
              {@render branch(wifi)}
              {#each repeaters.filter(repeater => repeater.logical) as repeater (repeater.id)}
                {@render branch(repeater)}
              {/each}
            </div>
            {#if continuous}<TrafficRoutes {map} {links} {compact} paused={!moving} />{/if}
          </div>
          <p class="map-note muted">{t('freebox.unknown_wire')}</p>
        </section>

        <div class="network-side">
          <section class="card chart">
            <div class="card-head"><h2>{t('freebox.history')}</h2><small>{t('freebox.history_window')}</small></div>
            <div class="plot">
              <div class="chart-scale" aria-hidden="true"><span>{displayRate(chartMax)}</span><span>{displayRate(chartMax / 2)}</span><span>0</span></div>
              <svg class="traffic-chart" viewBox="0 0 300 96" preserveAspectRatio="none" role="img" aria-label={t('freebox.history_label')}>
                <path class="grid-lines" d="M0 3H300 M0 48H300 M0 93H300 M75 3V93 M150 3V93 M225 3V93" />
                <path class="line down" d={trafficPath(recent, 'down', chartNow, chartMax, 300, 96)} />
                <path class="line up" d={trafficPath(recent, 'up', chartNow, chartMax, 300, 96)} />
              </svg>
              <div class="chart-times"><span>−60 s</span><span>{t('freebox.now')}</span></div>
            </div>
            <div class="chart-legend">
              <span class="down">↓ {t('freebox.down')}</span><span class="up">↑ {t('freebox.up')}</span>
            </div>
            {#if !recent.some(sample => sample.down != null || sample.up != null)}<p class="muted">{t('freebox.waiting')}</p>{/if}
          </section>

          <aside class="card supervision">
            <div class="card-head"><h2>{t('freebox.interfaces')}</h2></div>
            {#if current}
              <section class="selected">
                <div class="card-head"><h2>{current.label}</h2><button type="button" class="close" aria-label={t('freebox.close')} onclick={() => select(null)}>×</button></div>
                {#if !compact}
                  {@render devices(current)}
                  {@render deviceDetails(current)}
                {/if}
              </section>
            {:else if selected != null}<p class="muted" role="status">{t('freebox.missing')}</p>{/if}
            {#each interfaces as group (group.id)}
              <button class="interface" type="button" aria-pressed={selected === group.id} onclick={() => select(group.id)}>
                <span><b>{group.label}</b><small>{groupNote(group)}</small></span>
                {@render rates(groupRate(group, 'down'), groupRate(group, 'up'))}
              </button>
            {/each}
            <p class="map-note muted">{t('freebox.rate_note')}</p>
          </aside>
        </div>
      </div>
    {/if}
  {/if}

  <dialog class="device-sheet" bind:this={sheet} aria-labelledby="network-device-title" onclose={() => selectedDevice = null}>
    <header>
      <h2 id="network-device-title">{device ? deviceName(device) : t('freebox.device')}</h2>
      <button class="close" type="button" aria-label={t('freebox.close')} onclick={() => sheet.close()}><Icon name="close" size={22} /></button>
    </header>
    {#if current}{@render deviceDetails(current)}{:else}<p role="status">{t('freebox.gone')}</p>{/if}
  </dialog>
</div>

<style>
  .network {
    --traffic-down: #14724d;
    --traffic-up: #b33045;
    --traffic-glow: #47748870;
    --traffic-core: #e9ffff;
    display: grid;
    gap: clamp(12px, 1.1vw, 28px);
    min-width: 0;
  }
  :global([data-theme=night]) .network {
    --traffic-down: #55e5a4;
    --traffic-up: #ff7d87;
    --traffic-glow: #b9ffefcf;
    --traffic-core: #f6ffff;
  }
  .network-head { display: flex; align-items: center; justify-content: space-between; gap: 12px; flex-wrap: wrap; }
  .network button {
    color: var(--ink);
    background: var(--surface-2);
    border: 1px solid var(--line);
    border-radius: 12px;
    font: inherit;
    cursor: pointer;
    min-height: 44px;
  }
  .motion { padding: 8px 14px; }
  .metrics { display: grid; grid-template-columns: repeat(4, minmax(0, 1fr)); gap: 12px; }
  .metric { display: grid; gap: 5px; align-content: start; padding: clamp(12px, 1vw, 32px); }
  .metric strong { font-size: clamp(1.05rem, 1.9vw, 3rem); line-height: 1.15; overflow-wrap: anywhere; }
  .metric span, small, .chart-scale { color: var(--ink-2); }
  .metric i { display: inline-block; width: .45em; height: .45em; margin-inline-end: .35em; vertical-align: .14em; border-radius: 50%; background: var(--ink-3); }
  .metric .online i { background: var(--traffic-down); }
  .down { color: var(--traffic-down); }
  .up { color: var(--traffic-up); }
  .rates { display: flex; flex-wrap: wrap; gap: 2px 10px; font-size: .85em; }
  .rates span { white-space: nowrap; }
  .workspace { display: grid; grid-template-columns: minmax(0, 2.6fr) minmax(260px, 1fr); grid-template-areas: 'map side'; gap: 12px; align-items: start; }
  .network-side { grid-area: side; display: grid; gap: 12px; min-width: 0; }
  .graph { grid-area: map; min-width: 0; }
  .chart, .supervision { min-width: 0; }
  .map { position: relative; display: grid; grid-template-columns: minmax(0, 1.15fr) minmax(0, 1fr); align-items: center; }
  .origin { display: grid; grid-template-columns: minmax(0, 1fr) minmax(40px, 1.15fr) minmax(0, 1fr) clamp(14px, 2.5vw, 76px); align-items: center; }
  .origin::after { content: ''; border-block-start: 1px solid var(--line); }
  .anchor { min-height: 88px; display: grid; align-content: center; justify-items: center; gap: 5px; padding: 8px 4px; color: var(--cool); border: 1px solid var(--line); border-radius: 14px; background: var(--surface-2); text-align: center; overflow-wrap: anywhere; }
  .anchor b { color: var(--ink); }
  .branches { border-inline-start: 1px solid var(--line); }
  .branch { display: grid; grid-template-columns: clamp(38px, 8vw, 285px) minmax(0, 1fr); align-items: center; margin-block: 8px; }
  .busy .node { border-color: color-mix(in srgb, var(--traffic-down) 42%, var(--line)); }
  .child { position: relative; margin-inline-start: clamp(28px, 4vw, 140px); border-inline-start: 1px solid var(--line); }
  .child::before { position: absolute; content: ''; inset-block-start: -8px; inset-inline-start: -1px; height: 8px; border-inline-start: 1px solid var(--line); }
  .node { display: grid; grid-template-columns: auto minmax(0, 1fr) auto; align-items: center; gap: 8px; width: 100%; min-width: 0; text-align: start; padding: 10px; }
  .node > :global(svg) { color: var(--cool); }
  .node[aria-expanded=true], .interface[aria-pressed=true], .devices button[aria-pressed=true] { border-color: var(--cool); background: var(--cool-soft); }
  .node-count { display: inline-flex; align-items: center; gap: 4px; color: var(--ink-2); font-size: .8em; }
  .node b, .interface b, .devices b { overflow-wrap: anywhere; }
  .node small, .interface small, .devices small { display: block; }
  .map-note { margin: 12px 0 0; font-size: .85em; }
  .traffic-chart { display: block; width: 100%; height: clamp(96px, 9vw, 310px); }
  .grid-lines { stroke: var(--line); fill: none; vector-effect: non-scaling-stroke; }
  .line { stroke: currentColor; fill: none; stroke-width: 2; stroke-linecap: round; vector-effect: non-scaling-stroke; }
  .plot { display: grid; grid-template-columns: auto minmax(0, 1fr); gap: 4px 9px; }
  .chart-scale { display: flex; flex-direction: column; justify-content: space-between; text-align: end; font-size: .72em; }
  .chart-times { grid-column: 2; display: flex; justify-content: space-between; color: var(--ink-2); font-size: .75em; }
  .chart-legend { display: flex; flex-wrap: wrap; gap: 5px 14px; margin-block-start: 10px; font-size: .8em; }
  .line.up { stroke-dasharray: 4 3; }
  .interface { display: flex; width: 100%; text-align: start; justify-content: space-between; gap: 6px 12px; align-items: center; flex-wrap: wrap; padding: 8px; margin-top: 6px; }
  .interface .rates { justify-content: end; }
  .selected { margin-bottom: 12px; border-bottom: 1px solid var(--line); padding-bottom: 12px; }
  .inline-detail { display: none; }
  .device-sheet {
    box-sizing: border-box;
    width: min(100%, 32rem);
    max-height: 85dvh;
    padding: 20px 18px max(20px, calc(14px + env(safe-area-inset-bottom)));
    color: var(--ink);
    background: var(--surface);
    border: 1px solid var(--line);
    border-radius: 22px;
    overflow: auto;
    overscroll-behavior: contain;
  }
  .device-sheet::backdrop { background: #0309179e; }
  .device-sheet header { display: flex; align-items: start; justify-content: space-between; gap: 12px; }
  .device-sheet h2 { margin: 8px 0; overflow-wrap: anywhere; }
  .device-sheet .close { display: grid; place-items: center; flex: none; }
  .close { min-width: 44px; }
  .devices { display: grid; gap: 6px; }
  .devices button { display: flex; align-items: center; gap: 8px; padding: 7px 9px; text-align: start; }
  .device-info { display: grid; grid-template-columns: minmax(0, auto) minmax(0, 1fr); gap: 6px 10px; margin-bottom: 0; }
  dt { color: var(--ink-2); }
  dd { margin: 0; overflow-wrap: anywhere; }
  @supports (offset-path: path("M0 0 L1 1")) {
    .branch-wire { visibility: hidden; }
    .branches, .child, .origin::after, .child::before { border-color: transparent; }
  }
  @media (max-width: 1100px) {
    .workspace { grid-template-columns: minmax(0, 2.25fr) minmax(220px, 1fr); }
    .map { grid-template-columns: minmax(0, 1fr) minmax(0, 1.2fr); }
    .origin { grid-template-columns: minmax(0, 1fr) 34px minmax(0, 1fr) 12px; }
    .anchor { font-size: .85em; }
  }
  @media (max-width: 760px) {
    .workspace { grid-template-columns: minmax(0, 1fr); grid-template-areas: 'side' 'map'; }
    .supervision { display: none; }
    .traffic-chart { height: 72px; }
    .metrics { grid-template-columns: repeat(2, minmax(0, 1fr)); gap: 8px; }
    .metric:last-child { grid-column: 2; grid-row: 1; }
    .metric:nth-child(2), .metric:nth-child(3) { grid-row: 2; }
    .metric { padding: 10px; }
    .map { grid-template-columns: minmax(0, 1fr); }
    .origin { grid-template-columns: minmax(0, 1fr); justify-items: start; padding-inline-start: 16px; }
    .anchor { display: flex; justify-content: start; align-items: center; gap: 9px; width: 100%; min-height: 48px; padding: 8px; font-size: inherit; }
    .origin::after { width: 16px; height: 12px; border-inline-start: 1px solid var(--line); margin-inline-start: -16px; }
    .branches { margin-top: -1px; }
    .branch { grid-template-columns: clamp(34px, 11vw, 68px) minmax(0, 1fr); margin-block: 6px; }
    .node { padding: 7px 8px; }
    .node .rates { font-size: .9em; }
    .device-sheet { width: 100%; max-width: none; margin: auto 0 0; border-radius: 22px 22px 0 0; }
    .inline-detail .devices { max-height: 16rem; overflow: auto; overscroll-behavior: contain; padding: 4px; }
    .child { margin-inline-start: 16px; }
    .inline-detail { display: block; grid-column: 2; padding: 6px; border: 1px solid var(--line); border-radius: 12px; margin-top: 6px; }
  }
</style>

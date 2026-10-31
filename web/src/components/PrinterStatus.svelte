<script>
  import { hub } from '../lib/hub.svelte.js';
  import { formatValue, unitOf } from '../lib/format.js';
  import { t } from '../lib/i18n.svelte.js';
  import { PHASES, phase, duration } from '../maison/lib/printers.js';

  let { device } = $props();
  const v = (key) => device.state[key]?.value;
  const state = $derived(phase(v('state'), null));
  const fields = ['progress', 'remaining', 'nozzle_temperature', 'bed_temperature'];
  const points = $derived(device.points.filter((p) => fields.includes(p.key)));
  const ams = $derived(Object.values(hub.devices).filter((d) => d.id.startsWith(`${device.id}_AMS_`) || d.id === `${device.id}_ExternalSpool`));
</script>

<section aria-label={t('commun.atelier.imprimante')}>
  <p class="state">{device.online === false ? t('systeme.appareil.hors_ligne') : state ? PHASES[state].label : (v('state') ?? '—')}</p>
  <dl>
    {#each points as point (point.key)}
      <div><dt>{point.label || point.key}</dt><dd>{point.key === 'remaining' ? duration(v(point.key)) : `${formatValue(point, v(point.key))}${v(point.key) == null ? '' : ` ${unitOf(point)}`}`}</dd></div>
    {/each}
  </dl>
  {#each ams as unit (unit.id)}
    <h3>{unit.label?.name || unit.native_name}</h3>
    <ul>
      {#each unit.points.filter((p) => /^slot_\d$|^spool$/.test(p.key)) as point (point.key)}
        <li><span>{point.label}</span><b>{unit.state[point.key]?.value ?? '—'}</b></li>
      {/each}
    </ul>
  {/each}
</section>

<style>
  section { padding: 12px; background: var(--bg); border-radius: var(--radius); }
  .state { margin: 0 0 12px; font-weight: 750; }
  dl { display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: 12px; margin: 0; }
  dt { color: var(--ink-2); font-size: 12px; }
  dd { margin: 4px 0 0; font-weight: 700; }
  h3 { margin: 16px 0 8px; font-size: 13px; }
  ul { list-style: none; padding: 0; margin: 0; display: grid; gap: 8px; }
  li { display: flex; flex-wrap: wrap; justify-content: space-between; gap: 6px; font-size: 13px; overflow-wrap: anywhere; }
</style>

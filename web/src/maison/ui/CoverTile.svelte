<script>
  import { onDestroy } from 'svelte';
  import { device, value, act, nameOf, num, hub, home, relative } from '../lib/home.svelte.js';
  import { t } from '../../lib/i18n.svelte.js';

  /** A shutter: open, stop, close (whatever the device offers). */
  let { id, name = null } = $props();

  const d = $derived(device(id));
  const label = $derived(nameOf(id, name));
  const pos = $derived(value(id, 'percent_control'));
  const position = $derived(d?.points.find((p) => p.key === 'percent_control'));
  const positionDisabled = $derived(!hub.connected || d?.online === false || !position?.access?.write || pos == null);
  let draft = $state(null);
  let settle;
  onDestroy(() => clearTimeout(settle));
  const shown = $derived(draft ?? pos ?? 0);
  const control = $derived(d?.points.find((p) => p.key === 'control'));
  const offers = $derived(control?.kind.values ?? []);
  const disabled = $derived(!hub.connected || d?.online === false || !control?.access?.write);
  const received = $derived(value(id, 'last_received'));
  const moving = $derived(`${id}/control` in hub.pending || `${id}/percent_control` in hub.pending);
  function setPosition() {
    const target = draft;
    clearTimeout(settle);
    if (positionDisabled || target == null || target === pos) { draft = null; return; }
    settle = setTimeout(() => {
      draft = null;
      if (!positionDisabled) act(`${id}/percent_control`, target, t('commun.volet.ouvert_a', { pct: target }));
    }, 300);
  }
  // [command sent to the device, catalogue key of its button]
  const BUTTONS = [
    ['open', 'commun.volet.ouvrir'],
    ['stop', 'commun.volet.stop'],
    ['close', 'commun.volet.fermer'],
  ];
</script>

<div class="cover">
  <div class="shutter" class:unknown={pos == null}>
    <div class="slats" style:height="{100 - shown}%"></div>
    <input type="range" min="0" max="100" step="1" value={shown} disabled={positionDisabled}
      aria-orientation="vertical"
      aria-label={t('commun.volet.regler', { name: label })}
      aria-valuetext={pos == null ? t('commun.volet.inconnue') : t('commun.volet.ouvert_a', { pct: shown })}
      oninput={(e) => { clearTimeout(settle); draft = Number(e.currentTarget.value); }} onchange={setPosition}
      onpointercancel={() => { clearTimeout(settle); draft = null; }} onblur={() => (draft = null)} />
  </div>
  <span class="name">{label}<small class="muted">{pos != null ? t('commun.volet.ouvert_a', { pct: num(pos) }) : ''}</small>
    {#if pos == null}<small>{t('commun.volet.inconnue')}</small>{/if}
    {#if draft != null}<small>{t('commun.volet.cible', { pct: draft, actual: num(pos) })}</small>{/if}
    {#if !position?.access?.write && !disabled}<small>{t('commun.volet.position_non_supportee')}</small>{/if}
    {#if d?.online === false}<small>{t('commun.volet.indisponible')}</small>{/if}
    {#if received != null}<small>{t('commun.volet.reception', { age: relative(received * 1000, home.now) })}</small>{/if}
    {#if moving}<small role="status">{t('commun.volet.commande_envoyee')}</small>{/if}</span>
  {#each BUTTONS.filter(([cmd]) => offers.includes(cmd)) as [cmd, key] (cmd)}
    <button disabled={disabled} onclick={() => { clearTimeout(settle); draft = null; act(`${id}/control`, cmd, t('commun.action', { label, action: t(key).toLowerCase() })); }}>{t(key)}</button>
  {/each}
</div>

<style>
  .shutter { position: relative; width: 96px; height: 134px; flex: none; border: 5px solid var(--ink-3); border-radius: 6px; overflow: hidden; background: linear-gradient(160deg, color-mix(in srgb, var(--cool) 28%, var(--surface)), var(--surface)); }
  .slats { position: absolute; inset: 0 0 auto; background: repeating-linear-gradient(to bottom, var(--ink-3) 0 7px, var(--ink-2) 7px 9px); border-bottom: 4px solid var(--ink-2); pointer-events: none; }
  .unknown { opacity: .4; }
  input { position: absolute; width: 124px; height: 86px; left: -19px; top: 19px; transform: rotate(-90deg); opacity: .01; cursor: ns-resize; }
  .shutter:has(input:focus-visible) { outline: 3px solid var(--cool); outline-offset: 3px; }
  input:disabled { cursor: default; }
  input:not(:disabled) { touch-action: none; }
  .cover {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 12px 14px;
    background: var(--surface-2);
    border-radius: var(--r-md);
    color: var(--ink-2);
    flex-wrap: wrap;
  }

  .name {
    flex: 1;
    display: grid;
    font-weight: 650;
    color: var(--ink);
    min-width: 120px;
  }

  small {
    font-weight: 500;
  }

  button {
    border: 0;
    background: var(--surface);
    border-radius: 999px;
    padding: 8px 12px;
    min-height: 44px;
    min-width: 44px;
    font-size: 13px;
    font-weight: 650;
    box-shadow: var(--shadow);
  }
</style>

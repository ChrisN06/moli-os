<script>
  import { onMount } from 'svelte';
  import { looks, loadAmbiances, setLook, home, note } from '../lib/home.svelte.js';
  import { waitForLiveLights } from '../lib/live-light.js';
  import { t } from '../../lib/i18n.svelte.js';
  import Icon from './Icon.svelte';

  /** A strip of ambiances for these lights (a room): one tap, the room
   *  takes the look. Lamps that cannot follow are left alone by Moli. */
  let { lights, label } = $props();

  let saved = $state(false);
  let personalBusy = $state(false);
  const scope = $derived(JSON.stringify([...lights].sort()));
  $effect(() => {
    const abort = new AbortController();
    saved = false;
    if (lights.length) fetch(`/api/light-preset?lights=${encodeURIComponent(scope)}`, { signal: abort.signal })
      .then(async r => { if (r.ok) saved = (await r.json()).saved; }).catch(() => {});
    return () => abort.abort();
  });
  async function personal(save, ids = [...lights]) {
    personalBusy = true;
    try {
      await waitForLiveLights();
      const r = await fetch('/api/light-preset', { method: save ? 'PUT' : 'POST', headers: { 'Content-Type': 'application/json', 'x-moli-origin': 'ui' }, body: JSON.stringify({ lights: ids }) });
      const body = await r.json();
      if ((save && r.status === 403) || body.held?.length) {
        home.held = { label: t('preset.name'), reason: body.held?.[0]?.reason ?? body.error, custom: () => personal(save, ids) };
        return;
      }
      if (!r.ok) throw new Error(body.error ?? t('preset.failed'));
      if (body.failed?.length) throw new Error(body.failed[0].error);
      if (save) { saved = true; note(t('preset.saved'), 'info'); }
    } catch (e) { note(e.message ?? t('preset.failed'), 'error'); }
    finally { personalBusy = false; }
  }
  async function ambiance(a) {
    await waitForLiveLights();
    return setLook(lights, { ambiance: a.id }, t('commun.action', { label, action: a.name }));
  }

  onMount(loadAmbiances);

  // What the chip shows: the ambiance's colours, or its white.
  const swatch = (a) =>
    a.colors.length > 1
      ? `conic-gradient(${[...a.colors, a.colors[0]].join(', ')})`
      : (a.colors[0] ?? (a.white >= 4000 ? '#eef3ff' : '#ffd9a8'));
</script>

{#if looks.list?.length && lights.length}
  <div class="strip" role="group" aria-label={t('commun.ambiances', { label })}>
    <button class="look" onclick={() => personal(false)} disabled={personalBusy || looks.busy != null} title={t(saved ? 'preset.recall' : 'preset.missing')}>
      <span class="swatch personal">★</span><span class="name">{t('preset.name')}</span>
    </button>
    <button class="look" onclick={() => personal(true)} disabled={personalBusy || looks.busy != null} title={t('preset.note')}>
      <span class="swatch personal">＋</span><span class="name">{t(saved ? 'preset.replace' : 'preset.save')}</span>
    </button>
    {#each looks.list as a (a.id)}
      {@const busy = looks.busy === JSON.stringify({ ambiance: a.id })}
      <button class="look" class:busy onclick={() => ambiance(a)} disabled={personalBusy || looks.busy != null}>
        <span class="swatch" class:pale={!a.colors.length} style:background={swatch(a)}><Icon name={a.icon} size={16} /></span>
        <span class="name">{a.name}</span>
      </button>
    {/each}
  </div>
{/if}

<style>
  /* It scrolls within its parent, never widens it. */
  .strip {
    min-width: 0;
    max-width: 100%;
    display: flex;
    gap: 8px;
    overflow-x: auto;
    padding: 2px 2px 6px;
    scrollbar-width: thin;
    scroll-snap-type: x proximity;
  }

  .look {
    all: unset;
    box-sizing: border-box;
    flex: none;
    display: grid;
    justify-items: center;
    gap: 6px;
    width: 76px;
    padding: 8px 4px;
    border-radius: var(--r-md);
    background: var(--surface-2);
    cursor: pointer;
    scroll-snap-align: start;
    transition: transform 0.15s var(--ease), background 0.2s var(--ease);
  }

  .look:hover:not(:disabled) {
    transform: translateY(-2px);
  }

  .look:focus-visible {
    outline: 3px solid var(--cool);
    outline-offset: 2px;
  }

  .look:disabled {
    cursor: default;
  }

  .swatch {
    width: 38px;
    height: 38px;
    border-radius: 50%;
    display: grid;
    place-items: center;
    color: #fff;
    box-shadow: inset 0 0 0 1px rgb(0 0 0 / 12%);
  }

  .personal { background: var(--cool); font-size: 24px; }

  .swatch :global(svg) {
    filter: drop-shadow(0 1px 1px rgb(0 0 0 / 45%));
  }

  /* A white look: a dark icon on its pale swatch. */
  .swatch.pale {
    color: var(--ink-2);
  }

  .swatch.pale :global(svg) {
    filter: none;
  }

  .name {
    font-size: 12px;
    font-weight: 600;
    text-align: center;
    line-height: 1.2;
    color: var(--ink-2);
  }

  .busy .swatch {
    animation: pulse 0.9s ease-in-out infinite;
  }

  @keyframes pulse {
    50% {
      opacity: 0.5;
      transform: scale(0.92);
    }
  }
</style>

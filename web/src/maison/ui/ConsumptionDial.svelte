<script>
  import { num } from '../lib/home.svelte.js';
  import { t } from '../../lib/i18n.svelte.js';

  let { value, usual, unit, digits = 0, reference } = $props();
  const known = $derived(Number.isFinite(value));
  const comparable = $derived(known && Number.isFinite(usual));
  const upper = $derived(Math.max(usual * 2, value * 1.15) || 1);
  const angle = amount => -90 + 180 * amount / upper;
  const difference = $derived(comparable && usual > 0 ? Math.round((value / usual - 1) * 100) : null);
  const direction = $derived((difference ?? Math.sign(value - usual)) > 0 ? 'above' : (difference ?? Math.sign(value - usual)) < 0 ? 'below' : 'usual');
</script>

<div class="dial">
  <svg viewBox="0 0 180 105" aria-hidden="true">
    <path class="rim" d="M 20 89 A 70 70 0 0 1 160 89" />
    {#each [-90, -67.5, -45, -22.5, 0, 22.5, 45, 67.5, 90] as tick (tick)}
      <path class="tick" d="M 90 23 V 28" transform="rotate({tick} 90 89)" />
    {/each}
    {#if usual != null}
      <path class="reference" d="M 90 13 V 25" transform="rotate({angle(usual)} 90 89)" />
    {/if}
    {#if known}
      <path class="needle" class:above={comparable && direction === 'above'} class:below={comparable && direction === 'below'} d="M 90 34 L 94 90 L 86 90 Z" transform="rotate({angle(value)} 90 89)" />
    {/if}
    <circle cx="90" cy="89" r="6" fill="var(--ink)" />
  </svg>
  <strong class="num">{known ? `${num(value, digits)} ${unit}` : '—'}</strong>
  <small>{known ? t('energie.span.today') : t('orbite.consumption_pending')}</small>
  {#if comparable}
    <span class="comparison" class:above={direction === 'above'} class:below={direction === 'below'}>
      {t(`orbite.consumption_${direction}`)}{difference ? ` · ${difference > 0 ? '+' : '−'}${Math.abs(difference)} %` : ''}
    </span>
  {/if}
  <small class="reference-label">{usual == null ? t('orbite.consumption_no_reference') : t(reference, { value: `${num(usual, digits)} ${unit}` })}</small>
</div>

<style>
  .dial { display: grid; justify-items: center; gap: 4px; text-align: center; }
  svg { width: clamp(145px, 56%, 260px); max-width: 100%; display: block; }
  .rim { fill: none; stroke: var(--surface-3); stroke-width: 8; stroke-linecap: round; }
  .tick { stroke: var(--ink-3); stroke-width: 1; }
  .reference { stroke: var(--cool); stroke-width: 3; stroke-linecap: round; }
  .needle { fill: var(--ink); }
  .needle.above { fill: var(--warm-ink); }
  .needle.below { fill: var(--good); }
  strong { font-size: clamp(26px, 2.7vw, 58px); line-height: 1.06; }
  small { font-size: .85em; color: var(--ink-2); }
  .comparison { margin-block-start: 4px; font-size: .9em; font-weight: 650; }
  .comparison.above { color: var(--warm-ink); }
  .comparison.below { color: var(--good); }
  .reference-label { color: var(--cool); }
</style>

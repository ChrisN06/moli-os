<script>
  import { onMount } from 'svelte';
  import { freebox, useFreebox } from '../lib/freebox.svelte.js';
  import { rateText } from '../lib/freebox-data.js';
  import { t, locale } from '../../lib/i18n.svelte.js';
  onMount(useFreebox);
  const rate = bytes => rateText(freebox.failed ? null : bytes, locale());
</script>
<a class="card rates" href="#/reseau" aria-label={t('freebox.title')}>
  <span><small>{t('freebox.down')}</small><strong class="down">↓ {rate(freebox.data?.connection?.rate_down)}</strong></span>
  <span><small>{t('freebox.up')}</small><strong class="up">↑ {rate(freebox.data?.connection?.rate_up)}</strong></span>
</a>
<style>.rates{display:flex;gap:24px;justify-content:space-between;text-decoration:none}.rates span{display:grid;gap:6px}.rates small{color:var(--ink-2)}.rates strong{font-size:22px;font-variant-numeric:tabular-nums}.down{color:var(--good)}.up{color:var(--alert)}</style>

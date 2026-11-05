<script>
  import WaterImport from '../ui/WaterImport.svelte';
  import { onMount } from 'svelte';
  import { useEnergy, energy } from '../lib/energy-live.svelte.js';
  import { t } from '../../lib/i18n.svelte.js';
  import CameraDisplay from '../ui/CameraDisplay.svelte';
  import CameraAccounts from '../ui/CameraAccounts.svelte';
  import XiaomiAccounts from '../ui/XiaomiAccounts.svelte';
  import FreeboxSetup from '../ui/FreeboxSetup.svelte';
  import FixedPrice from '../ui/FixedPrice.svelte';
  onMount(useEnergy);
</script>

<div class="settings">
  <header><h1 class="page-title">{t('commun.nav.parametres')}</h1></header>
  {#if energy.summary && !energy.summary.period_point}<FixedPrice />{/if}
  {#if energy.summary}<FixedPrice subscription />{/if}
  <WaterImport />
  <FreeboxSetup />
  <CameraDisplay />
  <CameraAccounts />
  <XiaomiAccounts />
</div>

<style>.settings { display:grid; gap:22px; }</style>

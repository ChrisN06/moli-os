<script>
  import { hub, nameOf, deviceKind } from '../lib/home.svelte.js';
  import { cameraDisplay, setCameraRotation } from '../lib/camera-display.svelte.js';
  import { t } from '../../lib/i18n.svelte.js';
  const cameras = $derived(Object.values(hub.devices).filter(d => deviceKind(d) === 'camera'));
</script>
{#if cameras.length}<section class="card">
  <h2>{t('camera.display.title')}</h2>
  <p class="muted">{t('camera.display.note')}</p>
  {#each cameras as camera (camera.id)}
    <label>{nameOf(camera.id)} <select value={cameraDisplay[camera.id] ?? 0} onchange={e=>setCameraRotation(camera.id,Number(e.currentTarget.value))}>{#each [0,90,180,270] as angle}<option value={angle}>{angle}°</option>{/each}</select></label>
  {/each}
</section>{/if}
<style>label { display:flex; gap:16px; align-items:center; margin:12px 0; }select { min-height:44px; background:var(--surface); color:var(--ink); }</style>

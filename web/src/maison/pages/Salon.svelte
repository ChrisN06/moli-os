<script>
  import { home, hub, roomList, act, nameOf } from '../lib/home.svelte.js';
  import { t } from '../../lib/i18n.svelte.js';
  import ClimateHistory from '../ui/ClimateHistory.svelte';
  import CoverTile from '../ui/CoverTile.svelte';
  import Icon from '../ui/Icon.svelte';
  import LightTile from '../ui/LightTile.svelte';
  import AmbiancePicker from '../ui/AmbiancePicker.svelte';

  const covers = $derived(roomList().filter(r => (home.config?.salon?.cover_rooms ?? ['Salon', 'Cuisine']).includes(r.name)).flatMap(r => r.covers));
  const s = $derived(home.config?.salon ?? {});
  const lamps = $derived((s.lamps ?? []).filter((id) => hub.devices[id]));
  const group = $derived(hub.devices[s.room_light] ? s.room_light : null);
  const coversReady = $derived(covers.filter((cover) => hub.devices[cover.id]));
  function canControlCovers(command) {
    return hub.connected && coversReady.some((cover) => {
      const d = hub.devices[cover.id];
      return d?.online !== false && d?.points.some((point) => point.key === 'control' && point.access?.write && point.kind.values?.includes(command));
    });
  }
  function controlCovers(command) {
    for (const cover of coversReady) {
      const d = hub.devices[cover.id];
      if (d?.online === false) continue;
      const point = d?.points.find((candidate) => candidate.key === 'control' && candidate.access?.write && candidate.kind.values?.includes(command));
      if (point) act(`${cover.id}/control`, command, t('commun.action', { label: nameOf(cover.id, cover.name), action: t(`commun.volet.${command === 'open' ? 'ouvrir' : command === 'close' ? 'fermer' : 'stop'}`).toLowerCase() }));
    }
  }
</script>

<div class="salon">
  <header>
    <h1 class="page-title">{t('salon.titre')}</h1>
    <p class="page-sub">{t('salon.eclairage_description')}</p>
  </header>
  {#if covers.length}<section class="card covers"><div class="cover-actions"><h2>{t('salon.volets')}</h2><button disabled={!canControlCovers('open')} onclick={() => controlCovers('open')}>{t('commun.volet.ouvrir')}</button><button disabled={!canControlCovers('stop')} onclick={() => controlCovers('stop')}>{t('commun.volet.stop')}</button><button disabled={!canControlCovers('close')} onclick={() => controlCovers('close')}>{t('commun.volet.fermer')}</button></div>{#each covers as cover (cover.id)}<CoverTile id={cover.id} />{/each}</section>{/if}
  <div class="history"><ClimateHistory /></div>
  <section class="card lighting">
    <h2><Icon name="light-group" size={22} />{t('salon.ambiance')}</h2>
    {#if group || lamps.length}
      {#if group}<LightTile id={group} name={t('salon.tout_le_salon')} icon="sofa" dimmer members={lamps} />{/if}
      {#if lamps.length}
        <AmbiancePicker lights={lamps} label={t('salon.titre')} />
        <div class="lamps">
          {#each lamps as lamp (lamp)}<LightTile id={lamp} icon={home.config?.favorites?.lights?.find((l) => l.id === lamp)?.icon ?? 'light'} compact dimmer />{/each}
        </div>
      {/if}
    {:else}
      <p class="muted">{t('salon.eclairage_vide')}</p>
    {/if}
  </section>
</div>

<style>
  .salon { display: grid; gap: 20px; min-width: 0; }
  .salon > header { order: -1; }
  .salon > .lighting { order: 0; }
  .salon > .covers { order: 1; }
  .salon > .history { order: 3; min-width: 0; }
  .lighting { display: grid; gap: 20px; min-width: 0; padding: 24px; }
  .cover-actions { display: flex; flex-wrap: wrap; align-items: center; gap: 8px; margin-bottom: 12px; }
  .cover-actions h2 { margin-right: auto; }
  .cover-actions button { min-height: 44px; border: 0; border-radius: 999px; padding: 8px 14px; background: var(--surface); color: var(--ink); font-weight: 650; }
  .cover-actions button:disabled { opacity: .5; }
  h2 { display: flex; align-items: center; gap: 10px; margin: 0; }
  .lamps { display: grid; grid-template-columns: repeat(auto-fit, minmax(min(100%, 240px), 1fr)); gap: 12px; }
</style>

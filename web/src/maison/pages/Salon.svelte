<script>
  import { home, hub, roomList, num } from '../lib/home.svelte.js';
  import { t } from '../../lib/i18n.svelte.js';
  import { houseReading } from '../lib/readings.js';
  import CoverTile from '../ui/CoverTile.svelte';
  import Icon from '../ui/Icon.svelte';
  import LightTile from '../ui/LightTile.svelte';
  import AmbiancePicker from '../ui/AmbiancePicker.svelte';

  const temperature = $derived(houseReading(hub.devices, home.config, 'temperature'));
  const humidity = $derived(houseReading(hub.devices, home.config, 'humidity'));
  const covers = $derived(roomList().filter(r => (home.config?.salon?.cover_rooms ?? ['Salon', 'Cuisine']).includes(r.name)).flatMap(r => r.covers));
  const s = $derived(home.config?.salon ?? {});
  const lamps = $derived((s.lamps ?? []).filter((id) => hub.devices[id]));
  const group = $derived(hub.devices[s.room_light] ? s.room_light : null);
</script>

<div class="salon">
  <header>
    <h1 class="page-title">{t('salon.titre')}</h1>
    <p class="page-sub">{t('salon.eclairage_description')}</p>
  </header>
  <section class="card readings">
    <div><span>{t('maison.accueil.temperature_interieure')}</span><strong>{num(temperature, 1)} °C</strong></div>
    <div><span>{t('maison.accueil.humidite')}</span><strong>{num(humidity)} %</strong></div>
  </section>
  {#if covers.length}<section class="card lamps">{#each covers as cover (cover.id)}<CoverTile id={cover.id} />{/each}</section>{/if}
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
  .readings { display:flex; flex-wrap:wrap; gap:32px; }.readings div { display:grid; gap:8px; }
  .salon, .lighting { display: grid; gap: 20px; min-width: 0; }
  .lighting { padding: 24px; }
  h2 { display: flex; align-items: center; gap: 10px; margin: 0; }
  .lamps { display: grid; grid-template-columns: repeat(auto-fit, minmax(min(100%, 240px), 1fr)); gap: 12px; }
</style>

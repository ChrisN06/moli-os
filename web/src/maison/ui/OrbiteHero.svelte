<script>
  import { onMount } from 'svelte';
  import { home, hub, value, clock, greeting, longDate, num } from '../lib/home.svelte.js';
  import { houseReading } from '../lib/readings.js';
  import { weatherOf } from '../lib/icons.js';
  import { watchSky } from '../lib/sky.svelte.js';
  import { t } from '../../lib/i18n.svelte.js';
  import Icon from './Icon.svelte';
  import SkyDetail from './SkyDetail.svelte';
  const weather = $derived(home.config?.outdoor?.weather);
  const sky = $derived(weatherOf(value(weather, 'weather_code'), value(weather, 'daylight') !== false));
  const temperature = $derived(houseReading(hub.devices, home.config, 'temperature'));
  const humidity = $derived(houseReading(hub.devices, home.config, 'humidity'));
  let open = $state(false);
  onMount(watchSky);
</script>
<section class="orbite-hero">
  <div class="orbite-time"><p class="clock num">{clock(home.now)}</p><p class="date">{longDate(home.now)}</p></div>
  <div class="orbite-welcome"><h1>{greeting(home.now)}</h1>{#if weather}<button class="weather" onclick={() => open = true} aria-label={t('maison.meteo.ouvrir', { label: sky.label })}><Icon path={sky.icon} size={34}/><strong>{num(value(weather, 'temperature'), 0)}°</strong><span>{sky.label}</span></button>{/if}</div>
  <a class="orbite-climate" href="#/salon" aria-label={t('orbite.readings')}><strong class="num">{temperature == null ? '—' : `${num(temperature, 1)}°`}</strong><span>{t('orbite.readings')}</span><small>{t('orbite.humidity')} {humidity == null ? '—' : `${num(humidity)} %`}</small></a>
</section>
{#if open}<SkyDetail onclose={() => open = false}/>{/if}
<style>
.orbite-hero{display:grid;grid-template-columns:1.2fr 1fr .8fr;align-items:center;gap:28px;padding:20px 0 10px;min-width:0}.clock{font-size:clamp(72px,8vw,120px);line-height:1;letter-spacing:-6px;font-weight:300}.date{color:var(--ink-2);margin-top:14px}.orbite-welcome h1{font-size:30px;letter-spacing:-.8px;font-weight:450}.weather{display:flex;align-items:center;gap:12px;background:none;border:0;border-radius:16px;padding:12px 0;text-align:left;flex-wrap:wrap}.weather :global(svg){color:var(--warm-ink)}.weather strong{font-size:32px;font-weight:450}.weather span{color:var(--ink-2)}.orbite-climate{width:190px;aspect-ratio:1;max-width:100%;border-radius:50%;border:1px solid var(--line);display:flex;align-items:center;justify-content:center;flex-direction:column;gap:6px;position:relative;text-align:center;color:var(--ink-2);text-decoration:none;box-shadow:0 0 65px color-mix(in srgb,var(--cool) 9%,transparent);justify-self:end}.orbite-climate:before,.orbite-climate:after{content:'';position:absolute;border-radius:50%;pointer-events:none}.orbite-climate:before{inset:12px;border:1px solid var(--cool);opacity:.5}.orbite-climate:after{inset:25px;border:1px solid var(--line)}.orbite-climate strong{color:var(--cool);font-size:36px;font-weight:400;letter-spacing:-1px}.orbite-climate span,.orbite-climate small{font-size:11px;max-width:115px}
@media(max-width:760px){.orbite-hero{grid-template-columns:1fr auto;gap:20px;padding:8px 0}.orbite-time{grid-column:1/-1}.clock{font-size:76px}.orbite-welcome h1{font-size:26px}.orbite-climate{width:148px}.orbite-climate strong{font-size:30px}.orbite-climate:after{inset:20px}.orbite-climate span,.orbite-climate small{font-size:10px;max-width:100px}.weather strong{font-size:28px}.weather span{font-size:13px}}
@media(min-width:1800px){:global(.orbite:is([data-display=auto],[data-display=tv])) .clock{font-size:10vw}:global(.orbite:is([data-display=auto],[data-display=tv])) .orbite-welcome h1{font-size:2.6vw}:global(.orbite:is([data-display=auto],[data-display=tv])) .orbite-climate{width:18vw}:global(.orbite:is([data-display=auto],[data-display=tv])) .orbite-climate strong{font-size:3.4vw}:global(.orbite:is([data-display=auto],[data-display=tv])) .orbite-climate span,:global(.orbite:is([data-display=auto],[data-display=tv])) .orbite-climate small{font-size:1vw;max-width:12vw}:global(.orbite:is([data-display=auto],[data-display=tv])) .weather strong{font-size:3vw}}
</style>

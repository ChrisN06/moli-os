<script>
  import { onMount } from 'svelte';
  import Icon from './Icon.svelte';
  import { t } from '../../lib/i18n.svelte.js';
  let { id, name, big = false, onopen = null, online = true } = $props();
  let video;
  let playing = $state(false);
  let failed = $state(false);
  let hls;
  let generation = 0;
  function stop() {
    generation++;
    hls?.destroy(); hls = null;
    if (video) { video.pause(); video.removeAttribute('src'); video.load(); }
    playing = false;
  }
  async function start() {
    stop(); failed = false;
    if (!online || document.visibilityState !== 'visible') return;
    playing = true;
    const run = generation;
    const src = `/api/devices/${encodeURIComponent(id)}/video/stream.m3u8`;
    try {
      const native = video.canPlayType('application/vnd.apple.mpegurl');
      // Chrome can advertise native HLS without playing this live fMP4 stream.
      if (native && /Version\/.*Safari\//.test(navigator.userAgent)) {
        video.src = src;
        await video.play();
      } else {
        const { default: Hls } = await import('hls.js/dist/hls.light.mjs');
        if (run !== generation) return;
        if (!Hls.isSupported()) {
          if (!native) throw new Error('unsupported');
          video.src = src;
          await video.play();
          return;
        }
        hls = new Hls({ maxBufferLength: 4, backBufferLength: 0, enableWorker: false });
        hls.on(Hls.Events.ERROR, (_, data) => { if (data.fatal && run === generation) { stop(); failed = true; } });
        hls.on(Hls.Events.MANIFEST_PARSED, () => {
          if (run === generation) video.play().catch(() => { if (run === generation) { stop(); failed = true; } });
        });
        hls.loadSource(src); hls.attachMedia(video);
      }
    } catch { if (run === generation) { stop(); failed = true; } }
  }

  $effect(() => { if (!online) stop(); });
  onMount(() => {
    if (big) start();
    const visibility = () => { if (document.visibilityState !== 'visible') stop(); };
    document.addEventListener('visibilitychange', visibility);
    return () => { document.removeEventListener('visibilitychange', visibility); stop(); };
  });
</script>

<div class="live-camera">
  <video class:idle={!playing} bind:this={video} controls={playing} muted playsinline preload="none" aria-label={name} onerror={() => { if (playing) { stop(); failed = true; } }}></video>
  {#if !playing}<div class="placeholder" aria-hidden="true"><Icon name="cctv" size={40}/></div>{/if}
  <div class="bar"><strong>{name}</strong>{#if playing}<button onclick={stop}>{t('camera.video.stop')}</button>{:else}<button disabled={!online} onclick={() => onopen ? onopen(id) : start()}>{t('camera.video.play')}</button>{/if}</div>
  {#if failed}<p role="status">{t('camera.video.failed')}</p>{/if}
</div>

<style>
  .live-camera { background:#111621; border-radius:var(--r-md); overflow:hidden; color:white; }
  video { display:block; width:100%; aspect-ratio:16/9; background:#111621; }
  video.idle { display:none; }
  .placeholder { aspect-ratio:16/9; display:grid; place-items:center; color:#8a93a8; }
  .bar { display:flex; align-items:center; justify-content:space-between; gap:12px; padding:12px; }
  button { color:#111621; background:#f1f4f8; min-height:44px; padding:8px 14px; border:0; border-radius:8px; cursor:pointer; }
  p { padding:0 12px 12px; font-size:13px; }
</style>

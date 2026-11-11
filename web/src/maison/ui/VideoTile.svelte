<script>
  import { onMount, untrack } from 'svelte';
  import { cameraDisplay } from '../lib/camera-display.svelte.js';
  import Icon from './Icon.svelte';
  import { t } from '../../lib/i18n.svelte.js';
  let { id, name, big = false, autoplay = false, onopen = null, online = true } = $props();
  const rotation = $derived([0,90,180,270].includes(cameraDisplay[id]) ? cameraDisplay[id] : 0);
  let video;
  let frame;
  let inView = $state(true);
  let frameWidth = $state(0);
  let ratio = $state(16/9);
  let mounted = $state(false);
  let playing = $state(false);
  let failed = $state(false);
  let showControls = $state(false);
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
    video.muted = true;
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

  $effect(() => {
    const auto = mounted && online && inView && (autoplay || big);
    untrack(() => { if (auto) start(); else stop(); });
  });
  onMount(() => {
    mounted = true;
    const observer = new IntersectionObserver(entries => { inView = entries.some(entry => entry.isIntersecting); });
    observer.observe(frame);
    const visibility = () => { if (document.visibilityState !== 'visible') stop(); else if (online && inView && (autoplay || big)) start(); };
    document.addEventListener('visibilitychange', visibility);
    return () => { observer.disconnect(); document.removeEventListener('visibilitychange', visibility); stop(); };
  });
</script>

<div class="live-camera">
  <button class="stream" bind:this={frame} bind:clientWidth={frameWidth} style:aspect-ratio={rotation%180 ? 1/ratio : ratio} onclick={() => { if (!playing) start(); else if (onopen) onopen(id); else showControls = true; }} aria-label={name}>
  <video style:width={rotation%180 ? `${frameWidth*ratio}px` : '100%'} style:height={rotation%180 ? `${frameWidth}px` : '100%'} style:transform={`translate(-50%, -50%) rotate(${rotation}deg)`} onloadedmetadata={() => { if(video.videoWidth && video.videoHeight) ratio=video.videoWidth/video.videoHeight; }} class:idle={!playing} bind:this={video} controls={playing && showControls} muted playsinline preload="none" aria-label={name} onerror={() => { if (playing) { stop(); failed = true; } }}></video>
  {#if !playing}<div class="placeholder" aria-hidden="true"><Icon name="cctv" size={40}/></div>{/if}
  </button>
  {#if failed}<p role="status">{t('camera.video.failed')}</p>{/if}
</div>

<style>
  .live-camera { background:#111621; border-radius:var(--r-md); overflow:hidden; color:white; }
  video { position:absolute; left:50%; top:50%; display:block; width:100%; object-fit:contain; background:#111621; }
  video.idle { display:none; }
  .placeholder { aspect-ratio:16/9; display:grid; place-items:center; color:#8a93a8; }
  .stream { position:relative; display:block; width:100%; padding:0; background:transparent; border:0; color:inherit; }
  button { color:#111621; background:#f1f4f8; min-height:44px; padding:8px 14px; border:0; border-radius:8px; cursor:pointer; }
  p { padding:0 12px 12px; font-size:13px; }
</style>

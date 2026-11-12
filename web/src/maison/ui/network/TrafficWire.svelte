<script>
  let { down = null, up = null, paused = false, unknown = false, vertical = false } = $props();
  const particleCount = bytes => bytes > 0 ? Math.min(7, 2 + Math.floor(Math.log10(1 + bytes / 500))) : 0;
  const speed = bytes => Math.max(1.1, 4.8 - Math.log10(1 + bytes) / 2);
</script>

<svg viewBox="0 0 100 24" preserveAspectRatio="none" class:vertical class:unknown aria-hidden="true">
  <path d="M0 12 H100" class="wire" vector-effect="non-scaling-stroke" />
  {#each [['down', down, 8], ['up', up, 16]] as [direction, bytes, y] (direction)}
    {#each Array(particleCount(bytes)) as _, index (index)}
      <circle cx={direction === 'down' ? 0 : 100} cy={y} r="2" class={direction} class:paused
        style:--duration={`${speed(bytes)}s`}
        style:--delay={`${-index / particleCount(bytes) * speed(bytes)}s`} />
    {/each}
  {/each}
</svg>

<style>
  svg { display: block; width: 100%; height: 24px; }
  .wire { stroke: var(--line); stroke-width: 1.5px; }
  .unknown .wire { stroke-dasharray: 3 4; }
  circle { filter: drop-shadow(0 0 3px var(--traffic-glow)); }
  .down { fill: var(--traffic-down); animation: down var(--duration) linear var(--delay) infinite; }
  .up { fill: var(--traffic-up); animation: up var(--duration) linear var(--delay) infinite; }
  .paused { animation-play-state: paused; }
  @keyframes down { to { transform: translateX(100px); } }
  @keyframes up { to { transform: translateX(-100px); } }
  @media (prefers-reduced-motion: reduce) { circle { display: none; animation: none; } }
  @media (max-width: 760px) {
    .vertical { width: 34px; height: 22px; transform: rotate(90deg); }
  }
</style>

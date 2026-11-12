<script>
  import { onMount } from 'svelte';
  import { flowProfile } from '../../lib/freebox-data.js';

  let { down = null, up = null, paused = false, unknown = false, vertical = false } = $props();
  let element;
  let visible = $state(false);
  const lanes = $derived([{ direction: 'down', ...flowProfile(down) }, { direction: 'up', ...flowProfile(up) }]);

  onMount(() => {
    const observer = new IntersectionObserver(([entry]) => { visible = entry.isIntersecting; });
    observer.observe(element);
    return () => observer.disconnect();
  });

  function travel(node, options) {
    const shift = options.direction === 'down' ? ['translateX(-100%)', 'translateX(0%)'] : ['translateX(100%)', 'translateX(0%)'];
    const duration = 4800;
    const animation = node.animate(shift.map(transform => ({ transform })), { duration, iterations: Infinity });
    animation.currentTime = (options.index * 0.61803398875 % 1) * duration;

    function update({ speed, stopped }) {
      // Changing the playback rate preserves the particle's position.
      animation.updatePlaybackRate(speed);
      if (stopped) animation.pause();
      else animation.play();
    }
    update(options);
    return { update, destroy: () => animation.cancel() };
  }
</script>

<div class="wire" class:vertical class:unknown bind:this={element} aria-hidden="true">
  <div class="track">
    {#each lanes as lane (lane.direction)}
      <div class="lane" class:active={lane.count > 0} class:up={lane.direction === 'up'}>
        {#each Array(lane.count) as _, index (index)}
          <i class="particle" use:travel={{ ...lane, index, stopped: paused || !visible }}></i>
        {/each}
      </div>
    {/each}
  </div>
</div>

<style>
  .wire { min-width: 0; position: relative; height: 36px; }
  .track { position: absolute; inset: 0; overflow: hidden; }
  .lane {
    --stream: var(--traffic-down);
    position: absolute;
    width: 100%;
    height: 1px;
    top: 36%;
    background: var(--line);
  }
  .lane.up { --stream: var(--traffic-up); top: 64%; }
  .lane.active { background: color-mix(in srgb, var(--stream) 32%, var(--line)); }
  .unknown .lane { background: repeating-linear-gradient(to right, var(--line) 0 4px, transparent 4px 8px); }
  .particle { position: absolute; left: 0; width: 100%; }
  .particle::after {
    content: '';
    position: absolute;
    top: -2px;
    right: -2px;
    width: clamp(15px, 15%, 40px);
    height: 5px;
    border-radius: 50%;
    background: linear-gradient(to right, transparent 0%, var(--stream) 82%, var(--traffic-core) 96%);
    box-shadow: 3px 0 8px -2px var(--traffic-glow);
  }
  .up .particle::after {
    right: auto;
    left: -2px;
    transform: scaleX(-1);
    box-shadow: 3px 0 8px -2px var(--traffic-glow);
  }
  @media (prefers-reduced-motion: reduce) { .particle { display: none; } }
  @media (max-width: 760px) {
    .particle::after { width: clamp(7px, 18%, 12px); }
    .vertical { width: 42px; height: 56px; }
    .vertical .track { width: 56px; height: 36px; inset: 50% auto auto 50%; transform: translate(-50%, -50%) rotate(90deg); }
  }
</style>

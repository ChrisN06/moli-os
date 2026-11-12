<script>
  import { flowProfile, flowRoute } from '../../lib/freebox-data.js';

  let { map, links, compact, paused } = $props();
  let geometry = $state.raw(null);
  let visible = $state(false);
  const topology = $derived(JSON.stringify(links.map(({ id, parent }) => [id, parent])));

  $effect(() => {
    if (!map) return;
    const root = map;
    const vertical = compact;
    const definitions = JSON.parse(topology);
    const gateway = root.querySelector('.gateway');
    const branches = root.querySelector('.branches');
    const targets = new Map([...root.querySelectorAll('[data-traffic-link]')].map(node => [node.dataset.trafficLink, node]));
    const measure = () => {
      const frame = root.getBoundingClientRect();
      const anchor = gateway.getBoundingClientRect();
      const bus = branches.getBoundingClientRect().left - frame.left;
      const paths = new Map();
      for (const [id, parent] of definitions) {
        const target = targets.get(String(id))?.getBoundingClientRect();
        const upstream = parent == null ? null : targets.get(String(parent))?.getBoundingClientRect();
        if (!target) continue;
        const end = [target.left - frame.left, target.top + target.height / 2 - frame.top];
        const source = upstream ?? anchor;
        const fromBelow = vertical || !!upstream;
        const x = (fromBelow ? source.left + 18 : source.right) - frame.left;
        const y = (fromBelow ? source.bottom : source.top + source.height / 2) - frame.top;
        const trunk = upstream ? Math.min(x, end[0] - 12) : bus;
        paths.set(id, {
          rail: flowRoute(x, y, trunk, end, 0),
          down: flowRoute(x, y, trunk, end, -4),
          up: flowRoute(x, y, trunk, end, 4, true),
        });
      }
      geometry = paths;
    };
    measure();
    const resize = new ResizeObserver(measure);
    for (const node of [root, gateway, branches, ...targets.values()]) resize.observe(node);
    const intersection = new IntersectionObserver(([entry]) => { visible = entry.isIntersecting; });
    intersection.observe(root);
    return () => { resize.disconnect(); intersection.disconnect(); };
  });

  function follow(node, options) {
    const animation = node.animate([{ offsetDistance: '0%' }, { offsetDistance: '100%' }], {
      duration: options.duration, iterations: Infinity,
    });
    animation.currentTime = options.index * 0.61803398875 % 1 * options.duration;
    function update({ duration, speed, stopped }) {
      const timing = animation.effect.getTiming();
      if (timing.duration !== duration) {
        const progress = animation.currentTime / timing.duration;
        animation.effect.updateTiming({ duration });
        animation.currentTime = progress * duration;
      }
      animation.updatePlaybackRate(speed);
      if (stopped) animation.pause();
      else animation.play();
    }
    update(options);
    return { update, destroy: () => animation.cancel() };
  }
</script>

{#if geometry}
  <div class="routes" aria-hidden="true">
    <svg>
      {#each links as link (link.id)}
        {@const path = geometry.get(link.id)}
        {#if path}
          <path d={path.rail.path} class:unknown={link.logical} />
        {/if}
      {/each}
    </svg>
    {#each links as link (link.id)}
      {@const path = geometry.get(link.id)}
      {#if path && !link.logical}
        {#each ['down', 'up'] as direction (direction)}
          {@const profile = flowProfile(link[direction])}
          {#each Array(profile.count) as _, index (index)}
            <i class="particle" class:upload={direction === 'up'} style:offset-path={`path("${path[direction].path}")`}
              use:follow={{ index, speed: profile.speed, duration: Math.max(1100, path[direction].length * 14), stopped: paused || !visible }}></i>
          {/each}
        {/each}
      {/if}
    {/each}
  </div>
{/if}

<style>
  .routes, svg { position: absolute; inset: 0; width: 100%; height: 100%; pointer-events: none; }
  path { fill: none; stroke: var(--line); stroke-width: 1.3px; }
  path.unknown { stroke-dasharray: 4 4; }
  .particle {
    --stream: var(--traffic-down);
    position: absolute;
    inset: 0 auto auto 0;
    width: 17px;
    height: 5px;
    border-radius: 50%;
    background: linear-gradient(to right, transparent, var(--stream) 76%, var(--traffic-core) 96%);
    box-shadow: 3px 0 8px -2px var(--traffic-glow);
    offset-anchor: 88% 50%;
  }
  .upload { --stream: var(--traffic-up); }
  @media (max-width: 760px) { .particle { width: 11px; } }
  @media (prefers-reduced-motion: reduce) { .particle { display: none; } }
</style>

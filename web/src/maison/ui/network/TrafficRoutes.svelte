<script>
  import { flowWidth, flowRoute } from '../../lib/freebox-data.js';

  let { map, links, compact, paused, selected } = $props();
  let geometry = $state.raw(null);
  let visible = $state(false);
  let slow = $state(false);
  const topology = $derived(JSON.stringify(links.map(({ id, parent, logical }) => [id, parent, !!logical])));
  const moving = $derived(visible && !paused && !slow);
  const highlighted = $derived(new Set(selected == null ? [] : [
    selected, links.find(link => link.id === selected)?.parent,
  ]));
  const animated = $derived(links.filter(link =>
    (selected == null || highlighted.has(link.id)) && (link.down > 0 || link.up > 0),
  ).slice(0, 5).map(link => link.id));

  $effect(() => {
    if (!map) return;
    const root = map;
    const vertical = compact;
    const definitions = JSON.parse(topology);
    const internet = root.querySelector('.internet');
    const gateway = root.querySelector('.gateway');
    const targets = new Map([...root.querySelectorAll('[data-traffic-link]')].map(node => [node.dataset.trafficLink, node]));
    const measure = () => {
      const frame = root.getBoundingClientRect();
      const box = node => {
        const { left, top, width, height } = node.getBoundingClientRect();
        const x = left - frame.left;
        const y = top - frame.top;
        return { x, y, right: x + width, bottom: y + height, cx: x + width / 2, cy: y + height / 2, height };
      };
      const wan = box(internet);
      const hub = box(gateway);
      const paths = new Map();
      const lanes = (start, end, mode, offset = 6) => ({
        rail: flowRoute(start, end, mode),
        down: flowRoute(start, end, mode, -offset),
        up: flowRoute(start, end, mode, offset),
      });
      paths.set('internet', vertical
        ? lanes([hub.x + 21, wan.bottom], [hub.x + 21, hub.y], 'vertical')
        : lanes([wan.right, wan.cy], [hub.x, hub.cy], 'fan'));

      const ends = new Map([...targets].map(([id, node]) => [id, box(node)]));
      const direct = definitions.filter(([id, parent]) => id !== 'internet' && parent == null && ends.has(String(id)))
        .sort(([a], [b]) => ends.get(String(a)).cy - ends.get(String(b)).cy);
      const known = direct.filter(([, , logical]) => !logical);
      const gutter = known.length ? ends.get(String(known[0][0])).x - hub.x : 68;
      direct.forEach(([id, , logical], index) => {
        const end = ends.get(String(id));
        const startY = hub.y + hub.height * (index + 1) / (direct.length + 1);
        if (vertical) {
          // Each interface has its own lane back to the gateway. Earlier turns
          // use the rightmost lanes so they never cross later branches.
          const rank = known.findIndex(([key]) => key === id);
          const x = hub.x + (logical ? 4 : gutter - 12 - rank * (gutter - 24) / Math.max(1, known.length - 1));
          const start = [x, hub.bottom];
          const finish = [end.x, end.cy];
          paths.set(id, logical ? { rail: flowRoute(start, finish, 'elbow') } : lanes(start, finish, 'elbow', 3.5));
        } else {
          paths.set(id, lanes([hub.right, startY], [end.x, end.cy], 'fan'));
        }
      });
      for (const [id, parent] of definitions) {
        if (parent == null) continue;
        const source = ends.get(String(parent));
        const end = ends.get(String(id));
        if (source && end) {
          paths.set(id, { rail: flowRoute([source.x + 12, source.bottom], [end.x, end.cy], 'child') });
        }
      }
      geometry = paths;
    };
    measure();
    const resize = new ResizeObserver(measure);
    for (const node of [root, internet, gateway, ...targets.values()]) resize.observe(node);
    const intersection = new IntersectionObserver(([entry]) => { visible = entry.isIntersecting; });
    intersection.observe(root);
    return () => { resize.disconnect(); intersection.disconnect(); };
  });

  $effect(() => {
    if (!moving || !animated.length) return;
    let start;
    let frames = 0;
    let slowWindows = 0;
    let frame = requestAnimationFrame(sample);
    function sample(time) {
      start ??= time;
      if (time - start >= 1000) {
        slowWindows = frames * 1000 / (time - start) < 45 ? slowWindows + 1 : 0;
        if (slowWindows === 2) {
          slow = true;
          return;
        }
        start = time;
        frames = 0;
      }
      frames++;
      frame = requestAnimationFrame(sample);
    }
    return () => cancelAnimationFrame(frame);
  });
</script>

{#if geometry}
  <svg aria-hidden="true" class:quiet={!moving}>
    {#each links as link (link.id)}
      {@const path = geometry.get(link.id)}
      {#if path}
        {@const focus = highlighted.has(link.id)}
        {@const dim = selected != null && !focus}
        <g class:dim>
          <path class="rail" class:focus class:unknown={link.logical} d={path.rail} />
          {#each ['down', 'up'] as direction (direction)}
            {@const measured = flowWidth(link[direction])}
            {@const width = compact && link.id !== 'internet' && measured > 0 ? 1.5 + (measured - 1.5) * .58 : measured}
            {#if path[direction]}
              <path class="flow" class:upload={direction === 'up'} d={path[direction]} style:stroke-width={width} />
              {#if width && animated.includes(link.id)}
                <path class="pulse" class:upload={direction === 'up'} d={path[direction]} pathLength="100" style:stroke-width={Math.max(1, width * .38)} />
              {/if}
            {/if}
          {/each}
        </g>
      {/if}
    {/each}
  </svg>
{/if}

<style>
  svg { position: absolute; inset: 0; width: 100%; height: 100%; pointer-events: none; }
  path { fill: none; }
  .rail { stroke: var(--line); stroke-width: 1.25; }
  .rail.unknown { stroke-dasharray: 3 4; }
  .rail.focus { stroke: var(--cool); stroke-width: 2; }
  .dim { opacity: .28; }
  .flow { stroke: var(--traffic-down); transition: stroke-width .45s ease-out; }
  .flow.upload { stroke: var(--traffic-up); }
  .pulse {
    stroke: var(--traffic-core);
    stroke-linecap: round;
    stroke-dasharray: 5 95;
    animation: travel 3.4s linear infinite;
  }
  .pulse.upload { animation-direction: reverse; }
  .quiet .pulse { visibility: hidden; animation-play-state: paused; }
  @keyframes travel { to { stroke-dashoffset: -100; } }
  @media (prefers-reduced-motion: reduce) {
    .pulse { display: none; animation: none; }
    .flow { transition: none; }
  }
</style>

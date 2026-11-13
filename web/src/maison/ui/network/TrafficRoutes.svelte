<script>
  import { flowWidth, flowRoute } from '../../lib/freebox-data.js';

  let { map, links, compact, paused, selected } = $props();
  let geometry = $state.raw(null);
  let visible = $state(false);
  let slow = $state(false);
  const topology = $derived(JSON.stringify(links.map(({ id, parent }) => [id, parent])));
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
      const lanes = (start, end, mode) => ({
        rail: flowRoute(start, end, mode),
        down: flowRoute(start, end, mode, -6),
        up: flowRoute(start, end, mode, 6),
      });
      paths.set('internet', vertical
        ? lanes([hub.x + 21, wan.bottom], [hub.x + 21, hub.y], 'vertical')
        : lanes([wan.right, wan.cy], [hub.x, hub.cy], 'fan'));

      const ends = new Map([...targets].map(([id, node]) => [id, box(node)]));
      const direct = definitions.filter(([id, parent]) => id !== 'internet' && parent == null && ends.has(String(id)))
        .sort(([a], [b]) => ends.get(String(a)).cy - ends.get(String(b)).cy);
      const spineX = hub.x - 5;
      const spineY = hub.bottom + 6;
      const spine = vertical && direct.length
        ? `M${hub.x + 21} ${hub.bottom} V${spineY} H${spineX} V${ends.get(String(direct.at(-1)[0])).cy}`
        : null;
      direct.forEach(([id], index) => {
        const end = ends.get(String(id));
        const startY = hub.y + hub.height * (index + 1) / (direct.length + 1);
        paths.set(id, vertical
          ? lanes([spineX, end.cy], [end.x, end.cy], 'tap')
          : lanes([hub.right, startY], [end.x, end.cy], 'fan'));
      });
      for (const [id, parent] of definitions) {
        if (parent == null) continue;
        const source = ends.get(String(parent));
        const end = ends.get(String(id));
        if (source && end) {
          paths.set(id, { rail: flowRoute([source.x + 12, source.bottom], [end.x, end.cy], 'child') });
        }
      }
      const trunks = new Map(direct.map(([id]) => [id,
        `M${hub.x + 21} ${hub.bottom} V${spineY} H${spineX} V${ends.get(String(id)).cy}`,
      ]));
      geometry = { paths, spine, trunks };
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
    {#if geometry.spine}
      {@const trunk = links.find(link => highlighted.has(link.id) && link.parent == null && !link.logical)}
      <path class="rail" class:dim={selected != null} d={geometry.spine} />
      {#if trunk}<path class="rail focus" d={geometry.trunks.get(trunk.id)} />{/if}
    {/if}
    {#each links as link (link.id)}
      {@const path = geometry.paths.get(link.id)}
      {#if path}
        {@const focus = highlighted.has(link.id)}
        {@const dim = selected != null && !focus}
        <g class:dim>
          <path class="rail" class:focus class:unknown={link.logical} d={path.rail} />
          {#each ['down', 'up'] as direction (direction)}
            {@const width = flowWidth(link[direction])}
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

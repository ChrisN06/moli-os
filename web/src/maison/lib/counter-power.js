/** Average between real counter changes, never the hourly rows imported from CSV. */
export function counterPower(series, unit, now, maxWatts = 36_000) {
  const factor = unit === 'Wh' ? 1 : unit === 'kWh' ? 1000 : null;
  if (!factor || series?.truncated || series?.buckets?.length) return null;
  const points = [];
  for (const p of [series?.before, ...(series?.raw ?? [])]) {
    if (!p) continue;
    const [ts, value] = p;
    if (!Number.isFinite(ts) || !Number.isFinite(value) || value < 0) return null;
    const last = points.at(-1);
    if (last && ts <= last[0]) return null;
    if (!last || value !== last[1]) points.push(p);
  }
  if (points.length < 2) return null;
  const [a, b] = points.slice(-2);
  const duration = b[0] - a[0];
  const watts = ((b[1] - a[1]) * factor * 3_600_000) / duration;
  // Never infer a current draw from a whole day or exceed the recorder's configured ceiling.
  if (duration < 1000 || duration > 86_400_000 || b[0] > now + 60_000 || watts < 0 || watts > maxWatts) return null;
  return { watts, duration, end: b[0], stale: now - b[0] > 5_400_000 };
}

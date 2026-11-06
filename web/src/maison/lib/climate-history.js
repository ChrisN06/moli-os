/** Keep unknown history unknown; extend a recorded value only after its timestamp. */
export function samples(series, step = false) {
  if (!series || (step && series.buckets?.length)) return [];
  const numeric = ([ts, value]) => Number.isFinite(ts) && typeof value === 'number' && Number.isFinite(value);
  const points = series.buckets?.length
    ? series.buckets.map(b => [b.ts, b.avg]).filter(numeric)
    : (series.raw ?? []).filter(numeric);
  if (!series.buckets?.length && series.before && numeric(series.before)) points.unshift([series.from, series.before[1]]);
  if (points.length) points.push([series.to, points.at(-1)[1]]);
  return points;
}

export function readingAt(points, time) {
  for (let i = points.length - 1; i >= 0; i--) if (points[i][0] <= time) return points[i][1];
  return null;
}

export function curve(points, from, to, min, max, step = false) {
  if (!(to > from && max > min)) return '';
  const x = ts => ((ts - from) / (to - from) * 800).toFixed(2);
  const y = value => (240 - (value - min) / (max - min) * 240).toFixed(2);
  return points.map(([ts, value], i) => `${i ? (step ? `H${x(ts)}V` : `L${x(ts)},`) : `M${x(ts)},`}${y(value)}`).join(' ');
}

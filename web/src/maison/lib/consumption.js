const HOUR = 3_600_000;
const CONSUMPTION_ROLES = ['grid', 'total', 'circuit'];
const ROLES = [...CONSUMPTION_ROLES, 'solar', 'export'];

function mean(values) {
  return values.length >= 3 ? values.reduce((sum, value) => sum + value, 0) / values.length : null;
}

export function hasElectricity(meters, period) {
  return meters.some(meter => CONSUMPTION_ROLES.includes(meter.role) && Number.isFinite(period.meters[meter.id]?.kwh));
}

/** Compare against the same elapsed time on the previous seven days. */
export function usualElectricity(report, today, now) {
  if (!report || !today || now <= today.from || now >= today.to) return null;
  const date = new Intl.DateTimeFormat('en-CA', { timeZone: report.timezone, dateStyle: 'short' });
  const days = new Map();
  for (const bucket of report.buckets) {
    if (bucket.start >= today.from) break;
    const day = date.format(bucket.start);
    if (!days.has(day)) days.set(day, []);
    days.get(day).push(bucket);
  }
  const elapsed = now - today.from;
  const required = Math.ceil(elapsed / HOUR);
  const values = [];
  // The first day may start at an arbitrary hour.
  for (const day of [...days.values()].slice(1).slice(-7)) {
    const hours = day.slice(0, required);
    if (hours.length < required || !hours.every(bucket => hasElectricity(report.meters, bucket))) continue;
    const totals = new Map();
    for (const [index, bucket] of hours.entries()) {
      const fraction = Math.min(1, elapsed / HOUR - index);
      for (const meter of report.meters) {
        const amount = bucket.meters[meter.id]?.kwh;
        if (ROLES.includes(meter.role) && Number.isFinite(amount)) {
          totals.set(meter.role, (totals.get(meter.role) ?? 0) + amount * fraction);
        }
      }
    }
    // Mirror the daily total: select the greater of overlapping grid and
    // whole-house meters, or fall back to circuits. Add self-consumed solar.
    const [grid, total, circuits, solar, exported] = ROLES.map(role => totals.get(role));
    const bought = grid == null && total == null ? circuits : Math.max(grid ?? 0, total ?? 0);
    const selfConsumed = solar != null && exported != null ? Math.max(0, solar - exported) : 0;
    values.push(bought + selfConsumed);
  }
  return mean(values);
}

export function usualWater(days, today) {
  const cursor = new Date(today + 'T12:00:00Z');
  const values = [];
  for (let i = 0; i < 30; i++) {
    cursor.setUTCDate(cursor.getUTCDate() - 1);
    const value = days?.[cursor.toISOString().slice(0, 10)];
    if (Number.isFinite(value) && value >= 0) values.push(value);
  }
  return mean(values);
}

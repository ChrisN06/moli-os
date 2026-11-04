/** Current monthly TTC fee, weighted by local calendar days and elapsed day fraction. */
export function subscriptionCost(fee, from, to, timezone = 'Europe/Paris') {
  if (!Number.isFinite(fee) || fee < 0 || !Number.isFinite(from) || !Number.isFinite(to) || !(to > from)) return 0;
  const format = new Intl.DateTimeFormat('en-CA', {timeZone: timezone, year:'numeric', month:'numeric', day:'numeric', hour:'numeric', minute:'numeric', second:'numeric', hourCycle:'h23'});
  const parts = ts => Object.fromEntries(format.formatToParts(new Date(ts)).map(p => [p.type, Number(p.value)]));
  function midnight(year, month, day) {
    const target = Date.UTC(year, month - 1, day);
    let guess = target;
    for (let i = 0; i < 4; i++) {
      const p = parts(guess);
      const offset = Date.UTC(p.year, p.month - 1, p.day, p.hour, p.minute, p.second) - guess;
      guess = target - offset;
    }
    return guess;
  }
  function position(ts) {
    const p = parts(ts);
    const days = new Date(Date.UTC(p.year, p.month, 0)).getUTCDate();
    const start = midnight(p.year, p.month, p.day), end = midnight(p.year, p.month, p.day + 1);
    return { month: p.year * 12 + p.month, fraction: (p.day - 1 + (ts - start) / (end - start)) / days };
  }
  const a = position(from), b = position(to);
  return fee * (b.month - a.month + b.fraction - a.fraction);
}

// One request in flight; keep only the latest gesture, never a backlog.
const controls = new Set();
export const waitForLiveLights = () => Promise.all([...controls].map((c) => c.idle()));

export function liveLight(send, interval = 400) {
  const next = new Map();
  let last = null, timer = null, active = false, closed = false, changed = 0;
  let waiters = [];
  const notify = () => { if (!active && !next.size) { for (const done of waiters) done(); waiters = []; } };
  const same = (a, b) => JSON.stringify(a) === JSON.stringify(b);
  async function run() {
    timer = null;
    if (closed || active) return;
    if (Date.now() - changed > 2000) next.clear();
    let order = null;
    while (next.size) {
      const candidate = next.entries().next().value;
      next.delete(candidate[0]);
      if (!same(candidate, last)) { order = candidate; break; }
    }
    if (!order) { last = null; notify(); return; }
    active = true;
    try {
      const result = await send(...order);
      last = order;
      if (result === 'held' || result === 'error') next.clear();
    } catch {
      next.clear();
    } finally {
      active = false;
      notify();
      if (closed) controls.delete(control);
      if (!closed) timer = setTimeout(run, interval);
    }
  }
  const control = {
    push(key, value) {
      if (closed) return;
      changed = Date.now();
      if (key === 'white') next.delete('color');
      if (key === 'color') next.delete('white');
      next.set(key, value);
      if (!active && timer == null) void run();
    },
    idle() { return !active && !next.size ? Promise.resolve() : new Promise((done) => waiters.push(done)); },
    clear() { next.clear(); last = null; clearTimeout(timer); timer = null; notify(); },
    cancel() { closed = true; control.clear(); if (!active) controls.delete(control); },
  };
  controls.add(control);
  return control;
}

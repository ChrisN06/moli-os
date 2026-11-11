import { command } from '../../lib/hub.svelte.js';
const refreshed = new Map();
export function refreshCover(id) {
  const now = Date.now();
  const instance = id.slice(0, id.indexOf(':'));
  if (now - (refreshed.get(instance) ?? 0) < 30000) return;
  refreshed.set(instance, now);
  return command(`${id}/refresh`, true);
}

/** Prefer the driver's typed aliases; raw protocol states remain in the workshop. */
export function visibleReadings(device) {
  const readable = (device?.points ?? []).filter((p) => p.access?.read !== false);
  const typed = readable.filter((p) => !p.key.startsWith('raw.') && p.key !== 'last_received');
  return typed.length && readable.some((p) => p.key.startsWith('raw.')) ? typed : readable;
}

/** Read-only settings and indexes deserve a card even without a control or sensor chip. */
export function hasReadOnlyDetails(device) {
  return visibleReadings(device).some((p) => ['config', 'energy'].includes(p.semantic) && p.access?.read && !p.access?.write);
}

/** Translate at the surface, never in the hub's stable state values. */
export function readingWord(key, value) {
  if (!['heating_level', 'heating_mode', 'force_heating', 'last_button'].includes(key)) return null;
  const word = {
    eco: 'eco', comfort: 'confort', freeze: 'hors_gel', frostprotection: 'hors_gel', freezeMode: 'hors_gel',
    manualMode: 'manuel', auto: 'auto', autoMode: 'auto', atHome: 'present', atHomeMode: 'present',
    awayMode: 'absent', sleepingMode: 'nuit', on: 'actif', off: 'inactif', up: 'montee', down: 'descente', stop: 'arret',
  }[value];
  return typeof word === 'string' ? 'pilotes.overkiz.' + word : null;
}

/** Explicit references win; otherwise use a unique indoor sensor, never outdoor weather. */
export function houseReading(devices, config, key) {
  const point = config?.indoor?.[key];
  const read = (d, k) => {
    const v = d?.state?.[k]?.value;
    return d?.online !== false && typeof v === 'number' && Number.isFinite(v) ? v : null;
  };
  if (point) {
    const slash = point.indexOf('/');
    return slash > 0 ? read(devices[point.slice(0, slash)], point.slice(slash + 1)) : null;
  }
  const outdoor = new Set(Object.values(config?.outdoor ?? {}).filter((v) => typeof v === 'string'));
  const candidates = Object.values(devices).filter((d) => !outdoor.has(d.id) && d.points?.some((p) => p.key === key && p.semantic === key));
  return candidates.length === 1 ? read(candidates[0], key) : null;
}

/** Index under the finger, including gaps and the two plot edges. */
export function barIndex(clientX, left, width, count) {
  if (!(width > 0) || count < 1) return null;
  return Math.max(0, Math.min(count - 1, Math.floor(((clientX - left) / width) * count)));
}

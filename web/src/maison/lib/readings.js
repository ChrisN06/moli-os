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

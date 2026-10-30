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

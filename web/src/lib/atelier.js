/** Search the device's presentation and identity without changing its labels. */
export function filterDevices(devices, query, availability) {
  const norm = (text) => String(text ?? '').normalize('NFD').replace(/\p{M}/gu, '').toLowerCase();
  const words = norm(query).trim().split(/\s+/).filter(Boolean);
  return devices.filter((d) => {
    if (availability === 'online' && d.online !== true) return false;
    if (availability === 'offline' && d.online !== false) return false;
    const text = norm([d.label?.name, d.label?.room, d.native_name, d.native_room, d.manufacturer, d.model, d.id].filter(Boolean).join(' '));
    return words.every((word) => text.includes(word));
  });
}

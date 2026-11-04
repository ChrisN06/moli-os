let initial = {};
try { initial = JSON.parse(localStorage.getItem('moli.camera-display') ?? '{}'); } catch { /* defaults */ }
export const cameraDisplay = $state(initial && typeof initial === 'object' && !Array.isArray(initial) ? initial : {});
export function setCameraRotation(id, rotation) {
  if (![0,90,180,270].includes(rotation)) return;
  cameraDisplay[id] = rotation;
  try { localStorage.setItem('moli.camera-display', JSON.stringify(cameraDisplay)); } catch { /* this visit only */ }
}

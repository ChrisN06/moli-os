// Robot coordinates are millimetres; image pixels are 50 mm, counted from below.
export function at(map, x, y) {
  return [Math.round(map.left + Math.max(0, Math.min(map.width, x)) * 50), Math.round(map.bottom + (map.height - Math.max(0, Math.min(map.height, y))) * 50)];
}
export function rectangle(a,b) { return [Math.min(a[0],b[0]),Math.min(a[1],b[1]),Math.max(a[0],b[0]),Math.max(a[1],b[1])]; }
export function overlay(map,bounds) { return {x:(bounds[0]-map.left)/50,y:map.height-((bounds[3]-map.bottom)/50),width:(bounds[2]-bounds[0])/50,height:(bounds[3]-bounds[1])/50}; }
export function picture(map) {
  const pixels=atob(map.pixels);
  if (pixels.length!==map.width*map.height) throw new Error('map size');
  const canvas=document.createElement('canvas'); canvas.width=map.width; canvas.height=map.height;
  const context=canvas.getContext('2d'), image=context.createImageData(map.width,map.height);
  for(let row=0;row<map.height;row++) for(let x=0;x<map.width;x++) {
    const value=pixels.charCodeAt(row*map.width+x)&7, i=((map.height-1-row)*map.width+x)*4;
    const color=value===0?[238,243,249]:value===1?[39,89,139]:[132,196,235];
    image.data.set([...color,255],i);
  }
  context.putImageData(image,0,0); return canvas.toDataURL();
}

/** Translate a saved rectangle by the dock displacement; never resize or clip it. */
export function rebase(map, zone) {
  if (!zone?.bounds || !zone?.dock || !zone.confirmed_at) return null;
  const delta = map.dock.map((v, i) => v-zone.dock[i]);
  const bounds = zone.bounds.map((v,i) => v+delta[i%2]);
  if (bounds.some(v => !Number.isFinite(v) || v < 0 || v > 51200) || bounds[0]<map.left || bounds[1]<map.bottom || bounds[2]>map.left+map.width*50 || bounds[3]>map.bottom+map.height*50) return null;
  return bounds;
}

/** The dock is the fixed origin shown to people; protocol coordinates stay absolute. */
export function dockOffset(map, bounds, index) { return (bounds[index]-map.dock[index%2])/1000; }
export function dockBound(map, metres, index) { return Math.round(Number(metres)*1000+map.dock[index%2]); }

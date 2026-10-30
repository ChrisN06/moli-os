// Verify that the installable shell ships real PNGs at its declared sizes.
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';

const root = new URL('../web/', import.meta.url);
const manifest = JSON.parse(readFileSync(new URL('dist/manifest.webmanifest', root)));
assert.equal(manifest.start_url, '/');
assert.equal(manifest.scope, '/');
assert.equal(manifest.display, 'standalone');
assert.equal(manifest.icons.length, 2);
for (const icon of [...manifest.icons, { src: '/icons/apple-touch-icon.png', sizes: '180x180' }]) {
  const png = readFileSync(new URL(`dist${icon.src}`, root));
  assert.deepEqual([...png.subarray(0, 8)], [137, 80, 78, 71, 13, 10, 26, 10]);
  assert.equal(`${png.readUInt32BE(16)}x${png.readUInt32BE(20)}`, icon.sizes);
}
const html = readFileSync(new URL('dist/index.html', root), 'utf8');
assert.match(html, /rel="manifest" href="\/manifest.webmanifest"/);
assert.match(html, /rel="apple-touch-icon"/);
console.log('pwa-check: manifest and icons OK');

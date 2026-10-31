import { test } from 'node:test';
import assert from 'node:assert/strict';
import { filterDevices } from './atelier.js';

test('search combines words, ignores accents and preserves labels', () => {
  const devices = [
    { id: 'demo:1', label: { name: 'Éclairage', room: 'Cuisine' }, online: true },
    { id: 'demo:2', native_name: 'Sensor', native_room: 'Office', model: 'Example', online: false },
    { id: 'demo:3' },
  ];
  assert.deepEqual(filterDevices(devices, 'CUISINE eclair', 'all'), [devices[0]]);
  assert.deepEqual(filterDevices(devices, 'example office', 'offline'), [devices[1]]);
  assert.deepEqual(filterDevices(devices, '', 'online'), [devices[0]]);
  assert.deepEqual(filterDevices(devices, '', 'offline'), [devices[1]]);
  assert.deepEqual(filterDevices(devices, '', 'all'), devices);
  assert.equal(devices[0].label.name, 'Éclairage');
});

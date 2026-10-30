import test from 'node:test';
import assert from 'node:assert/strict';
import { visibleReadings, hasReadOnlyDetails, readingWord } from './readings.js';

test('typed readings take precedence without duplicating raw index or receipt time', () => {
  const device = { points: [
    { key: 'energy', access: { read: true, write: false } },
    { key: 'raw.core:ElectricEnergyConsumptionState', access: { read: true } },
    { key: 'last_received', access: { read: true } },
    { key: 'control', access: { read: false } },
  ] };
  assert.deepEqual(visibleReadings(device).map((p) => p.key), ['energy']);
  const raw = { points: device.points.slice(1, 3) };
  assert.equal(visibleReadings(raw).length, 2);
  assert.deepEqual(visibleReadings(null), []);
});

test('read-only settings show a card without treating writable controls as telemetry', () => {
  const point = { key: 'heating_level', semantic: 'config', access: { read: true, write: false } };
  assert.equal(hasReadOnlyDetails({ points: [point] }), true);
  assert.equal(hasReadOnlyDetails({ points: [{ ...point, semantic: 'energy' }] }), true);
  assert.equal(hasReadOnlyDetails({ points: [{ ...point, access: { read: true, write: true } }] }), false);
  assert.equal(hasReadOnlyDetails({ points: [{ ...point, semantic: 'temperature' }] }), false);
  assert.equal(hasReadOnlyDetails(null), false);
});

test('mode translation leaves unknown and unrelated protocol values literal', () => {
  assert.equal(readingWord('heating_level', 'eco'), 'pilotes.overkiz.eco');
  assert.equal(readingWord('heating_mode', 'freezeMode'), 'pilotes.overkiz.hors_gel');
  assert.equal(readingWord('force_heating', 'off'), 'pilotes.overkiz.inactif');
  assert.equal(readingWord('last_button', 'down'), 'pilotes.overkiz.descente');
  assert.equal(readingWord('heating_level', 'future-mode'), null);
  assert.equal(readingWord('unrelated', 'eco'), null);
});

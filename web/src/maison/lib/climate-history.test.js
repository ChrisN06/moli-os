import test from 'node:test';
import assert from 'node:assert/strict';
import { samples, readingAt, curve } from './climate-history.js';
import { housePoint } from './readings.js';
test('stable history uses preceding value and never fills unknown past', () => {
  assert.deepEqual(samples({ from: 10, to: 30, before: [5, 21], raw: [] }), [[10,21],[30,21]]);
  const points = samples({ from: 10, to: 30, raw: [[20, 19], [25, 21]] });
  assert.equal(readingAt(points, 15), null);
  assert.equal(readingAt(points, 24), 19);
  assert.equal(readingAt(points, 30), 21);
  assert.deepEqual(samples(null), []);
});
test('setpoint is a step, never averaged into a fictional setting', () => {
  assert.equal(curve([[10, 10], [20, 20]], 10, 30, 0, 30, true), 'M0.00,160.00 H400.00V80.00');
  assert.deepEqual(samples({ buckets: [{ts:10,avg:17.5}] }, true), []);
  assert.equal(curve([], 10, 10, 0, 30), '');
});
test('chart references follow indoor choices and reject ambiguous thermostats', () => {
  const devices = { a: {id:'a',points:[{key:'reported_setpoint',semantic:'config'}]} };
  assert.equal(housePoint(devices, {}, 'reported_setpoint'), 'a/reported_setpoint');
  devices.b = { ...devices.a, id:'b' };
  assert.equal(housePoint(devices, {}, 'reported_setpoint'), null);
  assert.equal(housePoint(devices, {indoor:{temperature:'chosen/temperature'}}, 'temperature'), 'chosen/temperature');
});

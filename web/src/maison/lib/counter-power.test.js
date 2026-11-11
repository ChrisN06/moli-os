import test from 'node:test';
import assert from 'node:assert/strict';
import { counterPower } from './counter-power.js';

test('counter averages use real elapsed time, units and freshness', () => {
  const s = { raw: [[1000, 100], [1_801_000, 600]] };
  assert.equal(counterPower(s, 'Wh', 1_801_000).watts, 1000);
  assert.equal(counterPower({ raw: [[1000, 0.1], [1_801_000, 0.6]] }, 'kWh', 1_801_000).watts, 1000);
  assert.equal(counterPower(s, 'Wh', 8_000_000).stale, true);
  assert.equal(counterPower({ before: s.raw[0], raw: [s.raw[1]] }, 'Wh', 1_801_000).duration, 1_800_000);
  for (const bad of [[], [[1000, 1]], [[1000, 3], [2000, 2]], [[1000, 1], [1000, 2]], [[1000, 1], [2000, 1000]], [[1000, 1], [90_001_000, 2]], [[1000, null], [2000, 2]]]) {
    assert.equal(counterPower({ raw: bad }, 'Wh', 100_000_000), null);
  }
  assert.equal(counterPower(s, 'VA', 1_801_000), null);
  assert.equal(counterPower(s, 'Wh', 1_801_000, 500), null);
  assert.equal(counterPower({ ...s, buckets: [{}] }, 'Wh', 1_801_000), null);
  assert.equal(counterPower({ ...s, truncated: true }, 'Wh', 1_801_000), null);
  assert.equal(counterPower(s, 'Wh', 0), null);
  assert.equal(counterPower({ raw: [[1000, 1], [2000, 1], [3_601_000, 2]] }, 'Wh', 3_601_000).watts, 1);
});

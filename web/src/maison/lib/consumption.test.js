import assert from 'node:assert/strict';
import { test } from 'node:test';
import { usualElectricity, usualWater } from './consumption.js';

test('water compares against measured past days only, including zero', () => {
  const days = { '2026-11-11': 0.3, '2026-11-09': 0.3, '2026-10-31': 0, '2026-11-12': 0.9 };
  assert.ok(Math.abs(usualWater(days, '2026-11-12') - 0.2) < 1e-10);
  assert.equal(usualWater(days, '2026-11-10'), null);
});

test('electricity compares the same elapsed hours and excludes incomplete days', () => {
  const hour = 3_600_000;
  const from = Date.UTC(2026, 10, 7, 22);
  const today = { from: Date.UTC(2026, 10, 12), to: Date.UTC(2026, 10, 13) };
  const meters = [
    { id: 'meter', role: 'grid' }, { id: 'clamp', role: 'total' }, { id: 'kitchen', role: 'circuit' },
  ];
  const buckets = Array.from({ length: 100 }, (_, i) => {
    const start = from + i * hour;
    return { start, meters: i === 26 ? {} : { meter: { kwh: 4 }, clamp: { kwh: 3 }, kitchen: { kwh: 2 } } };
  });
  const report = { from, timezone: 'UTC', buckets, meters };
  assert.equal(usualElectricity(report, today, today.from + 1.5 * hour), 6);
  assert.equal(usualElectricity({ ...report, buckets: buckets.slice(0, 44) }, today, today.from + 1.5 * hour), null);
});

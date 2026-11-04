import test from 'node:test';
import assert from 'node:assert/strict';
import {subscriptionCost} from './subscription.js';
const cost = (fee, a, b) => subscriptionCost(fee, Date.parse(a), Date.parse(b));
test('subscription uses whole calendar months and partial days', () => {
 assert.ok(Math.abs(cost(15, '2026-11-01T00:00:00+01:00','2026-12-01T00:00:00+01:00')-15)<1e-9);
 assert.ok(Math.abs(cost(15, '2026-11-04T00:00:00+01:00','2026-11-04T12:00:00+01:00')-.25)<1e-9);
 assert.ok(Math.abs(cost(28, '2028-02-01T00:00:00+01:00','2028-03-01T00:00:00+01:00')-28)<1e-9);
 assert.equal(subscriptionCost(null,0,100),0);
});
test('DST days cost one calendar day; repeated hours remain positive', () => {
 assert.ok(Math.abs(cost(31,'2026-03-29T00:00:00+01:00','2026-03-30T00:00:00+02:00')-1)<1e-9);
 assert.ok(Math.abs(cost(31,'2026-10-25T00:00:00+02:00','2026-10-26T00:00:00+01:00')-1)<1e-9);
 assert.ok(cost(31,'2026-10-25T02:00:00+02:00','2026-10-25T02:00:00+01:00')>0);
});

import test from 'node:test';
import assert from 'node:assert/strict';
import {waterMonth,waterTotal,localMonth} from './water.js';
test('water totals keep missing days distinct from measured zero',()=>{
 const days={'2026-11-01':.12,'2026-11-02':.24,'2026-11-03':.36,'2026-11-04':.48,'2026-10-31':4};
 const rows=waterMonth(days,'2026-11');
 assert.equal(rows.length,30);assert.equal(rows[4].value,null);
 assert.equal(waterTotal(rows),1.2);assert.equal(waterTotal(waterMonth({},'2026-11')),null);
 assert.equal(waterTotal(waterMonth({'2026-11-01':0},'2026-11')),0);
 assert.equal(waterMonth({},'2028-02').length,29);assert.deepEqual(waterMonth({},'2026-13'),[]);
 assert.equal(localMonth(new Date(2026,10,1)),'2026-11');
});

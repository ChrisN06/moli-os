import test from 'node:test';
import assert from 'node:assert/strict';
import { liveLight, waitForLiveLights } from './live-light.js';
const settle = async () => { await Promise.resolve(); await Promise.resolve(); };
test('live light sends before release, serializes and coalesces each control', async (t) => {
  t.mock.timers.enable({ apis: ['setTimeout'] });
  const calls = [];
  let finish;
  const control = liveLight((...args) => { calls.push(args); return new Promise((r) => { finish = r; }); });
  control.push('brightness', 10);
  control.push('brightness', 20);
  control.push('brightness', 80);
  control.push('white', 3000);
  assert.deepEqual(calls, [['brightness', 10]]);
  finish('ok'); await settle();
  t.mock.timers.tick(399); assert.equal(calls.length, 1);
  t.mock.timers.tick(1); assert.deepEqual(calls[1], ['brightness', 80]);
  finish('ok'); await settle(); t.mock.timers.tick(400);
  assert.deepEqual(calls[2], ['white', 3000]);
  finish('ok'); await settle();
  control.cancel(); t.mock.timers.tick(4000); assert.equal(calls.length, 3);
});
test('release duplicates are suppressed, but a new gesture can repeat a value', async (t) => {
  t.mock.timers.enable({ apis: ['setTimeout'] });
  const calls=[]; const control=liveLight(async (...args) => { calls.push(args); return 'ok'; });
  control.push('brightness', 40); control.push('brightness', 40);
  await settle(); t.mock.timers.tick(400); assert.equal(calls.length,1);
  control.push('brightness', 40); assert.equal(calls.length,2); control.cancel();
});
test('held and failed orders discard queued updates, unmount cancels pending work', async (t) => {
  t.mock.timers.enable({ apis: ['setTimeout'] });
  for (const result of ['held','error']) {
    const calls=[]; const control=liveLight(async (...args) => { calls.push(args); return result; });
    control.push('brightness', 1); control.push('brightness', 99);
    await settle(); t.mock.timers.tick(400); assert.equal(calls.length,1); control.cancel();
  }
  let resolve; const calls=[];
  const control=liveLight((...args) => { calls.push(args); return new Promise((r) => { resolve=r; }); });
  control.push('white',2700); control.push('white',6500); control.cancel(); resolve('ok');
  await settle(); t.mock.timers.tick(5000); assert.equal(calls.length,1);
});

test('preset barrier includes an in-flight command after unmount',async()=>{
 let resolve;const control=liveLight(()=>new Promise(r=>{resolve=r;}));
 control.push('brightness',50);control.cancel();
 let done=false;const barrier=waitForLiveLights().then(()=>{done=true;});
 await settle();assert.equal(done,false);resolve('ok');await barrier;assert.equal(done,true);
});
test('latest color mode supersedes pending white and clear cancels pending changes',async(t)=>{
 t.mock.timers.enable({apis:['setTimeout']});
 let resolve;const calls=[];const control=liveLight((...args)=>{calls.push(args);return new Promise(r=>{resolve=r;});});
 control.push('brightness',10);control.push('white',2700);control.push('color','#ff0000');
 resolve('ok');await settle();t.mock.timers.tick(400);
 assert.deepEqual(calls,[['brightness',10],['color','#ff0000']]);
 control.push('brightness',99);control.clear();resolve('ok');await settle();t.mock.timers.tick(400);assert.equal(calls.length,2);control.cancel();
});

test('suspended browser never sends a stale pending gesture on return',async(t)=>{
 t.mock.timers.enable({apis:['setTimeout','Date']});
 let resolve;const calls=[];const control=liveLight((...args)=>{calls.push(args);return new Promise(r=>{resolve=r;});});
 control.push('brightness',10);control.push('brightness',90);
 t.mock.timers.tick(3000);resolve('ok');await settle();t.mock.timers.tick(400);
 assert.deepEqual(calls,[['brightness',10]]);
 control.push('brightness',80);assert.deepEqual(calls[1],['brightness',80]);resolve('ok');await settle();control.cancel();
});

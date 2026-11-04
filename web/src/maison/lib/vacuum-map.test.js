import test from 'node:test';
import assert from 'node:assert/strict';
import {at,rectangle,overlay} from './vacuum-map.js';
test('map conversion keeps millimetres and vertical orientation',()=>{
 const map={left:5000,bottom:10000,width:200,height:300};
 assert.deepEqual(at(map,0,0),[5000,25000]);
 assert.deepEqual(at(map,200,300),[15000,10000]);
 const bounds=rectangle(at(map,94,66),at(map,0,0));
 assert.deepEqual(overlay(map,bounds),{x:0,y:0,width:94,height:66});
 assert.equal(bounds[2]-bounds[0],4700);assert.equal(bounds[3]-bounds[1],3300);
 assert.deepEqual(at(map,-100,999),[5000,10000]);
});

test('saved zone follows the dock without resizing or clipping',async()=>{
 const {rebase}=await import('./vacuum-map.js');
 const zone={bounds:[10000,11000,14700,14300],dock:[15000,15000],confirmed_at:1};
 const map={left:5000,bottom:5000,width:400,height:400,dock:[16000,13000]};
 assert.deepEqual(rebase(map,zone),[11000,9000,15700,12300]);
 assert.deepEqual(zone.bounds,[10000,11000,14700,14300]);
 assert.equal(rebase({...map,dock:[100,100]},zone),null);
 assert.equal(rebase(map,{...zone,confirmed_at:0}),null);
});

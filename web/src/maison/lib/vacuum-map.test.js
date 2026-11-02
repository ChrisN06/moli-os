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

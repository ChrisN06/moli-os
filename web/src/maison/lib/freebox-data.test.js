import {test} from 'node:test';
import assert from 'node:assert/strict';
import {appendTraffic, flowProfile, flowRoute, normalizeFreebox, rateText, stationRate, trafficPath, trafficScale} from './freebox-data.js';
const repeater='aa:bb:cc:dd:ee:01',client='aa:bb:cc:dd:ee:02';
function fixture(){return {timestamp:10000,connection:{rate_down:100,rate_up:50},ports:[{id:2,link:'up',speed:'1000',mac_list:[{mac:repeater},{mac:client}]}],stats:{2:{tx_bytes_rate:100}},radios:[[],[]],hosts:[{active:true,host_type:'freebox_wifi',l2ident:{id:repeater},primary_name:'Repeater',access_point:{connectivity_type:'ethernet',ethernet_information:{speed:1000}}},{active:true,l2ident:{id:client},primary_name:'Camera',l3connectivities:[{af:'ipv4',active:true,addr:'192.168.1.10'}],access_point:{type:'repeater',mac:repeater,connectivity_type:'wifi',rx_rate:20,tx_rate:30,wifi_information:{signal:-50,band:'5g',phy_tx_rate:8667,phy_rx_rate:1733}}}]}}
test('repeater clients attach to Ethernet parent with PHY units and no duplicates',()=>{const d=normalizeFreebox(fixture());assert.equal(d.repeaters[0].port,2);assert.equal(d.repeaters[0].stations[0].ip,'192.168.1.10');assert.equal(d.repeaters[0].stations[0].link_down,866.7);assert.equal(d.ports[0].mac_list.length,1);assert.equal(d.timestamp,10)});
test('Pop roaming has one stable MAC and reports its own link speeds',()=>{const f=fixture();f.radios[1]=[{mac:client,hostname:'Camera',signal:-40,last_tx:{bitrate:12000},last_rx:{bitrate:6000},rx_rate:12,tx_rate:24}];const d=normalizeFreebox(f);assert.equal(d.wifi.length,1);assert.equal(d.wifi[0].link_down,1200);assert.equal(d.wifi[0].link_up,600);assert.equal(d.repeaters[0].stations.length,0);assert.equal(d.wifi[0].mac,client)});

test('an empty radio returned as null keeps Ethernet and the other radio available',()=>{
 const raw=fixture();
 raw.radios=[null,[{mac:client,hostname:'Camera',signal:-40,last_tx:{bitrate:12000}}]];
 const data=normalizeFreebox(raw);
 assert.equal(data.connection.rate_down,100);
 assert.equal(data.ports.length,1);
 assert.equal(data.wifi.length,1);
 assert.equal(data.wifi[0].band,'5g');
 assert.equal(data.wifi[0].link_down,1200);
 raw.radios=[null,null];
 assert.equal(normalizeFreebox(raw).wifi.length,0);
});

test('rates distinguish zero, missing measurements and decimal units', () => {
  assert.equal(rateText(null, 'en'), '—');
  assert.equal(rateText(0, 'en'), '0 kbit/s');
  assert.equal(rateText(19_787.5, 'en'), '158.3 kbit/s');
  assert.equal(rateText(125_000_000, 'en'), '1 Gbit/s');
  assert.equal(stationRate([{ tx_rate: 1 }, { tx_rate: null }], 'tx_rate'), null);
  assert.equal(stationRate([{ tx_rate: 1 }, { tx_rate: 2 }], 'tx_rate'), 3);
});

test('traffic keeps only distinct samples and leaves outages blank', () => {
  const datum = timestamp => ({ timestamp, connection: { rate_down: 12 } });
  let samples = [];
  for (const time of [100, 105, 155, 160, 165]) samples = appendTraffic(samples, datum(time));
  assert.equal(appendTraffic(samples, datum(165)), samples);
  assert.deepEqual(samples.map(s => s.time), [105, 155, 160, 165]);
  assert.equal(samples[0].up, null);
  assert.equal(trafficPath(samples, 'down', 165, 24, 60, 96), 'M0.0 48.0h.01 M50.0 48.0h.01 L55.0 48.0 L60.0 48.0');
});


test('only measured positive traffic animates, with bounded speed and density', () => {
  for (const rate of [null, undefined, -1, 0]) assert.equal(flowProfile(rate).count, 0);
  const quiet = flowProfile(500);
  const busy = flowProfile(1_000_000);
  assert.ok(busy.count > quiet.count && busy.speed > quiet.speed);
  assert.deepEqual(flowProfile(1_000_000_000), { count: 8, speed: 1.6 });
});

test('traffic axis grows to fit and waits before shrinking', () => {
  const large = trafficScale({ ceiling: 200, lowerSince: null }, 173_000, 10);
  assert.equal(large.ceiling, 200_000);
  const waiting = trafficScale(large, 30_000, 40);
  assert.equal(trafficScale(waiting, 30_000, 54), waiting);
  assert.equal(trafficScale(waiting, 30_000, 55).ceiling, 50_000);
  assert.deepEqual(trafficScale(waiting, 120_000, 54), large);
});

test('branch trajectory spans the junction, with reversed upload and straight zero-length turns', () => {
  assert.deepEqual(flowRoute(10, 30, 40, [90, 80], 0), {
    path: 'M10 30 L28 30 Q40 30 40 42 L40 68 Q40 80 52 80 L90 80',
    length: 130,
  });
  assert.match(flowRoute(10, 30, 40, [90, 80], 4, true).path, /^M90 84 .* L10 34$/);
  assert.doesNotMatch(flowRoute(40, 30, 40, [90, 30], 0).path, /NaN|Infinity/);
});

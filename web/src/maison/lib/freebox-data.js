// Freebox rates are bytes/s; PHY rates are reported in 100 kbit/s.
function hostInfo(host = {}) {
  const point=host.access_point||{}, info=point.wifi_information||{};
  return {hostname:host.primary_name||host.default_name,mac:host.l2ident?.id,
    ip:host.l3connectivities?.find(a=>a.af==='ipv4'&&a.active)?.addr,
    signal:info.signal,band:info.band,standard:info.standard,
    link_down:(info.phy_tx_rate||0)/10,link_up:(info.phy_rx_rate||0)/10,
    rx_rate:point.rx_rate||0,tx_rate:point.tx_rate||0};
}
export function normalizeFreebox(raw) {
  const hosts=raw.hosts||[], index=new Map(hosts.map(h=>[h.l2ident?.id?.toLowerCase(),h]));
  const ports=(raw.ports||[]).map(p=>({...p,stats:raw.stats?.[p.id]||{},mac_list:(p.mac_list||[]).map(d=>({...d,...hostInfo(index.get(d.mac?.toLowerCase())),mac:d.mac,hostname:index.get(d.mac?.toLowerCase())?.primary_name||d.hostname,port_speed:p.link==='up'?Number(p.speed):0}))}));
  const wifi=new Map();
  (raw.radios||[]).forEach((stations,radio)=>stations.forEach(s=>wifi.set(s.mac,{...hostInfo(s.host),hostname:s.hostname||s.host?.primary_name,mac:s.mac,signal:s.signal,band:radio===0?'2d4g':'5g',link_down:(s.last_tx?.bitrate||0)/10,link_up:(s.last_rx?.bitrate||0)/10,rx_rate:s.rx_rate||0,tx_rate:s.tx_rate||0})));
  const repeaters=new Map();
  for(const h of hosts)if(h.active&&h.host_type==='freebox_wifi'){
    const mac=h.l2ident.id.toLowerCase(),p=ports.find(p=>p.link==='up'&&p.mac_list.some(d=>d.mac.toLowerCase()===mac));
    repeaters.set(mac,{id:'repeater-'+mac.replaceAll(':',''),hostname:h.primary_name,mac,port:p?.id,connection:h.access_point?.connectivity_type,speed:h.access_point?.ethernet_information?.speed,stations:[]});
  }
  for(const h of hosts){const a=h.access_point;if(!h.active||a?.type!=='repeater'||a.connectivity_type!=='wifi')continue;const mac=a.mac.toLowerCase();if(!repeaters.has(mac))repeaters.set(mac,{id:'repeater-'+mac.replaceAll(':',''),mac,stations:[]});const d=hostInfo(h);if(!wifi.has(d.mac))repeaters.get(mac).stations.push(d)}
  const wireless=new Set([...wifi.keys(),...[...repeaters.values()].flatMap(r=>r.stations.map(d=>d.mac))].map(mac=>mac?.toLowerCase()));
  for(const p of ports)p.mac_list=p.mac_list.filter(d=>!wireless.has(d.mac.toLowerCase()));
  return {connection:raw.connection,ports,wifi:[...wifi.values()],repeaters:[...repeaters.values()],timestamp:raw.timestamp/1000};
}

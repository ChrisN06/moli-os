export function createNetwork(root,t,locale){
let animation;
let flowColors;
function refreshColors(){const style=getComputedStyle(root.host);flowColors=[['down',style.getPropertyValue('--network-down').trim()||'#32ffa0'],['up',style.getPropertyValue('--network-up').trim()||'#ff6677']]}
refreshColors();
const themeObserver=new MutationObserver(refreshColors);
const themeHost=root.host.closest('[data-theme]');
if(themeHost)themeObserver.observe(themeHost,{attributes:true,attributeFilter:['data-theme']});
function wifiDetail(device) {
  const band = device.band === '5g' ? '5 GHz' : device.band === '2d4g' ? '2,4 GHz' : '—';
  return `${device.signal ?? '—'} dBm · ${band}`;
}
function smoothValue(current, target, seconds) {
  return current + (target - current) * (1 - Math.exp(-Math.max(0, seconds) / 0.45));
}
function topologyGroups(data) {
  const ports = [...data.ports].sort((a, b) => a.id - b.id);
  const groups = ports.map(port => ({
    id: port.id,
    title: t('freebox.ethernet',{id:port.id}),
    subtitle: port.link === 'up' ? port.speed + ' Mbit/s · ' + (port.duplex === 'full' ? t('freebox.full') : t('freebox.half')) : t('freebox.disconnected'),
    active: port.link === 'up',
    devices: port.link === 'up' ? (port.mac_list || []).map(device => ({...device, name: device.hostname || device.mac, detail: device.mac})) : []
  }));
  groups.push({id: 'wifi', title: t('freebox.wifi_pop'), subtitle: '2,4 GHz + 5 GHz', active: data.wifi.length > 0,
    devices: data.wifi.map(device => ({...device, name: device.hostname || t('freebox.device'), detail: wifiDetail(device)}))});
  for (const repeater of data.repeaters || []) {
    const group = {id: repeater.id, title: t('freebox.repeater'),
      subtitle: repeater.connection === 'ethernet' ? 'Ethernet · ' + repeater.speed + ' Mbit/s' : t('freebox.unknown_link'),
      parent: repeater.port, active: true,
      devices: repeater.stations.map(device => ({...device, name: device.hostname || device.mac, detail: wifiDetail(device)}))};
    const index = groups.findIndex(port => port.id === repeater.port);
    groups.splice(index >= 0 ? index + 1 : groups.length, 0, group);
  }
  let top = 35;
  for (const group of groups) {
    group.top = top;
    group.height = Math.max(100, Math.ceil(group.devices.length / 3) * 46 + 44);
    group.y = top + group.height / 2;
    top += group.height + 26;
  }
  return {groups, height: Math.max(600, top)};
}

function flowStyle(bytesPerSecond) {
  const rate = Math.max(0, Number(bytesPerSecond) || 0);
  const mbps = rate * 8 / 1e6;
  return {
    visible: rate > 0,
    particles: rate > 0 ? Math.round(45 + Math.min(455, 110 * Math.log10(1 + mbps))) : 0,
    waveAmplitude: 3,
    width: rate > 0 ? 1.5 + Math.min(20, 5 * Math.log10(1 + mbps)) : 0,
    duration: 5 / (1 + Math.log10(1 + rate) / 2)
  };
}


const particleCanvas = root.getElementById('particles');
const particleContext = particleCanvas.getContext('2d');
const reducedMotion = matchMedia('(prefers-reduced-motion: reduce)');
let particleLinks = [], particleClock = 0, previousFrame = 0;
function prepareParticles() {
  const previous=new Map(particleLinks.map(link=>[link.group.id,link]));
  particleLinks=[...root.querySelectorAll('#edges > g')].filter(group=>group.querySelector('.flow')).map(group=>{
    const old=previous.get(group.id),path=group.querySelector('.wire'),d=path.getAttribute('d');
    return {group,path,d,points:old?.points||[],resampleUntil:old?.d===d?old.resampleUntil:performance.now()+900,
      state:old?.state||{down:{rate:0,phase:0},up:{rate:0,phase:0}}};
  });
}
function particleFrame(time) {
  const bounds=particleCanvas.getBoundingClientRect(), dpr=Math.min(devicePixelRatio,2);
  if(particleCanvas.width!==Math.round(bounds.width*dpr)||particleCanvas.height!==Math.round(bounds.height*dpr)){
    particleCanvas.width=Math.round(bounds.width*dpr);particleCanvas.height=Math.round(bounds.height*dpr);
  }
  const ctx=particleContext;
  ctx.setTransform(1,0,0,1,0,0);ctx.clearRect(0,0,particleCanvas.width,particleCanvas.height);
  const elapsed=Math.min(50,time-previousFrame||0)/1000;
  const moving=!root.host.classList.contains('paused')&&!reducedMotion.matches;
  if(moving)particleClock+=elapsed;
  previousFrame=time;
  const view=root.querySelector('svg').viewBox.baseVal;
  const scale=Math.min(bounds.width/view.width,bounds.height/view.height);
  ctx.setTransform(dpr*scale,0,0,dpr*scale,dpr*(bounds.width-view.width*scale)/2,dpr*(bounds.height-view.height*scale)/2);
  for(const link of particleLinks){
    if(!link.points.length||performance.now()<link.resampleUntil){const length=link.path.getTotalLength();link.points=Array.from({length:101},(_,i)=>link.path.getPointAtLength(length*i/100))}
    for(const[direction,color]of flowColors){
      const rate=Number(link.group.querySelector('.'+direction).dataset.rate)||0;
      const state=link.state[direction];state.rate=smoothValue(state.rate,rate,elapsed);
      const style=flowStyle(state.rate);if(state.rate<.1)continue;
      const targetCount=45+Math.min(455,110*Math.log10(1+state.rate*8/1e6));
      const count=targetCount*Math.min(1,state.rate/100);
      if(moving)state.phase=(state.phase+elapsed/(style.duration*2))%1;
      const sign=direction==='down'?1:-1, width=style.width*1.8;
      ctx.fillStyle=color;ctx.shadowColor=color;ctx.shadowBlur=5;
      for(let i=0;i<Math.ceil(count);i++){
        const progress=((((i*.61803398875)%1)+sign*state.phase)%1+1)%1;
        const index=Math.min(99,Math.floor(progress*100)), mix=progress*100-index;
        const a=link.points[index],b=link.points[index+1];
        const dx=b.x-a.x,dy=b.y-a.y,length=Math.hypot(dx,dy)||1;
        const wave=Math.sin(progress*12+particleClock*1.7+sign)*style.waveAmplitude;
        const spread=Math.sin(i*127.1)*Math.cos(i*31.7)*width*.75;
        const offset=wave+spread+sign*2;
        const x=a.x+(b.x-a.x)*mix-dy/length*offset;
        const y=a.y+(b.y-a.y)*mix+dx/length*offset;
        ctx.globalAlpha=(.28+.65*((Math.sin(i*17.3)+1)/2))*Math.min(1,count-i);
        ctx.beginPath();ctx.arc(x,y,.55+((i*7)%5)*.13,0,Math.PI*2);ctx.fill();
      }
      ctx.globalAlpha=1;ctx.shadowBlur=0;
    }
  }
  animation=requestAnimationFrame(particleFrame);
}
animation=requestAnimationFrame(particleFrame);

const NS='http://www.w3.org/2000/svg';
let latest,selected=2,selectedDevice=null;
const deviceRows=new Map(),groupNodes=new Map(),linkNodes=new Map();
function el(tag,attrs,parent){const n=document.createElementNS(NS,tag);for(const[k,v]of Object.entries(attrs))n.setAttribute(k,v);parent.append(n);return n}
function text(parent,x,y,value,cls=''){const n=el('text',{x,y,class:cls},parent);n.textContent=value;return n}
function clickable(node,action){node.setAttribute('tabindex','0');node.setAttribute('role','button');node.onclick=action;node.onkeydown=e=>{if(e.key==='Enter'||e.key===' '){e.preventDefault();action()}}}
function node(id,x,y,title,sub,symbol,action){let n=groupNodes.get(id);if(!n){n=el('g',{class:'node',id},root.querySelector('#nodes'));el('rect',{x:-58,y:-36,width:116,height:72,rx:13},n);text(n,0,-7,symbol,'symbol');text(n,0,14,title);text(n,0,55,sub,'sub');if(action)clickable(n,action);groupNodes.set(id,n)}n.style.transform=`translate(${x}px,${y}px)`;n.querySelector('.sub').textContent=sub;return n}
function edge(id,x1,y1,x2,y2,animated=true){let g=linkNodes.get(id);if(!g){g=el('g',{id},root.querySelector('#edges'));el('path',{class:animated?'wire':'wire unknown'},g);if(animated){el('path',{class:'flow down'},g);el('path',{class:'flow up'},g);text(g,0,0,'','link-rate green');text(g,0,13,'','link-rate red')}linkNodes.set(id,g)}const d=`M ${x1} ${y1} C ${(x1+x2)/2} ${y1}, ${(x1+x2)/2} ${y2}, ${x2} ${y2}`;g.querySelectorAll('path').forEach(p=>p.setAttribute('d',d));g.querySelectorAll('.link-rate').forEach((n,i)=>{n.setAttribute('x',x1>x2?600:Math.max(x1+8,x2-95));n.setAttribute('y',y2-25+i*13)});return g}
function allDevices(){if(!latest)return [];return [...latest.ports.flatMap(p=>(p.mac_list||[]).map(d=>({...d,point:t('freebox.ethernet',{id:p.id}),wireless:false}))),...latest.wifi.map(d=>({...d,point:t('freebox.wifi_pop'),wireless:true})),...latest.repeaters.flatMap(r=>r.stations.map(d=>({...d,point:r.hostname,wireless:true})))]}
function renderTopology(data){const layout=topologyGroups(data),seenRows=new Set(),seenNodes=new Set(),seenLinks=new Set();root.querySelector('svg').setAttribute('viewBox',`0 0 1200 ${layout.height}`);const center=layout.height/2;
  edge('wan',130,center,222,center);seenLinks.add('wan');node('internet',72,center,t('freebox.internet'),'FTTH','◎');node('box',280,center,'Freebox',t('freebox.gateway'),'▰');seenNodes.add('internet');seenNodes.add('box');
  for(const group of layout.groups){const id=group.id==='wifi'?'wifi':'p'+group.id,parent=layout.groups.find(g=>g.id===group.parent);edge(id,parent?588:338,parent?parent.y:center,472,group.y);seenLinks.add(id);const nodeId=group.id==='wifi'?'wifinode':'port'+group.id;node(nodeId,530,group.y,group.title,group.subtitle,typeof group.id==='string'?'◉':'▤',()=>choose(group.id));seenNodes.add(nodeId);edge('devices-'+group.id,588,group.y,680,group.y,false);seenLinks.add('devices-'+group.id);
    let card=root.getElementById('card-'+group.id);if(!card)card=el('rect',{id:'card-'+group.id,class:'group-card',x:680,width:490,rx:13},root.querySelector('#cards'));card.setAttribute('y',group.top);card.setAttribute('height',group.height);card.dataset.seen='yes';
    group.devices.forEach((device,index)=>{const key=device.mac||group.id+'-'+device.name+'-'+index;seenRows.add(key);let row=deviceRows.get(key);if(!row){row=el('g',{class:'device-entry'},root.querySelector('#nodes'));text(row,0,0,'','device-name');text(row,0,12,'','device-meta');text(row,0,24,'','device-speed');clickable(row,()=>{selectedDevice=key;renderDetails();root.querySelector('.detail').classList.add('open')});deviceRows.set(key,row)}row.style.opacity='1';row.style.transform=`translate(${694+(index%3)*162}px,${group.top+25+Math.floor(index/3)*46}px)`;row.dataset.key=key;updateDeviceRow(row,device,group.id);});
  }
  for(const[key,row]of deviceRows)if(!seenRows.has(key)){row.remove();deviceRows.delete(key)}
  for(const[id,n]of groupNodes)if(!seenNodes.has(id)){n.remove();groupNodes.delete(id)}
  for(const[id,n]of linkNodes)if(!seenLinks.has(id)){n.remove();linkNodes.delete(id)}
  root.querySelectorAll('#cards rect').forEach(card=>{if(card.dataset.seen!=='yes')card.remove();else card.dataset.seen='no'});prepareParticles();
}
function linkText(device){return device.port_speed?t('freebox.port')+' '+device.port_speed+' Mbit/s':`${t('freebox.link')} ↓ ${device.link_down||'—'} · ↑ ${device.link_up||'—'} Mbit/s`}
function updateDeviceRow(row,device,groupId){row.querySelector('.device-name').textContent=(device.name||device.hostname||device.mac||t('freebox.device')).slice(0,23);row.querySelector('.device-meta').textContent=typeof groupId==='string'?wifiDetail(device):(device.ip||device.mac||'');row.querySelector('.device-speed').textContent=linkText(device).replace(t('freebox.link')+' ','');}
function choose(id){const panel=root.querySelector('.detail');if(selected===id&&selectedDevice===null&&panel.classList.contains('open')){panel.classList.remove('open');return}selected=id;selectedDevice=null;renderDetails();panel.classList.add('open')}
function renderDetails(){if(!latest)return;const container=root.getElementById('devices');container.replaceChildren();let items,title,sub,speed;if(selectedDevice){const d=allDevices().find(d=>d.mac===selectedDevice);items=d?[d]:[];title=d?.hostname||t('freebox.gone');sub=d?.point||t('freebox.missing');speed=d?linkText(d):''}else{const r=latest.repeaters.find(r=>r.id===selected),p=latest.ports.find(p=>p.id===selected);items=r?r.stations:selected==='wifi'?latest.wifi:p?.mac_list||[];title=r?t('freebox.wifi_repeater'):selected==='wifi'?t('freebox.wifi_pop'):t('freebox.ethernet',{id:selected});sub=t('freebox.click_device');speed=r?'Ethernet '+(r.speed||'—')+' Mbit/s':p?p.link==='up'?p.speed+' Mbit/s':t('freebox.disconnected'):items.length+' · '+t('freebox.stations')}root.getElementById('detailTitle').textContent=title;root.getElementById('detailSub').textContent=sub;root.getElementById('speed').textContent=speed;for(const d of items){const row=document.createElement('div');row.className='device';const content=document.createElement('div'),b=document.createElement('b'),small=document.createElement('small');b.textContent=d.hostname||d.mac;small.textContent=[d.ip||t('freebox.ip_missing'),d.mac,d.signal!=null?wifiDetail(d):'',linkText(d),d.signal!=null?`${t('freebox.current')} ↓ ${rateText(d.tx_rate)} · ↑ ${rateText(d.rx_rate)}`:t('freebox.individual_missing')].filter(Boolean).join(' · ');content.append(b,small);row.append(content);if(!selectedDevice)row.onclick=()=>{selectedDevice=d.mac;renderDetails()};container.append(row)}if(!items.length)container.textContent=t('freebox.no_device')}
function rateText(value){const mbps=Math.max(0,Number(value)||0)*8/1e6;return mbps<1?(mbps*1000).toFixed(1)+' kbit/s':mbps.toFixed(2)+' Mbit/s'}
function animate(id,down,up){const g=linkNodes.get(id);if(!g)return;for(const[cls,value]of [['down',down],['up',up]]){g.querySelector('.'+cls).dataset.rate=String(Math.max(0,Number(value)||0));const label=g.querySelector('.link-rate.'+(cls==='down'?'green':'red'));label.textContent=(cls==='down'?'↓ ':'↑ ')+rateText(value)}}
root.querySelector('.close-detail').onclick=()=>root.querySelector('.detail').classList.remove('open');root.getElementById('pause').onclick=function(){const paused=root.host.classList.toggle('paused');this.textContent=paused?t('freebox.resume'):t('freebox.pause');this.setAttribute('aria-pressed',paused)};


function update(data){latest=data;if(!data){root.getElementById('status')?.remove();root.getElementById('state').textContent=t('freebox.failed');for(const id of ['download','upload'])root.getElementById(id).textContent='—';root.querySelectorAll('.link-rate,.device-speed').forEach(n=>n.textContent='—');root.querySelector('.detail').classList.remove('open');root.querySelectorAll('.flow').forEach(n=>n.dataset.rate='0');return}renderTopology(data);const c=data.connection;root.getElementById('state').textContent=t(c.state==='up'?'freebox.connected':'freebox.disconnected');root.getElementById('download').textContent=rateText(c.rate_down);root.getElementById('upload').textContent=rateText(c.rate_up);animate('wan',c.rate_down,c.rate_up);for(const p of data.ports)animate('p'+p.id,p.stats.tx_bytes_rate,p.stats.rx_bytes_rate);for(const r of data.repeaters)animate('p'+r.id,r.stations.reduce((n,d)=>n+(d.tx_rate||0),0),r.stations.reduce((n,d)=>n+(d.rx_rate||0),0));animate('wifi',data.wifi.reduce((n,d)=>n+(d.tx_rate||0),0),data.wifi.reduce((n,d)=>n+(d.rx_rate||0),0));root.getElementById('updated').textContent=t('freebox.updated')+' · '+new Date(data.timestamp*1000).toLocaleTimeString(locale());if(root.querySelector('.detail.open'))renderDetails()}
return {update,destroy(){themeObserver.disconnect();cancelAnimationFrame(animation);root.replaceChildren()}};
}

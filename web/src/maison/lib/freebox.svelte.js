import { normalizeFreebox } from './freebox-data.js';
export const freebox = $state({data:null,configured:null,failed:false});
let users=0,timer,controller;
async function refresh(){if(!users||document.hidden||controller)return;controller=new AbortController();try{
  const response=await fetch('/api/freebox/traffic',{signal:controller.signal,cache:'no-store'});
  if(response.status===404){freebox.configured=false;freebox.data=null;return}
  if(!response.ok)throw Error();freebox.data=normalizeFreebox(await response.json());freebox.configured=true;freebox.failed=false;
}catch(error){if(error.name!=='AbortError'){freebox.data=null;freebox.failed=true}}
finally{controller=null;if(users&&!document.hidden)timer=setTimeout(refresh,5000)}}
function visibility(){clearTimeout(timer);if(document.hidden)controller?.abort();else refresh()}
export function useFreebox(){if(++users===1){document.addEventListener('visibilitychange',visibility);refresh()}return()=>{if(--users===0){clearTimeout(timer);controller?.abort();document.removeEventListener('visibilitychange',visibility)}}}

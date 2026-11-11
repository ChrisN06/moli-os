<script>
  import { onMount } from 'svelte';
  import { freebox, useFreebox } from '../lib/freebox.svelte.js';
  import { t, locale } from '../../lib/i18n.svelte.js';
  import template from './network/template.html?raw';
  import { createNetwork } from './network/renderer.js';
  let element;
  let renderer=$state.raw(null);
  const escape = value => value.replaceAll('&','&amp;').replaceAll('<','&lt;').replaceAll('>','&gt;').replaceAll('"','&quot;');
  onMount(()=>{
    const root=element.attachShadow({mode:'open'});
    root.innerHTML=template.replace(/\{\{(freebox\.[a-z_]+)\}\}/g,(_,key)=>escape(t(key)));
    renderer=createNetwork(root,t,locale);renderer.update(freebox.data);
    const stop=useFreebox();return()=>{stop();renderer.destroy()};
  });
  $effect(()=>{const data=freebox.data;renderer?.update(data)});
</script>
<div class="network-view"><div bind:this={element} class="topology"></div>
{#if freebox.configured === false}
  <p class="setup">{t('freebox.not_configured')} <a href="#/parametres">{t('freebox.settings')}</a></p>
{/if}
</div>
<style>.network-view{height:100%;min-height:0;position:relative;border:1px solid var(--line);border-radius:var(--r-lg);overflow:hidden;background:var(--surface)}.topology{height:100%;min-height:0}.setup{position:absolute;top:180px;left:20px;right:20px;color:var(--ink-2);padding:16px;background:var(--surface);border-radius:12px}@media(max-width:700px){.network-view{height:calc(100dvh - 172px);min-height:620px}.topology{height:100%;min-height:0}}</style>

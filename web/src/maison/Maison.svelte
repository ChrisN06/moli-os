<script>
  import '@fontsource-variable/figtree';
  import './maison.css';
  import './orbite.css';
  import { attachRemoteNavigation } from './lib/remote-navigation.js';
  import { onMount, tick } from 'svelte';
  import { home, hub, start, value, savePref, watches, hidden } from './lib/home.svelte.js';
  import { t } from '../lib/i18n.svelte.js';
  import Icon from './ui/Icon.svelte';
  import PinSheet from './ui/PinSheet.svelte';
  import Correct from './moli/Correct.svelte';
  import Setup from './ui/Setup.svelte';
  import Login from './ui/Login.svelte';
  import Accueil from './pages/Accueil.svelte';
  import Salon from './pages/Salon.svelte';
  import Pieces from './pages/Pieces.svelte';
  import Plan from './pages/Plan.svelte';
  import Cameras from './pages/Cameras.svelte';
  import Parametres from './pages/Parametres.svelte';
  import Energie from './pages/Energie.svelte';
  import Aspirateur from './pages/Aspirateur.svelte';
  import Impression from './pages/Impression.svelte';
  import Dehors from './pages/Dehors.svelte';
  import Moli from './pages/Moli.svelte';
  import MoliSheet from './moli/MoliSheet.svelte';
  import Automatismes from './pages/Automatismes.svelte';
  import Systeme from './pages/Systeme.svelte';
  import Parler from './pages/Parler.svelte';
  import Orb from './moli/Orb.svelte';
  import { moli } from './lib/moli.svelte.js';

  let { route } = $props();
  let shell, moreDialog;
  let display = $state('auto');

  // Shown only if the link stays down a few seconds (a blink is not news).
  let offline = $state(false);
  $effect(() => {
    if (hub.connected) {
      offline = false;
      return;
    }
    const timer = setTimeout(() => (offline = true), 3000);
    return () => clearTimeout(timer);
  });

  // `label` is a catalogue key, said where it is shown; Moli is a name.
  const NAV = [
    { path: '', label: 'commun.nav.accueil', icon: 'home' },
    { path: 'moli', name: 'Moli', orb: true },
    { path: 'salon', label: 'commun.nav.salon', icon: 'sofa' },
    { path: 'pieces', label: 'commun.nav.pieces', icon: 'rooms' },
    { path: 'plan', label: 'commun.nav.plan', icon: 'home-roof' },
    { path: 'cameras', label: 'commun.nav.cameras', icon: 'cctv' },
    { path: 'parler', label: 'commun.nav.parler', icon: 'talk' },
    { path: 'energie', label: 'commun.nav.energie', icon: 'bolt' },
    { path: 'aspirateur', label: 'xiaomi.vacuum.title', icon: 'robot', shown: () => Object.values(hub.devices).some(d => d.model === 'rockrobo.vacuum.v1') },
    { path: 'impression', label: 'commun.nav.impression', icon: 'printer3d' },
    { path: 'dehors', label: 'commun.nav.dehors', icon: 'flower-outline' },
    { path: 'automatismes', label: 'commun.nav.automatismes', icon: 'robot' },
    { path: 'reseau', label: 'freebox.title', icon: 'lan' },
    { path: 'infra', label: 'commun.nav.infra', icon: 'server' },
    // Only when something is watched: a house with nothing to watch sees no empty page.
    { path: 'parc', label: 'commun.nav.parc', icon: 'package', shown: () => Object.values(hub.devices).some((d) => !hidden(d.id) && watches(d)) },
  ];
  const nav = $derived(NAV.filter((item) => (!item.shown || item.shown()) && (!Array.isArray(home.config?.navigation) || home.config.navigation.includes(item.path))));

  const primary = $derived(['', 'salon', 'energie', 'aspirateur', 'moli'].map(path => nav.find(item => item.path === path)).filter(Boolean));
  let moliOpen = $state(false);
  let prefersDark = $state(false);

  onMount(() => {
    start();
    try { const saved = localStorage.getItem('maison-display'); if (['auto', 'desktop', 'tablet', 'phone', 'tv'].includes(saved)) display = saved; } catch { /* Private browsing keeps the automatic layout. */ }
    const stopRemote = attachRemoteNavigation(shell);
    const media = matchMedia('(prefers-color-scheme: dark)');
    prefersDark = media.matches;
    const update = (e) => (prefersDark = e.matches);
    media.addEventListener('change', update);
    return () => { media.removeEventListener('change', update); stopRemote(); };
  });

  // Auto: night after sunset (the house's own daylight), else the system's.
  const daylight = $derived(value(home.config?.outdoor?.weather, 'daylight'));
  const theme = $derived(
    home.theme === 'light'
      ? 'day'
      : home.theme === 'dark'
        ? 'night'
        : daylight === false
          ? 'night'
          : daylight === true
            ? 'day'
            : prefersDark
              ? 'night'
              : 'day',
  );

  $effect(() => {
    const root = document.documentElement.style;
    root.background = theme === 'night' ? '#0d1016' : '#edf0f5';
    document.body.style.background = 'transparent';
    return () => {
      root.background = '';
      document.body.style.background = '';
    };
  });

  // The theme button says the preference; a forced one says its own side.
  const themeName = $derived(home.theme === 'auto' ? t('commun.theme.auto') : theme === 'night' ? t('commun.theme.nuit') : t('commun.theme.jour'));
  const themeTitle = $derived(
    home.theme === 'auto' ? t('commun.theme.titre_auto') : home.theme === 'dark' ? t('commun.theme.titre_nuit') : t('commun.theme.titre_jour'),
  );

  function cycleTheme() {
    home.theme = home.theme === 'auto' ? (theme === 'night' ? 'light' : 'dark') : 'auto';
    savePref('maison-theme', home.theme);
  }

  const page = $derived(route.split('/')[0] ?? '');
  $effect(() => {
    const destination = route;
    let cancelled = false;
    tick().then(() => { if (!cancelled && route === destination) window.scrollTo({ top: 0, behavior: 'instant' }); });
    return () => { cancelled = true; };
  });
</script>

<div class="maison orbite" data-theme={theme} data-display={display} data-page={page || 'accueil'} bind:this={shell}>
  <header class="orbite-bar">
    <a class="orbite-brand" href="#/" aria-label={t('commun.nav.logo')}>{t('orbite.brand')}<span>{t('orbite.name')}</span></a>
    <div class="orbite-tools">
      <label>{t('orbite.display')}<select aria-label={t('orbite.display')} value={display} onchange={(event) => { display = event.currentTarget.value; savePref('maison-display', display); }}>
        <option value="auto">{t('orbite.auto')}</option><option value="desktop">{t('orbite.desktop')}</option><option value="tablet">{t('orbite.tablet')}</option><option value="phone">{t('orbite.phone')}</option><option value="tv">{t('orbite.tv')}</option>
      </select></label>
      <button class="theme" onclick={cycleTheme} title={themeTitle}><Icon name={theme === 'night' ? 'moon' : 'sun'} size={20} /><span>{themeName}</span></button>
    </div>
  </header>
  <nav class="orbite-dock" aria-label={t('commun.nav.aria')}>
    {#each primary as item (item.path)}
      <a href="#/{item.path}" aria-current={page === item.path ? 'page' : undefined} data-section={item.path || 'accueil'}>{#if item.orb}<Orb size={22} state={moli.busy ? 'thinking' : 'idle'}/>{:else}<Icon name={item.icon} size={22}/>{/if}<span>{item.name ?? t(item.label)}</span></a>
    {/each}
    <button onclick={() => moreDialog.showModal()} aria-haspopup="dialog" class:active={!primary.some(item => item.path === page)}><Icon name="rooms" size={22}/><span>{t('orbite.more')}</span></button>
  </nav>
  <main class="content" class:network={page === 'reseau'}>
    {#if page === 'salon'}
      <Salon />
    {:else if page === 'pieces'}
      <Pieces />
    {:else if page === 'plan'}
      <Plan />

    {:else if page === 'cameras'}
      <Cameras />
    {:else if page === 'parler'}
      <Parler />
    {:else if page === 'energie' && route.startsWith('energie/solaire')}
      <!-- The solar demo (a simulation): loaded only when visited. -->
      {#await import('./pages/Solaire.svelte') then { default: Solaire }}<Solaire />{/await}
    {:else if page === 'parametres'}
      <Parametres />
    {:else if page === 'energie'}
      <Energie />
    {:else if page === 'aspirateur'}
      <Aspirateur />
    {:else if page === 'impression'}
      <Impression />
    {:else if page === 'dehors'}
      <Dehors />
    {:else if page === 'moli'}
      <Moli />
    {:else if page === 'automatismes'}
      <Automatismes {route} />
    {:else if page === 'systeme'}
      <Systeme />
    {:else if page === 'reseau'}
      {#await import('./ui/NetworkTopology.svelte') then { default: NetworkTopology }}<NetworkTopology />{/await}
    {:else if page === 'infra'}
      <!-- The house's machines: loaded only when visited. -->
      {#await import('./pages/Infra.svelte') then { default: Infra }}<Infra />{/await}
    {:else if page === 'parc'}
      <!-- What the house watches beyond itself: loaded only when visited. -->
      {#await import('./pages/Parc.svelte') then { default: Parc }}<Parc />{/await}
    {:else}
      <Accueil />
    {/if}
  </main>
  <dialog class="orbite-more" bind:this={moreDialog} aria-labelledby="orbite-more-title">
    <header><h2 id="orbite-more-title">{t('orbite.explore')}</h2><button onclick={() => moreDialog.close()}>{t('commun.fermer')}</button></header>
    <div class="orbite-more-grid">
      {#each nav.filter(item => !primary.some(p => p.path === item.path)) as item (item.path)}
        <a href="#/{item.path}" onclick={() => moreDialog.close()}><Icon name={item.icon ?? 'robot'} size={24}/>{item.name ?? t(item.label)}</a>
      {/each}
      <a href="#/systeme" onclick={() => moreDialog.close()}><Icon name="branch" size={24}/>{t('commun.nav.systeme')}</a>
      <a href="#/parametres" onclick={() => moreDialog.close()}><Icon name="tune" size={24}/>{t('commun.nav.parametres')}</a>
      <a href="#/atelier" onclick={() => moreDialog.close()}><Icon name="atelier" size={24}/>{t('commun.nav.atelier')}</a>
    </div>
  </dialog>


  {#if page !== 'moli'}
    <button class="orb" onclick={() => (moliOpen = true)} aria-label={t('commun.demander_moli')}>
      <Orb size={38} state={moli.busy ? 'thinking' : 'idle'} />
      <span class="orb-text">{t('commun.demander_moli')}</span>
    </button>
  {/if}

  {#if moliOpen}
    <MoliSheet onclose={() => (moliOpen = false)} />
  {/if}

  {#if moli.fix}
    <Correct />
  {/if}

  {#if home.held}
    <PinSheet />
  {/if}

  {#if hub.session?.login_required}
    <!-- From the Internet, not signed in: nothing of the house, only this. -->
    <Login />
  {/if}

  {#if hub.session?.setup && !home.setupLater}
    <Setup onlater={() => (home.setupLater = true)} />
  {/if}

  {#if offline}
    <div class="offline" role="status"><Icon name="offline" size={16} />{t('commun.hors_ligne')}</div>
  {/if}

  <div class="notices" role="status" aria-live="polite">
    {#each hub.notices as n (n.id)}
      <div class="notice">
        <span class="bell"><Icon name="bell" size={18} /></span>
        <div>
          <b>{n.title ?? n.from ?? 'Moli'}</b>
          <p>{n.message}</p>
          {#if n.title && n.from}<small>{n.from}</small>{/if}
        </div>
        <button onclick={() => (hub.notices = hub.notices.filter((x) => x.id !== n.id))} aria-label={t('commun.fermer')}><Icon name="close" size={16} /></button>
      </div>
    {/each}
  </div>

  <div class="notes" role="status" aria-live="polite">
    {#each home.notes as n (n.id)}
      <p class="note {n.tone}">{n.text}</p>
    {/each}
  </div>
</div>

<style>
  /* Moli, always one tap away. */
  .orb {
    position: fixed;
    right: 28px;
    bottom: 28px;
    z-index: 30;
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 10px 22px 10px 10px;
    border: 0;
    border-radius: 999px;
    background: var(--surface);
    box-shadow: var(--shadow-lift);
    font-weight: 650;
    transition: transform 0.2s var(--ease);
  }

  .orb:hover {
    transform: translateY(-2px);
  }

  .notes {
    position: fixed;
    left: 50%;
    bottom: 28px;
    transform: translateX(-50%);
    display: grid;
    gap: 8px;
    z-index: 40;
    pointer-events: none;
  }

  /* The link to Moli dropped (a restart, the Wi-Fi): say so instead of a
     dashboard that silently stops moving. */
  .offline {
    position: fixed;
    top: 12px;
    left: 50%;
    transform: translateX(-50%);
    z-index: 60;
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px 14px;
    border-radius: 999px;
    background: var(--warm);
    color: #1c1407;
    font-size: 13.5px;
    font-weight: 600;
    box-shadow: var(--shadow);
  }

  .phone-foot {
    display: none;
  }

  .notices {
    position: fixed;
    top: 18px;
    right: 18px;
    z-index: 42;
    display: grid;
    gap: 10px;
    width: min(380px, calc(100vw - 32px));
  }

  .notice {
    display: flex;
    gap: 12px;
    align-items: flex-start;
    padding: 14px 12px 14px 14px;
    border-radius: var(--r-md);
    background: var(--surface);
    box-shadow: var(--shadow-lift), inset 0 0 0 1px var(--line);
    animation: drop 0.4s var(--ease) both;
  }

  .notice .bell {
    width: 36px;
    height: 36px;
    flex: none;
    border-radius: 12px;
    display: grid;
    place-items: center;
    background: color-mix(in srgb, #b05fd8 14%, var(--surface));
    color: #b05fd8;
  }

  .notice div {
    flex: 1;
    display: grid;
    gap: 2px;
  }

  .notice p {
    font-size: 14.5px;
    line-height: 1.4;
  }

  .notice small {
    color: var(--ink-3);
    font-size: 12px;
  }

  .notice button {
    border: 0;
    background: none;
    color: var(--ink-3);
    padding: 2px;
  }

  @keyframes drop {
    from {
      opacity: 0;
      transform: translateY(-10px);
    }
  }

  .note {
    background: var(--ink);
    color: var(--bg);
    padding: 12px 18px;
    border-radius: 16px;
    font-weight: 550;
    box-shadow: var(--shadow-lift);
  }

  .note.error {
    background: var(--alert);
    color: #fff;
  }

  /* On phones the Orbite dock owns navigation; notifications stay clear of it. */
  @media (max-width: 760px) {
    .orb { right: 16px; bottom: 100px; padding: 8px; }
    .orb-text { display: none; }
    .notes { bottom: 150px; width: calc(100% - 32px); }
  }
</style>

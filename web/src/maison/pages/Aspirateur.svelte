<script>
 import {hub} from '../lib/home.svelte.js';
 import {t} from '../../lib/i18n.svelte.js';
 import VacuumCard from '../ui/VacuumCard.svelte';
 const robots=$derived(Object.values(hub.devices).filter(d=>d.model==='rockrobo.vacuum.v1'));
</script>
<div class="vacuum"><header><h1 class="page-title">{t('xiaomi.vacuum.title')}</h1></header>
{#each robots as robot(robot.id)}<VacuumCard id={robot.id}/>{:else}<p>{t('xiaomi.account.wait')} <a href="#/parametres">{t('commun.nav.parametres')}</a></p>{/each}
</div>
<style>.vacuum{display:grid;gap:22px}</style>

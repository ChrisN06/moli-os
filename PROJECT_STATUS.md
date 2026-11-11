# Moli OS — état du fork et reprise

## Dernier déploiement — PR #7, 11 novembre 2026

Commit fusionné sur `main` : `de935f60a5080a7432b2abf8869666e305b2845c` ; commit de
branche `600a8c2`. Image épinglée :
`ghcr.io/chrisn06/moli-os@sha256:b09a30a26888eb4285e3a90365c968daf2dae69273832bcc67bc100435da5c05`.
Workflows check, audit et image réussis. Sauvegarde native avant mise à jour vérifiée :
manifeste valide avec neuf fichiers.
Retour à l’image précédente `sha256:26e70014bd3f3e07aaa1daff2d758e23763e0cd85a94305513255c34df74b803`
vérifié ; aucune restauration de données requise.

Après déploiement, `/api/health` répond : 35 appareils, 10/10 pilotes actifs,
244 points et zéro échec journal. Moli et go2rtc sont actifs. Aucune commande physique.
Affichage mobile après déploiement non vérifié.

## Consolidation locale — 11 novembre 2026

Branche consolidée `codex/consolidation-20261111`, issue de `personal/main` (`1603bbb`),
avec les branches publiées intégrées jusqu'à Orbite (`ee86d85`). Le 11 novembre, `main` a
avancé en fast-forward local jusqu'à `bc0c984` ; l'ancien sommet est conservé sur
`codex/main-before-finalize-20261111`. Branche active : `main`. Point de retour de consolidation :
`codex/rollback-20261111` ; état local initial sauvegardé sur
`codex/local-checkpoint-20261111`. Archives validées hors projet :
`../moli-os-backups/2026-11-11/`.

Orbite est l'interface Maison par défaut. Commit local `dcc8338` restaure dans ses pages
les zones Xiaomi avec confirmation, la ventilation instantanée de puissance, l'import CSV,
le panneau de sécurité caméra et l'éclairement instantané. Les modifications locales
So'EAU/CSRF, commandes groupées des volets, adaptations mobiles, rendu réseau et règles
de reprise sont aussi intégrées dans le lot local suivant.

Dernier déploiement documenté : `ee86d85`, image amd64
`sha256:3e74b639214f79375b46e9dff538da3731d46b80b05cdd07c5cb4965a79780f6`, le 7 novembre.
État Portainer non relu pendant cette consolidation. Cette référence ne prouve pas l'image
actuellement active.

Validations observées : 51 tests web, build, i18n (3 438 clés), PWA, catalogue, fuite,
8 tests Somfy, build `moli-os`, Clippy strict et tests ciblés API (61), énergie (31),
Xiaomi (17) passent. Les 33 tests `moli-profile` passent après portabilisation du harnais
loopback. Les tests `moli-tuya` passent également après portabilisation décrite ci-dessous.
Build web garde les avertissements Svelte et taille de bundle préexistants. Ne pas pousser,
déployer, déplacer ou supprimer les anciens checkouts sans accord explicite. Aucun appareil
physique n'a été testé.

### Stabilisation locale et branche `main` — 11 novembre 2026

Les tests TCP de `moli-tuya` utilisent des sockets sur `127.0.0.1:0` ; les adresses
d'appareils restent logiques et sont routées uniquement sous `cfg(test)`. Le code de
production garde sa connexion TCP directe. Résultat : 41 tests Tuya réussis, dont connexion,
preuve d'identité, erreurs et recherche après changement DHCP.

Validations locales : `cargo fmt --all -- --check`, `sh scripts/check.sh`,
`cargo build --locked -p moli-os`, 51 tests web et `npm run build` réussis. Le contrôle i18n
réussit avec 3 438 clés FR/EN et aucune ligne non traduite ; il a été lancé depuis une copie
temporaire excluant les `.DS_Store` non suivis, car le script les traite comme des dossiers.
Le catalogue conserve six avertissements existants. Le build web conserve les avertissements
Svelte et taille de bundle déjà observés. Les tests n'ont envoyé aucune commande physique.

## Orbite — version locale du 6 novembre 2026

Clone dédié `moli-orbite`, branche `feat/orbite-local`, base `59d3213` du clone
`moli-improvements`. Les checkouts antérieurs sont préservés. Aucun commit, push
ni déploiement de cette version locale.

- Habillage Orbite validé par le propriétaire : accueil horloge/météo/climat,
  couleurs par usage, synthèse énergétique multicolore avec les séries existantes.
- Pages et commandes natives conservées, navigation basse et menu Plus ; thèmes
  jour/nuit corrigés face à l'ancien habillage global. Le format téléphone forcé
  se replie indépendamment de la largeur du moniteur. Navigation par flèches,
  contrôles natifs et focus des dialogues conservés.
- Exécution locale via `scripts/orbite-local.sh`, sur `127.0.0.1:8791` uniquement.
  La villa de démonstration fournit les appareils et l'historique simulés.
  Données sous `.local/orbite-data`, ignorées par Git. Voir `LOCAL_ORBITE.md`.
- Validation : compilation Rust locale, build web, 51 tests web, traductions et PWA.
  Dans le navigateur : volet simulé fermé puis rouvert à 100 %, flèche droite entre
  commandes, menu modal, énergie multicolore et caméras de démonstration.
  Absence de débordement horizontal constatée sur Accueil et Énergie à 320 px,
  sur Énergie à 3840 px ; format téléphone forcé et thèmes jour/nuit vérifiés.
- Limites : données de démonstration seulement. Aucun robot, Freebox, fournisseur
  d'eau ni assistant distant raccordé dans cette configuration locale. Les vues
  existent et conservent leurs états indisponibles. Télécommande physique,
  raccordement à la maison et recette NAS restent à faire avant production.


## Compatibilité des stations Freebox — correctif du 6 novembre 2026

L'API répond et l'appairage est valide, mais une liste de stations Wi-Fi peut être
`null`. Le normaliseur web appelait `forEach` sur cette valeur et affichait toute la
Freebox indisponible. Une liste nulle est désormais traitée comme une liste vide,
sans modifier l'autre radio ni les ports Ethernet. Régression couverte par fixture
synthétique ; 41 tests web, build, i18n, PWA et Clippy strict réussis. Les quatre
échecs réseau macOS préexistants restent inchangés.

Code `55a480b` déployé, image applicative
`sha256:9ba5a58844bcfec9c50c71f502070ae9141b5bb6a46db5415b0f57f4e12c13a3`.
CI Linux [check](https://github.com/ChrisN06/moli-os/actions/runs/42175214108),
[audit](https://github.com/ChrisN06/moli-os/actions/runs/42175214098) et
[image](https://github.com/ChrisN06/moli-os/actions/runs/42175213064) réussis.
Recette dans le navigateur interne : ligne connectée, débits et topologie affichés.
Appairage existant conservé, aucun changement sur la Freebox. Sauvegarde native
créée sur le serveur avant remplacement ; sa copie SSH n'avait pas été confirmée
au moment du déploiement. Le 6 novembre, la connexion SSH est rétablie et la référence
Compose du NAS est alignée avec celle de Portainer après comparaison du contenu
et contrôle des empreintes. L'ancien fichier est conservé ; aucun service redémarré.

### Point de reprise du 6 novembre 2026 — fonctionnalité Freebox close

Le propriétaire confirme le bon fonctionnement après déploiement. Relecture ciblée
du normaliseur, de son appel dans le rafraîchissement et des tests : aucun autre
changement nécessaire. Branche `feat/climate-water-camera`, code `55a480b`,
référence documentaire de départ `25e064a`.

Depuis la racine du dépôt : `node --test web/src/maison/lib/freebox-data.test.js`
relancé lors de cette clôture : **3 tests réussis**. `git diff --check` réussi.
Les 41 tests web, le build, i18n, PWA, format et Clippy ainsi que les CI Linux
ci-dessus sont réutilisés : aucun code n'a changé depuis leur exécution.
Les preuves des autres fonctionnalités restent celles de leurs sections datées ;
elles ne sont pas revérifiées dans cette clôture. Aucun nouveau déploiement requis.

Git était synchronisé au début de la clôture (`FETCH_HEAD` et `HEAD` identiques),
sans différence indexée ni modification suivie. Le fichier `.DS_Store` non suivi,
préexistant, reste exclu de la sauvegarde Git. Seul ce point de reprise documentaire
est ajouté ; aucune règle permanente ni procédure d'utilisation ne change.

Prochaines étapes :
1. Vérifier le prochain cycle réel de douze heures de SUEZ, sans modifier la collecte.
2. Lors d'un futur déploiement, maintenir les références Compose du NAS et de
   Portainer alignées. La divergence signalée lors de la clôture est résolue.

## Synchronisation So’EAU — livrée et vérifiée les 5–6 novembre 2026

Code `a9be6fe`, branche `feat/climate-water-camera`, [PR 4](https://github.com/ChrisN06/moli-os/pull/4)
en brouillon, sans fusion. Image applicative déployée :
`sha256:8f59109d4d3471875adf9ccf6b583f996b7bbf5e03825e1681780aa369341c7e`.
Le relais vidéo précédemment corrigé est conservé.

- Connexion directe Rust par le formulaire HTTPS normal (CSRF, cookies temporaires
  sur un hôte fixe), découverte du compteur et télémétrie journalière. Aucun ajout
  de dépendance Home Assistant/MQTT, aucun contournement de contrôle humain.
- Identifiants saisis par le propriétaire, conservés dans le coffre chiffré natif
  après validation. Aucun identifiant ou cookie retourné par l'API. Droits propriétaire
  requis pour connecter ou actualiser le compte.
- Trois derniers mois récupérés, puis actualisés au démarrage et toutes les 12 h.
  Un refus temporaire observé pendant la recette a motivé un unique nouvel essai
  après une minute ; jamais de nouvel essai devant un contrôle humain.
- Connexion réelle confirmée par le propriétaire, historique et total mensuel vérifiés
  dans Énergie et Accueil. Reprise autonome après remplacement de l'application
  confirmée sans ressaisie ni clic d'actualisation. Les anciennes dates sont conservées.
- Sauvegarde native avec compte chiffré copiée dans le dossier privé, SQLite intègres.
  Anciennes mesures, heures d'énergie terminées, tarifs et abonnement préservés.
  Référence Compose synchronisée ; autres services conservés. La tentative de
  sauvegarde supplémentaire du lendemain n'a pas été confirmée après expiration
  de la session Portainer ; la sauvegarde avec compte vérifiée reste disponible.

Validation : 99 tests API, 40 tests web, build, i18n, PWA, format et Clippy stricts.
CI Linux [check](https://github.com/ChrisN06/moli-os/actions/runs/42114905731),
[audit](https://github.com/ChrisN06/moli-os/actions/runs/42114905728) et
[image amd64/arm64](https://github.com/ChrisN06/moli-os/actions/runs/42114903495) réussis.
Les quatre échecs réseau macOS préexistants de `moli-profile` restent inchangés.
Le cycle réel de douze heures n'a pas encore été observé ; le déclenchement au
démarrage et la temporisation de reprise sont vérifiés. Les données plus anciennes
restent importables en CSV. Détails privés dans le dossier de la maison.

## Améliorations du 5 novembre 2026 — livraison initiale, complétée ci-dessus

Branche `feat/climate-water-camera`, base publiée `b8845ca`, code `40f778d`.
[PR 4](https://github.com/ChrisN06/moli-os/pull/4) en brouillon, sans fusion.
Les anciens checkouts modifiés sont préservés. Application construite sur `a0124df` ;
les changements suivants concernent uniquement le relais vidéo.

- Salon : température, humidité et consigne **rapportée**, périodes 24 h / 7 j /
  30 j, axes séparés et consigne en paliers. Historique réel et curseur clavier
  vérifiés dans le navigateur interne. Une dérogation Somfy peut différer.
- Eau : barres journalières dans Énergie, total du mois courant sur Accueil,
  import CSV depuis Paramètres réservé au propriétaire. Import réel et affichage
  vérifiés. Dates inconnues distinguées des mesures nulles, fusion par date et
  stockage dans le coffre natif. **Synchronisation automatique non livrée** :
  la connexion du fournisseur comporte un contrôle humain. Aucun contournement
  ni récupération de cookie personnel.
- Robot : coordonnées visibles depuis la base orange. Le recalage existant utilise
  déjà la base ; test prouvant que déplacer le robot ne déplace pas la zone.
  Aucune zone réelle modifiée, aucun moteur commandé. À la dernière recette,
  découverte miIO en délai dépassé : carte réelle non vérifiable, robot hors ligne.
- Caméra : relais go2rtc 1.9.15 corrigé pour le démarrage SD puis HD de `hlc7`.
  Le premier essai a échoué à initialiser HLS et a été annulé. La seconde version
  conserve aussi la première image clé HD consommée pendant la détection.
  Flux réel : initialisation et SPS des images à **2304 × 1296**, lecture active
  dans le navigateur interne. Cadrage correct confirmé par le propriétaire sur iPhone après
  un chargement long. Safari Mac non confirmé séparément pour Xiaomi. Les deux Tapo sont
  confirmées par le propriétaire sur Safari et iPhone, avec démarrage parfois lent.
  Autres modèles et demandes SD/auto inchangés.

Images déployées, épinglées par digest :
- Application : `sha256:35139423f0784a41e6b79698936d01bfeb01ee2e117281405c212d0b594a57b0`.
- Relais : `sha256:bb735ef5052eecdafb9b29d6120229ed153c5c13e350b88cfbd4ccbe7d18908b`,
  branche de construction `release/xiaomi-hd-relay`, commit `ad545aa`.
  Cette branche remplace le Dockerfile racine uniquement pour le workflow image ;
  ne pas la fusionner dans l'application.

Contrôles : 40 tests web, build, traductions, PWA, fuites, 92 tests API,
format et Clippy workspace réussis. Tests Go ciblés réussis ; `GOAMD64=v1`, CGO
désactivé. CI Linux [check](https://github.com/ChrisN06/moli-os/actions/runs/42106295397)
et [audit](https://github.com/ChrisN06/moli-os/actions/runs/42106295458) réussis.
[Application](https://github.com/ChrisN06/moli-os/actions/runs/42065210649) et
[relais](https://github.com/ChrisN06/moli-os/actions/runs/42106370755) construits pour
amd64/arm64. Catalogue : 36 paquets, six avertissements existants, aucune erreur.
Quatre échecs macOS préexistants de `moli-profile` liés aux adresses réseau.

Sauvegardes avant/après copiées dans le dossier privé ; intégrité des deux SQLite
vérifiée. Toutes les anciennes mesures d'historique, heures d'énergie terminées,
tarifs et abonnement préservés. Les neuf conteneurs voisins gardent leurs identifiants
et états. Référence Compose synchronisée avec contrôle de l'empreinte initiale.
Détails d'installation et échantillons vidéo exclusivement privés.

## Réseau Freebox et thème bleu nuit — point de reprise du 4 novembre 2026

Fonctionnalité livrée et vérifiée sur Mac ; projet Moli OS toujours actif.
Version applicative `005fcf509f8dffa86f7d34d49d83ccbdd14ecf88`, branche
`feat/freebox-network`, clone de travail isolé `moli-freebox`, base `f599497`.
Les commits `35ad98f` (intégration) et `005fcf5` (débordement du menu) sont publiés.
[PR 2](https://github.com/ChrisN06/moli-os/pull/2) en brouillon, base
`fix/vacuum-controls-map` ; aucune fusion dans `personal/main`.

### Livré et preuves

- Page `#/reseau` : trois ports Ethernet, Wi-Fi Pop et répéteur Ethernet,
  noms des appareils, signal et vitesses PHY. Données réelles observées sur Mac.
- Actualisation nominale de cinq secondes, cache serveur partagé de 4,5 secondes,
  lectures à la demande et arrêt sans vue concernée visible. Le délai
  réseau s'ajoute à l'intervalle ; ne pas promettre une synchronisation exacte.
- Flux descendants verts et montants rouges : débit lissé, densité et épaisseur
  variables, amplitude d'ondulation fixe. Identité MAC conservée lors des changements
  d'association ; transition vérifiée sur données synthétiques.
- Accueil : les deux débits globaux Freebox uniquement. Paramètres : appairage
  propriétaire avec validation physique sur la Freebox, confirmée par le propriétaire.
  Jeton dans le coffre chiffré ; appairage conservé après le dernier redémarrage.
- Palette bleu nuit Maison et Atelier observée. Page Réseau vérifiée à 1280 × 720
  sans débordement général ; le menu latéral défile séparément. Panneau de port
  fermé par second clic, détails Wi-Fi et absence d'erreur console vérifiés.
- Image amd64 générique construite hors du serveur, CI Linux et sauvegardes
  vérifiées avant remplacement. Historique, tarif et identifiants d'appareils/points
  conservés. Les autres services gardent leurs dates de création et états visibles.
  Détails d'installation et captures exclusivement dans le dossier privé voisin.

### Vérifications et limites

Depuis la racine du clone `moli-freebox`, sur le code livré :

- `npm test --prefix web` : 32 tests réussis ; relancé lors de cette clôture.
- `node scripts/i18n-check.mjs` : 3352 clés FR/EN, aucun texte en dur détecté ; relancé.
- `npm run build --prefix web` : réussi avant livraison, avertissements Svelte
  existants conservés. Pas de changement applicatif depuis ce build.
- `cargo test -p moli-api -j2` : 88 tests réussis avant livraison, dont garde
  propriétaire et HMAC Freebox. PWA et contrôle générique de fuites réussis.
- `sh scripts/check.sh` : format et Clippy workspace réussis ; quatre échecs
  `moli-profile` sur macOS liés aux adresses de boucle locale indisponibles.
  Aucune modification du réseau du Mac. Catalogue : 36 paquets, six avertissements.
- CI Linux sur `005fcf5` : [check 41812414207](https://github.com/ChrisN06/moli-os/actions/runs/41812414207),
  [image 41812444735](https://github.com/ChrisN06/moli-os/actions/runs/41812444735)
  et audit réussis. Ces résultats récents sont réutilisés ; aucune reconstruction
  ni nouvelle exécution sur les équipements pendant la clôture documentaire.
- Recette réelle du déploiement : Safari Mac et navigateur intégré. Pas de recette
  iPhone, de débranchement réel pour tester un déplacement de port, ni de commande
  physique par l'agent. CPU prolongé non mesuré ; mesures mémoire ponctuelles privées.
- Cette version cible Pop (trois ports, deux radios). Les vitesses Wi-Fi sont PHY,
  distinctes du trafic ; Ethernet mesure le port entier, pas chaque client d'un switch.
  Le câblage des switchs intermédiaires n'est pas connu. Sur fenêtre compacte,
  les libellés sont réduits pour garder toute la topologie visible.

### Points d'entrée et reprise du lot

- API et tests : `crates/moli-api/src/freebox.rs`, `crates/moli-api/tests/household.rs`.
- Données et polling : `web/src/maison/lib/freebox-data.js`, `freebox.svelte.js` ;
  tests de normalisation : `freebox-data.test.js`.
- Rendu : `web/src/maison/ui/NetworkTopology.svelte`, `ui/network/renderer.js`
  et `template.html` ; accueil et appairage : `FreeboxRates.svelte`, `FreeboxSetup.svelte`.
- Palette commune : `web/src/night.css` ; documentation :
  [dashboard](docs/components/dashboard.md#réseau-freebox-et-habillage-bleu-nuit)
  et [API](docs/components/api.md#freebox-locale-lecture-seule).
- Preuves et retour arrière privés : `../moli-maison/freebox-20261104/DIAGNOSTIC.md`.
- À l'ouverture de cette clôture : arbre applicatif identique à `005fcf5`, index vide,
  seul `.DS_Store` non suivi, exclu et conservé. `PROJECT_STATUS.md`, `README.md`
  et `README.fr.md` restent à commiter après accord ; index non modifié. Aucun
  nouveau commit ni push effectué pendant cette clôture. Les motifs génériques
  de fuite ont été recherchés dans les 24 fichiers du lot et la documentation,
  sans occurrence non fictive ; liste noire privée indisponible, historique complet
  non audité. Liens et ancres documentaires vérifiés.
- Le checkout historique `moli-os` reste à préserver : corruption Git constatée
  le 3 novembre, pas réauditée ici. Ne pas y faire de pull/reset/clean/réparation.

## Actualisation robot — 3 novembre 2026

État historique du 3 novembre, conservé pour les preuves robot. La version courante et le clone de reprise figurent ci-dessus.

- Version déployée : `b6df9a4`, branche `fix/vacuum-controls-map`, [PR 1](https://github.com/ChrisN06/moli-os/pull/1), encore en brouillon. `personal/main` reste à `1603bbb`.
- Option d’autorisation des déplacements retirée. Les commandes sont disponibles après vérification du compte et du modèle par le pilote. La garde et les échéances restent actives.
- **Recette utilisateur réelle réussie** : télémétrie, démarrage d’un nettoyage classique, pause et retour à la base. Le propriétaire a lui-même effectué ces commandes. Aucun mouvement n’a été lancé par l’agent.
- **Carte rétablie et affichée dans Moli** : lecture de référence par le RPC cloud Xiaomi, puis téléchargement et décodage RR existants. La carte réelle, la base et la sélection de rectangle sont visibles. Les demandes locales avaient expiré ; le cloud a ensuite révélé des réponses de préparation `retry`.
- La lecture cloud réutilise le client existant. Route limitée à `get_map_v1`, paramètres vides, identifiant d’appareil numérique borné et identifiant de requête 1..9999 ; aucune commande moteur cloud. Les seules réponses exactes `["retry"]` sont relues au plus cinq fois, à 500 ms d’intervalle. Les autres erreurs ne sont pas répétées. Version déployée après CI Linux et sauvegarde ; ses 14 tests Xiaomi et la recette de carte réelle passent.
- **Nettoyage zoné validé par le propriétaire le 3 novembre 2026** : il indique avoir effectué un essai réussi. Aucun mouvement lancé par l’agent. Les coordonnées et réglages personnels restent privés ; aucun nettoyage global de repli. Prochaine étape : poursuivre les recettes restantes selon la priorité du propriétaire.
- Sauvegardes natives prises avant chaque remplacement ; la première paire de bases passe l’intégrité SQLite. La sauvegarde finale a été vérifiée. CI Linux verte pour la version finale publiée. Le contrôle global macOS retrouve les quatre échecs moli-profile connus.
- **Git local à préserver** : AGENTS.md modifié, PROJECT_STATUS.md et .DS_Store non suivis ; modifications applicatives copiées dans ce checkout sans commit local. Un pack Git y est de nouveau illisible. Aucune réparation ni suppression effectuée. Publication depuis le clone isolé `/private/tmp/moli-robot-release-20261103` ; détails et preuves dans `../moli-maison/robot-20261103/DIAGNOSTIC.md`. Ne pas faire de pull/reset/clean sur ce checkout modifié.

Ce document décrit l’avancement du fork, pas toutes les capacités amont.
Architecture et règles : [AGENTS.md](AGENTS.md), [index technique](docs/README.md).
Les configurations, captures et preuves d’installation restent dans le dossier privé voisin.
Aucun service ni équipement réel n’a été retesté pendant cette clôture documentaire.

## Fonctionnalités vérifiées

| Fonctionnalité | Preuve et portée | Repères pour le code/documentation |
|---|---|---|
| Web français, navigation responsive | Recette utilisateur Safari Mac/iPhone du socle ; versions ultérieures soumises à recette ciblée | `web/src/maison`, [dashboard](docs/components/dashboard.md) |
| Somfy/Overkiz cloud | États et capacités des volets, température/humidité et capteurs observés ; droits de commandes conservés lors des mises à jour | `crates/moli-overkiz`, [intégration](integrations/overkiz/README.md) |
| Historisation et import énergétique | Données importées conservées ; comparaison avant/après du 3 novembre : intégrité SQLite, lignes historiques antérieures à la mise à jour et tarifs identiques | `crates/moli-history`, `crates/moli-energy`, `tools/somfy`, [énergie](docs/components/historique-energie.md) |
| Estimation de puissance et Dehors | Estimation, ancienneté, données air/UV et graphique d’éclairement observés en production le 1 novembre | [dashboard](docs/components/dashboard.md#estimation-air-et-volets), `web/src/maison/lib/counter-power.js` |
| Réglages et accueil intérieur | Page Paramètres, comptes caméra et tarif fixe ; sélection de température/humidité implémentée et livrée le 2 novembre | [dashboard](docs/components/dashboard.md#réglages-vidéo-et-température-intérieure) |
| Vidéo Tapo | Direct confirmé par l’utilisateur sur Safari Mac/iPhone ; playlists à nouveau vérifiées le 3 novembre | `crates/moli-tapo-camera`, [intégration](integrations/tapo-camera/README.md) |
| MI-Light RGB+CCT | Deux zones, couleurs et réglages confirmés par l’utilisateur après le correctif `0d2ebd8` | `crates/moli-milight`, [intégration](integrations/milight/README.md) |
| Xiaomi robot : télémétrie, commandes et carte | Le 3 novembre : télémétrie, nettoyage classique, pause et retour à la base confirmés par le propriétaire ; carte réelle affichée dans Moli | `crates/moli-xiaomi/src/vacuum.rs`, [robot](integrations/xiaomi-vacuum/README.md) |
| Xiaomi robot : nettoyage zoné | Essai physique réussi confirmé par le propriétaire le 3 novembre ; aucune commande lancée par l’agent | `crates/moli-xiaomi/src/vacuum.rs`, [robot](integrations/xiaomi-vacuum/README.md) |
| Xiaomi caméra : direct | Direct confirmé par l’utilisateur ; playlists vérifiées le 3 novembre, sans consultation des images par l’agent | `crates/moli-xiaomi/src/camera.rs`, [caméra](integrations/xiaomi-camera/README.md) |
| Réseau Freebox et thème commun | Version `005fcf5` déployée le 4 novembre ; appairage réel, ports, répéteur, PHY, deux débits sur l’accueil et rendu Mac vérifiés | `crates/moli-api/src/freebox.rs`, `web/src/maison/ui/NetworkTopology.svelte`, `web/src/night.css` |
| Exploitation Docker | Version `005fcf5` déployée après CI Linux et sauvegardes ; autres services et historique conservés, Compose synchronisée | [exploitation](docs/components/exploitation.md) ; preuves privées `freebox-20261104/DIAGNOSTIC.md` |

« Vérifié » décrit uniquement la portée indiquée ; cela ne garantit pas toutes les
capacités d’un protocole, ni leur fonctionnement permanent.

## Partiellement opérationnel ou en développement

| Sujet | Livré / vérifié | Ce qui manque |
|---|---|---|
| Curseurs lumineux en direct | Coalescence et bornes testées ; livrés dans `a223b4a` | Recette tactile iPhone avec les lampes réelles encore non consignée |
| Favori Perso | Première ambiance, mémorisation serveur et rappel testés en démo ; livré dans `a223b4a` | Recette des réglages réels et persistance après redémarrage encore non consignée |
| Coût historique et graphiques tactiles | Correction au tarif courant, sélection tactile et rendu livrés dans `fb6e9d7`, contrôles locaux/CI réussis | Confirmation utilisateur finale des montants historiques et du geste iPhone non consignée |
| Volets à pourcentage | Commandes exposées et positions réelles observées ; slider et garde implémentés | Une recette physique documentée du parcours complet reste nécessaire avant d’affirmer toutes les commandes validées |
| PWA | Manifeste et contrôle `scripts/pwa-check.mjs` présents | Installation et sessions iPhone sous HTTPS local non validées ; aucun contrôle hors ligne promis |

## Bugs, limites et vérifications manquantes

- **Robot** : nettoyage zoné confirmé par le propriétaire ; coordonnées et réglages non audités par l’agent.
  Préserver le contrôle du repère et l’absence de nettoyage global de repli.
  Points d’entrée : `crates/moli-xiaomi/src/map.rs`, `crates/moli-xiaomi/src/cloud.rs`,
  `crates/moli-api/src/xiaomi.rs`, `web/src/maison/ui/VacuumCard.svelte`.
- MI-Light ne fournit pas d’état physique des ampoules : accusé UDP et état supposé
  seulement. Une autre application/télécommande peut désynchroniser l’affichage.
- Puissance calculée : moyenne entre variations récentes d’index, pas mesure TIC.
  Ne pas utiliser les heures importées pour produire une puissance actuelle.
- Coût historique au tarif fixe courant : estimation au tarif choisi, pas reconstitution
  garantie des factures passées. Ne pas additionner le total général et les circuits.
- Réseau host utilisé pour les médias Xiaomi après essai autorisé ; isolation réseau
  réduite. Relais loopback uniquement, RTSP/WebRTC désactivés. Conserver ces bornes.
- HTTP LAN accepté pour les saisies précédentes ; ne pas le présenter comme chiffré.
  HTTPS local reste à préparer. Aucun changement de politique SSH pour les tunnels.
- Plafonds mémoire observés ; quota CPU effectif non démontré sur l’ancien Docker,
  malgré sa déclaration dans Compose.
- Dernier contrôle complet local : quatre échecs préexistants `moli-profile` sur macOS
  liés aux changements d’adresse. CI Linux verte sur `005fcf5` ; ne pas les ignorer
  ni modifier le réseau du Mac pour faire passer les tests.
- Catalogue : six avertissements de fixtures au dernier contrôle, aucune erreur.
  Certains textes amont indiquent encore 24 intégrations ; dernier catalogue contrôlé : 36 paquets.
- `web/package.json` mentionne deux fois `vacuum-map.test.js` dans la commande de tests.
  Constat documentaire seulement ; aucune correction applicative dans ce lot.
- `.DS_Store` non suivi préexistant : hors périmètre, non supprimé, à exclure de tout commit.

## Prévu, sans validation matérielle

- Bambu A1 mini/AMS Lite : réutiliser `crates/moli-bambu`, imposer et tester la lecture
  seule côté serveur avant toute connexion réelle ; ne pas promettre le flux caméra.
- Interrupteur Tapo/Kasa et autres éclairages Xiaomi : modèles/protocoles à identifier
  avant d’annoncer une compatibilité ou d’étendre un pilote.
- Tablette murale : dashboard paysage, kiosque, nuit, puis faisabilité du réveil caméra
  mesurée sur le matériel ; aucune application dédiée présumée nécessaire.
- Shield/Android TV : navigation focus/télécommande réutilisant l’API ; le pilote
  Android TV amont ne constitue pas une application domotique TV livrée.
- Scénarios personnels et analyse énergétique : après validation des données et commandes,
  en réutilisant le moteur existant. Pas de nouvelle base ni de broker par défaut.

## Décisions à conserver

- Cœur Rust, dashboard Svelte embarqué, API commune et SQLite existants ; intégrations
  modulaires. Ne pas réécrire les fonctions déjà opérationnelles.
- Somfy cloud pour une passerelle ancienne ; aucune API locale supposée.
- HLS Safari, médias à la demande via go2rtc ; aucun enregistrement ni transcodage permanent.
- Coffre chiffré et clé maître externe préservés ; comptes saisis par le propriétaire.
- Fuseau de l’import énergétique : `Europe/Paris`, changements été/hiver compris.
- Dev Apple Silicon, image amd64 générique construite hors du NAS ; aucune compilation native CPU spécifique.
- Fork : `main` réservé au suivi amont, travail personnel sur `personal/main`, branches
  courtes et worktrees isolés. Vérifier les branches disponibles avant de les utiliser.
- Documentation publique sans installation personnelle ; détails et preuves dans
  `../moli-maison/`. Ne jamais recopier les configurations ni captures privées ici.

## Prochaines étapes ordonnées

1. Ouvrir `#/reseau` sur l’iPhone du propriétaire et vérifier lisibilité, topologie complète
   et détails des ports ; consigner le résultat dans le dossier privé. Aucun essai physique
   ni déploiement supplémentaire n’est autorisé par cette clôture.
2. Au besoin, proposer une recette de déplacement de port au propriétaire ; ne rien débrancher automatiquement.
3. Consigner les autres recettes manquantes : curseurs iPhone, Perso, coût historique,
   graphiques tactiles et volets. Préparer HTTPS/PWA selon priorité utilisateur.
4. Examiner les PR en brouillon et leur dépendance avant de proposer une fusion ; aucune fusion autorisée ici.

## Archive des contrôles et de la reprise robot — 3 novembre 2026

Vérifications récentes du code publié, exécutées depuis le clone isolé mentionné plus haut :

- `cargo test --locked -p moli-xiaomi` : 14 tests réussis sur la version finale, dont RPC borné et reprise `retry`.
- Tests ciblés `moli-api` et `moli-xiaomi` : 99 réussis avant le dernier ajout de reprise ;
  les 14 tests Xiaomi finaux couvrent cet ajout.
- Depuis `web/`, `npm test` : 30 tests réussis ; `npm run build` réussi.
  Aucun changement web depuis ces résultats.
- `sh scripts/check.sh` : format et Clippy strict réussis ; quatre échecs macOS
  préexistants `moli-profile` liés aux changements d’adresse. Aucun réglage réseau modifié.
- Catalogue : 36 paquets, six avertissements de fixtures. Traductions : 3300 clés.
- CI Linux finale : `check` 41634427495 et `image` 41634612322 réussies.
- Recette réelle : carte affichée dans Moli ; commandes classiques testées par le propriétaire.
  Le nettoyage zoné est également confirmé par le propriétaire le 3 novembre.

Ces preuves sont réutilisées après comparaison octet par octet des 13 fichiers applicatifs
locaux avec la version publiée. Aucun build relancé pendant la clôture : préserver l’espace disque.
Le 3 novembre, les caches régénérables de `target/debug` ont été supprimés à la demande du
propriétaire. `target` est passé d’environ 32 Gio à 118 Mio ; binaire de démonstration conservé.
La prochaine compilation recréera ses caches. L’espace disponible n’a pas augmenté immédiatement ;
ne pas présenter les tailles supprimées comme des blocs déjà libérés.

État Git à préserver :

- Checkout principal sur `fix/vacuum-controls-map`, HEAD `1603bbb`, modifications applicatives
  non commitées ici mais identiques aux quatre commits déjà publiés : `3148df1`, `7837406`,
  `281d1b7`, `b6df9a4`. Aucun nouveau commit applicatif à créer à partir de cette copie.
- Clone isolé sur `b6df9a4`, branche publiée propre ; PR 1 en brouillon, non fusionnée.
- `AGENTS.md` comporte des modifications documentaires préexistantes ; ne pas les inclure implicitement.
  Ce fichier `PROJECT_STATUS.md` reste non suivi ; `.DS_Store` préexistant exclu.
- Pack Git local illisible et certaines commandes bloquantes : aucun reset, clean ou réparation.
  Ne pas assimiler un contrôle expiré à un index vide. Revérifier avant toute opération Git.
- Contrôles documentaires du 3 novembre : liens relatifs valides, aucun espace final,
aucune occurrence des motifs génériques de fuite dans le fichier proposé ; liste noire
privée indisponible. Index du clone vide ; état et index du checkout principal non
vérifiables (commandes expirées après cinq secondes). Aucun index modifié.

La clôture du 3 novembre avait autorisé le commit et le push du seul `PROJECT_STATUS.md`.
Cette autorisation historique ne couvre pas les nouveaux documents du 4 novembre.
Le commit documentaire est préparé dans le clone isolé, sur `fix/vacuum-controls-map` ;
le checkout principal conserve ses modifications. `AGENTS.md` et les fichiers privés sont exclus.
Vérifier le résultat Git à la reprise ; ce document ne présume pas une fusion de la PR.

Lire d’abord les instructions applicables, ce fichier et la documentation du composant.
S’il existe, lire `../moli-maison/AGENTS.md` et le dernier état privé ; ne pas considérer
une ancienne connexion SSH, session Portainer ou image active comme toujours valide.

Après une étape importante, `fonctionnalite-terminee` peut actualiser ce fichier et
les preuves du périmètre. `projet-termine` reste réservé à la clôture définitive.

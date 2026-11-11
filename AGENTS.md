# Moli OS — guide pour agents

Ce fichier est lu par n'importe quel agent (Claude Code, Codex…). Rien ici ne suppose un agent particulier.

## Avant tout

1. Au début de chaque nouvelle tâche de développement, lire **`PROJECT_STATUS.md`**, puis `docs/README.md` (index, état, règles), puis `docs/ARCHITECTURE.md` (les décisions prises, à ne pas rediscuter sans raison nouvelle) et la doc du composant touché (`docs/components/`).
2. **La maison de l'utilisateur** : si un dossier `../moli-maison/` existe à côté de ce dépôt, lire son `AGENTS.md`. Il décrit l'installation de cette maison (machine, déploiement, règles de la famille). Rien de ce dossier n'entre jamais dans ce dépôt.
3. Examiner la branche, les différences indexées/non indexées et les fichiers non suivis. `git pull --ff-only` avant de commencer, seulement sur un checkout propre, sans autre conversation en train de l’utiliser ; sinon préserver le travail et préparer une synchronisation isolée. Ne jamais écraser des changements pour satisfaire cette étape.
4. **Installer Moli chez quelqu'un** (pas modifier son code) : suivre `.claude/skills/installer-moli/SKILL.md`, lisible par tout agent.

## Commandes

Tout tourne en conteneur, aucune toolchain locale requise :

```
.\scripts\dev.ps1 sh scripts/check.sh   # Windows : fmt + clippy pedantic -D warnings + tests + catalogue + fuites, obligatoire avant chaque commit
sh scripts/check.sh                     # Linux, macOS : la même chose, dans l'image docker/dev.Dockerfile
cd web && npm run build && cd .. && node scripts/i18n-check.mjs   # si l'interface a changé : build et traductions
docker build -t moli-os .               # l'image (Svelte → Rust musl → scratch)
```

La CI de GitHub (`.github/workflows/check.yml`) refait tout cela sur chaque pull request.

Construire, lancer, sauvegarder, revenir en arrière : `docs/components/exploitation.md`.

## GitHub et déploiement en production

- Utiliser `gh` pour les opérations GitHub, y compris Actions, les builds CI, les releases et les vérifications. Employer `gh run list/view/watch`, `gh pr checks`, `gh release list/view` ou `gh api` selon le besoin. Privilégier les CLI et API au navigateur. Vérifier le dépôt et la branche avant chaque opération.
- Administrer et déployer les stacks via l’API REST HTTPS de Portainer. Prendre l’URL et la référence du coffre 1Password dans les consignes privées de l’installation. Pour chaque appel à l’API Portainer, utiliser `curl --cacert "$HOME/.config/portainer/cert.pem"`. Ne jamais utiliser `-k` ni `--insecure`. Identifier l’environnement et la stack par des requêtes `GET` avant toute action.
- Résoudre la référence `op://…` via `op run`, uniquement pour le processus qui envoie l’en-tête `X-API-Key` à Portainer. Passer cet en-tête à `curl -H @-` via l’entrée standard. Garder la clé hors des arguments de commande, des fichiers et des journaux. Désactiver les traces shell et les sorties HTTP verbeuses.
- Tester les accès en lecture seule avec `gh run list`, `gh release list` et `GET /api/stacks`. Demander une confirmation explicite avant toute modification en production, déploiement et rollback compris. Suivre les procédures de sauvegarde, de déploiement et de retour arrière de `docs/components/exploitation.md`.

## Piloter une instance

- MCP : `http://<hôte>:8790/mcp` (streamable HTTP). Liste des outils et règles : `docs/components/api.md`.
- REST + SSE : `http://<hôte>:8790/api/…` (même doc).
- Un agent est soumis à la **garde** : dans une pièce protégée ou aux heures calmes, sa commande devient une demande qu'un humain valide (202). Un automatisme écrit par un agent reste un brouillon jusqu'à la validation d'un humain.

## Règles

- Jamais de secret dans un fichier : ils vivent dans le coffre chiffré de Moli (`moli-os secrets set <instance> <nom>`, valeur par un tube, jamais au clavier ni affichée) ; la clé maître arrive par l'environnement. Ne jamais régénérer `MOLI_MASTER_KEY`.
- Jamais de donnée personnelle dans le dépôt : prénoms, domaine, adresses du réseau, adresses MAC, identifiants d'appareils, clés. Dans les tests et les captures d'appareils, des valeurs fictives (`aa:bb:cc:dd:ee:ff`, `192.168.1.x`, `user@example.com`). `scripts/leak-check.sh` le vérifie ; avec un dossier `../moli-maison/`, il applique aussi sa liste noire privée.
- Ce dépôt est public, messages de commit compris : rien qui désigne une maison, une personne ou un lieu.
- L'interface et les messages sont traduits dès l'écriture : chaque texte visible passe par `t('zone.cle')` (web) ou `tr!("zone.cle")` (Rust), dans `locales/fr` **et** `locales/en`. Voir `docs/components/traduction.md`.
- Jamais de LLM dans le chemin d'exécution des commandes : l'IA crée et explique, le moteur est déterministe.
- La performance est un budget (`docs/PERFORMANCE.md`) : une régression se corrige avant de continuer.
- Une commande sur un appareil physique = effet réel dans une maison habitée. **Jamais les pièces protégées** (on y dort, des enfants parfois). Tester sur une cible neutre et remettre l'état initial.
- Ne rien envoyer à des tiers (messages, mails) sans l'accord du propriétaire.


## Architecture et repères du fork

Résumé d’orientation ; les contrats détaillés restent dans [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md)
et les [documents de composants](docs/README.md#les-composants). L’avancement du fork,
les preuves datées et les prochaines tâches vivent uniquement dans [PROJECT_STATUS.md](PROJECT_STATUS.md).
Les chiffres anciens des README amont ne constituent pas un inventaire actuel ni une recette matérielle.

| Emplacement | Responsabilité |
|---|---|
| `crates/moli-core` | Domaine sans entrées/sorties : identités stables, points typés et unités |
| `crates/moli-runtime` | Hub, bus, garde, échéances, journal, coffre et supervision des pilotes |
| `crates/moli-api` | REST `/api`, SSE, MCP, sessions, ressources web embarquées et accès média |
| `crates/moli-os` | CLI, configuration TOML, composition des services, backup/restore |
| `crates/moli-net` | Transports bornés, TLS et validation réseau |
| `crates/moli-history`, `crates/moli-energy` | SQLite : historique de valeurs et agrégats énergétiques |
| Autres `crates/moli-*`, `integrations/` | Pilotes natifs, profils déclaratifs, manifestes et documentation de protocoles |
| `crates/moli-automation`, `crates/moli-assistant` | Moteur déterministe ; assistant séparé de l’exécution des commandes |
| `web/src/maison`, `web/src/lib` | Dashboard Maison/Atelier, composants partagés, état et traductions |
| `locales/fr`, `locales/en` | Catalogues partagés français/anglais |
| `tools/somfy`, `scripts/`, `.github/workflows/` | Import et contrôles ; CI et construction des images |
| `mobile/`, `agent/` | Clients et agents amont ; ne pas les confondre avec une interface Android TV livrée |

Technologies déclarées : workspace Rust édition 2024, minimum 1.89 ; Tokio, Axum,
Serde, rusqlite/SQLite embarqué, rustls/ring, rumqttc et rmcp. Dashboard Svelte 5,
Vite 7, Three.js pour la 3D, hls.js pour les navigateurs sans HLS natif.
`Cargo.lock` et `web/package-lock.json` fixent les dépendances résolues.
Les versions de construction sont dans `Dockerfile` et `docker/dev.Dockerfile` ;
la CI web utilise Node 24. Vérifier ces fichiers plutôt qu’installer arbitrairement une autre version.

Le dashboard appelle l’API commune ; le hub valide puis route les commandes vers
les pilotes supervisés. Une acceptation n’est pas une confirmation physique.
Les vidéos passent par le relais go2rtc et les routes média, pas par le bus SSE.
Réutiliser les services et composants existants ; ne pas ajouter de base, broker ou API parallèle par défaut.

## Vérifications complémentaires

Toutes les commandes suivantes partent de la racine. Elles complètent le contrôle
obligatoire ci-dessus ; ne pas installer de dépendances implicitement lors d’une simple vérification.

```sh
# Dans l’image de développement, ou avec la toolchain compatible déjà disponible :
cargo build --locked -p moli-os
cargo test --locked -p moli-xiaomi -p moli-tapo-camera
# Avec les dépendances web déjà présentes :
(cd web && npm test && npm run build)
node scripts/i18n-check.mjs
node scripts/pwa-check.mjs
python3 -m unittest discover -s tools/somfy -p 'test_*.py'
python3 scripts/prepare-personal-dev.py --self-test
(cd web && node scripts/solar-sim.check.mjs)
```

Le binaire embarque `web/dist` : construire le web avant un binaire destiné à la recette.
`sh scripts/check.sh` reste le contrôle Rust/catalogue/fuites avant commit ; il ne remplace
ni les contrôles web ni Safari réel. La CI Linux est décrite dans `.github/workflows/check.yml`.
Les échecs connus et les résultats effectivement observés sont dans `PROJECT_STATUS.md`.

## Contraintes de compatibilité et de production

- Profil de développement : Mac Apple Silicon à mémoire limitée. Réutiliser les caches,
  limiter les tâches de compilation ; ne pas compiler sur le serveur de production ancien.
- Cible de production : Docker Linux amd64 sur ancien AMD (profil N54L/DSM 7.1.1).
  Conserver le x86-64 générique du Dockerfile et des dépendances natives ; jamais
  `target-cpu=native`. Une image amd64 ne garantit pas à elle seule la compatibilité CPU/noyau.
- Image finale `scratch`, non-root : aucun shell embarqué. Tester les commandes CLI
  directement ; ne pas présumer qu’un `docker exec … sh` fonctionne.
- Ressources limitées : préserver les plafonds mémoire ; vérifier les limites réellement
  appliquées par Docker. Une valeur Compose déclarée n’est pas une preuve de quota CPU effectif.
- Aucun enregistrement ni transcodage vidéo permanent. Conserver la libération des flux
  hors écran et les protections des routes média. Ne pas exposer go2rtc directement au LAN.
- Ni nouvelle exposition Internet, ni changement de réseau, de proxy ou d’autres services
  sans examen et autorisation adaptés. La configuration privée fait foi pour le déploiement réel.
- Avant remplacement de Moli : sauvegarde native cohérente, image épinglée, contrôle des
  volumes/droits et retour arrière préparé. Jamais deux instances sur le même volume.
  Après migration de données, restaurer une image et une sauvegarde compatibles ensemble.
- Préserver la garde, les délais, les états hors ligne, la fraîcheur des données et les
  droits existants. Ne pas activer un appareil ni lancer un test physique au seul démarrage d’une tâche.
- Énergie : distinguer W, VA et Wh/kWh, index et périodes ; total et sous-circuits ne
  s’additionnent pas. Une moyenne de variations d’index reste une estimation, jamais une mesure instantanée.
- Saisie des comptes et codes par le propriétaire dans l’interface ; aucun secret dans
  Git, les captures ou les journaux. Ne jamais consulter de clé privée ou de coffre personnel.

## Plusieurs conversations Codex et entretien documentaire

- Les conversations de ce projet sont séquentielles : réutiliser le checkout permanent `moli-os` et sa branche active. Ne pas créer de clone ou worktree pour une conversation seule.
- Créer une branche ou un worktree séparé seulement si le propriétaire demande un travail
  simultané. Ne pas partager un checkout modifié, un port de développement ou des données de test.
  Examiner aussi les instructions privées applicables avant de préparer un environnement.
- Avant de modifier un fichier, vérifier qu’il n’a pas changé depuis sa lecture.
  Ne jamais utiliser reset/clean/stash pour effacer le travail d’une autre conversation.
- Une seule opération de déploiement ou de migration à la fois ; vérifier à nouveau
  l’image et la configuration actives avant tout remplacement. Les autorisations d’un ancien chat
  ne dispensent pas de lire la demande et les contraintes de la tâche actuelle.
- `AGENTS.md` contient les règles durables ; `PROJECT_STATUS.md` contient l’état courant
  daté et les tâches restantes. Les contrats techniques restent dans `docs/` et `integrations/`.
  Les adresses, comptes, configurations et preuves privées restent dans `../moli-maison/`.
- Après une étape importante, le skill personnel **fonctionnalite-terminee**, s’il est
  disponible, peut vérifier le périmètre et actualiser `PROJECT_STATUS.md`. Il ne clôture pas le projet.
- **projet-termine** est réservé à une demande explicite de clôture définitive.
  Les skills personnels existent hors du dépôt ; ne pas les recréer ni les recopier ici.
- Pour ce lot documentaire : aucun commit, push ou déploiement sans accord explicite.
  L’actualisation de la documentation et l’usage d’un skill ne constituent pas cet accord.

# Version personnelle : démarrage, validations et exploitation

Le cœur Rust, le dashboard et SQLite restent ceux de Moli OS. Les modifications du fork
sont isolées sur `personal/main` ; `main` demeure le miroir de l'amont.

## Ce qui est livré

- Manifeste et icônes pour l'écran d'accueil, sans cache hors ligne ni service worker.
- Navigation optionnelle `home.json.navigation` (liste des routes sans `#/`) ; sans cette
  liste, la navigation amont reste identique. Système et Atelier restent accessibles.
- Cartes volets : commandes désactivées hors connexion/hors ligne/en lecture seule ; cibles
  tactiles de 44 pixels. Date de réception cloud distincte de la date de mesure.
- Pilote expérimental `overkiz` : session historique Somfy Europe, lecture seule par défaut.
  Voir `integrations/overkiz/README.md` pour ses limites, notamment OAuth non implémenté.
- Bambu : option `read_only = true` à la fois dans les points et au dernier accès MQTT.
- Démo synthétique, configuration de production sans pilote actif et Compose indépendant.
- Images de construction Node/Rust épinglées par version et digest multiarchitecture.
  Construction amd64 générique, sans `target-cpu=native`, deux tâches de compilation.

Les informations de la maison appartiennent au dossier privé voisin `moli-maison`, jamais
à ce dépôt. Aucun historique TaHoma, transcodage, pilote Xiaomi inconnu ou application native
n'a été ajouté sans validation du matériel. Les étapes suivantes restent des jalons réels,
pas des compatibilités déclarées.

## Démo locale sans matériel

Rust et Node doivent être déjà disponibles, ou utiliser l'image de développement amont.
Le mode natif ci-dessous est un repli pour un Mac sans Docker ; la CI Linux reste nécessaire.

```sh
npm --prefix web ci --no-audit --no-fund
npm --prefix web run build
python3 scripts/prepare-personal-dev.py --self-test
python3 scripts/prepare-personal-dev.py
cargo build --locked -p moli-os -j 2
MOLI_UI_PIN=000000 target/debug/moli-os --config data/development/moli.toml serve
```

Ouvrir `http://127.0.0.1:8791`. `000000` est uniquement le code public d'une démo synthétique,
jamais un secret de production. Le générateur refuse d'écraser une configuration existante.
Tous les appareils sont fictifs ; ni mot de passe réel, ni accès cloud, ni assistant IA.
Les données sous `data/` sont exclues de Git.

## Contrôles

```sh
sh scripts/check.sh
npm --prefix web run build
npm --prefix web test
node scripts/i18n-check.mjs
node scripts/pwa-check.mjs
python3 scripts/prepare-personal-dev.py --self-test
```

Le contrôle Rust amont complet s'exécute normalement dans Linux Docker/CI. Certains tests
amont de `moli-profile` et `moli-tuya` utilisent plusieurs adresses 127.0.0.x : macOS ne les
fournit pas comme Linux. Ne pas ajouter des alias système pour rendre les tests verts.
Un résultat Mac excluant ces suites n'est pas une réussite du contrôle complet Linux.
Les tests HTTP factices nécessitent le droit d'écouter sur localhost dans le bac à sable.

## Image et stack de test NAS

Le workflow `image` construit amd64 et arm64 sur runners natifs. Il publie dans le registre
du fork ; choisir le tag de commit et conserver son digest. Aucun build sur le NAS.
La réussite du build ne prouve pas l'exécution sur un ancien CPU ou noyau.

`deploy/personal/compose.yaml` est une stack autonome compatible avec la syntaxe Compose 2.4.
Variables requises : `MOLI_IMAGE`, `MOLI_DATA_DIR`, `MOLI_KEY_FILE`, `MOLI_PIN_FILE`.
`MOLI_BIND_IP` vaut 127.0.0.1 par défaut ; mettre l'adresse LAN seulement après contrôle des
ports et du réseau. Aucun port Internet, aucun socket Docker monté dans Moli.
Les fichiers de secrets doivent exister, être lisibles par UID 1000 et montés en lecture seule.
La syntaxe courte des volumes impose de vérifier l'existence avant déploiement : certaines
versions de Compose créent un dossier lorsque le fichier source manque.

Créer une stack dédiée `moli-personal`, avec un dossier de données neuf ; ne réutiliser ni
réseau ni volume d'une autre application. Le modèle `moli.example.toml` ne démarre aucun
pilote physique. Plafond : 128 Mio, pas de swap supplémentaire, 0,5 cœur, logs 3 × 10 Mo.
Le système de fichiers racine est en lecture seule ; les données persistent dans `/data`.

Avant mise en production :

1. Contrôles CI Linux complets ; démarrage amd64 sur le N54L sans instruction illégale.
2. Santé, TLS cloud, SQLite, permissions et arrêt gracieux validés sur le noyau réel.
3. Surveillance 24 h ; comparaison CPU/RAM/disque et services existants avant/après.
4. Sauvegarde et restauration dans une destination isolée.
5. Recette Safari Mac puis iPhone réel (tactile, rotation, retour de veille, réseau coupé).
6. Secret maître récupérable après redémarrage complet du NAS. Un tmpfs seul ne suffit pas.
7. HTTPS local après validation des sessions derrière Caddy. Ne pas confondre la garde Host
   et une authentification ; ne pas faire apparaître tous les clients comme une même machine.

## Somfy et équipements suivants

Le pilote `overkiz` est inactif sans configuration et sans identifiants dans le coffre.
L'humain saisit ses secrets directement ; ni code de passerelle ni mot de passe dans le chat.
Ne pas répéter un échec de login rapidement : le serveur peut limiter ou bloquer le compte.
Comparer les états 48 h. Ensuite seulement : volet neutre, personne présente, retour à la
position de départ. Un acquittement cloud n'est jamais une confirmation de mouvement.

Thermostat et énergie : relever les définitions réelles avant d'associer unités et commandes.
Les valeurs brutes ne deviennent pas automatiquement des compteurs. Garder la distinction
W / VA / Wh / kWh, les resets, l'ancienneté et les périodes manquantes. Le total comprend les
circuits ; les appareils dans ces circuits n'ajoutent pas une troisième consommation.

Caméras : étape suivante, go2rtc distinct et à la demande, H.264 sans encodeur vidéo permanent.
Le relais de signalisation/HLS et son contrôle d'accès ne sont pas encore implémentés.
Ne jamais mettre des identifiants RTSP dans `home.json` ou le navigateur. La caméra A1 mini
reste une étude séparée. Pas de changement de firmware ou de mode Bambu automatique.

Tablette/Shield : réutiliser l'API web ; validation du WebView, du focus télécommande et du
lecteur vidéo avant enveloppe native. Détection caméra tablette locale à faible cadence,
sans calcul de vision sur le serveur ; aucun réveil écran éteint promis sans essai réel.

## Sauvegardes et retour arrière

Utiliser `moli-os backup <destination>` pour obtenir des bases cohérentes, pas une copie
naïve des fichiers SQLite en cours d'écriture. Le manifeste amont contrôle noms et tailles,
pas une empreinte cryptographique de chaque fichier. Vérifier aussi l'intégrité SQLite.
Conserver la clé maître séparément et chiffrer la destination de sauvegarde.

Politique à activer après recette NAS : 7 quotidiennes, 4 hebdomadaires, copie hors NAS.
Le stockage distant et le planificateur ne sont pas configurés automatiquement.
Avant chaque mise à jour : sauvegarde, digest courant et image précédente conservés.
Retour arrière : arrêter seulement Moli, remettre l'image précédente et, en cas de migration,
la sauvegarde compatible. Ne jamais restaurer sur une instance encore active.

## Git

```sh
git fetch upstream
git switch main
git merge --ff-only upstream/main
git switch -c sync/upstream personal/main
git merge main
# Contrôles et revue, puis fusion dans personal/main.
```

Aucun rebase d'une version déployée. Les données privées ne sont jamais ajoutées à l'index.
Les correctifs génériques peuvent être proposés à l'amont après revue des fuites.

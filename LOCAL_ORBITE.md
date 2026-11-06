# Orbite — version locale

Cette branche reprend les pages et commandes Moli OS existantes avec l’habillage
Orbite validé : thèmes jour/nuit, navigation basse, menu Plus, accents de couleur,
synthèse énergétique avec les vrais consommateurs du rapport et formats adaptatifs.
Le graphique de détail conserve les barres empilées et les calculs existants.

## Démarrer

Depuis la racine du dépôt :

```sh
./scripts/orbite-local.sh
```

Ouvrir <http://127.0.0.1:8791>. Arrêter avec Ctrl+C dans le terminal du serveur.
Node/npm, Cargo et les dépendances déjà présentes sont nécessaires. Le script ne
les installe pas. Cargo utilise son cache local, sans télécharger de dépendance.

## Données et commandes

La configuration `demo/orbite/moli.toml` n’active que les pilotes de démonstration
et les aides locales. La villa fournie par le projet simule les appareils, les
caméras, les changements d’état et l’historique. Les commandes ne vont pas à la maison.
Les fonctions sans appareil configuré gardent leur état indisponible ou restent
masquées, plutôt que d’afficher une donnée inventée.

Le serveur écoute uniquement sur `127.0.0.1:8791`. Ses données, son historique et sa
clé de démonstration vivent sous `.local/orbite-data`, ignoré par Git. Comme la villa
d’origine, la démonstration est réinitialisée à chaque démarrage ; ne pas y importer
de données personnelles. Les variables de clé de la maison sont retirées du processus
local avant le démarrage.

## Navigation

Les flèches déplacent le focus entre les commandes visibles. Entrée active la
commande native ; Échap ferme le menu Plus. Les champs, curseurs et dialogues
conservent leurs interactions et protections. Le menu Plus donne accès aux pages
existantes, dont Pièces, Plan, Caméras, Automatismes, Réseau, Système et Paramètres.
Le sélecteur Affichage propose automatique, ordinateur, tablette, téléphone et TV.
La recette avec une télécommande physique reste nécessaire.

## Vérifier

```sh
npm test --prefix web
npm run build --prefix web
node scripts/i18n-check.mjs
node scripts/pwa-check.mjs
```

Aucun déploiement NAS, changement de configuration de production, commit ou push
n’est effectué par le script. Le raccordement à la maison réelle fera l’objet d’une
étape distincte après recette locale.

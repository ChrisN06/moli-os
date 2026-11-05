# Le dashboard (`web/`)

Svelte 5 (runes) + Vite, **embarqué dans le binaire** (`rust-embed`, ~210 Ko JS / 66 Ko CSS,
compressé à l'envoi). Deux visages sur le même hub, routés par le hash (`web/src/App.svelte`) :

| Visage | Route | Pour qui |
|---|---|---|
| **Maison** | `#/…` (défaut) | la famille : un membre de la famille qui n'est pas technicien doit pouvoir s'en servir sans aide |
| **Atelier** | `#/atelier` | bricoler : tous les appareils, toutes les valeurs, journal, approbations, bench |

Le flux SSE `/api/events` est ouvert **une seule fois** et partagé (`web/src/lib/hub.svelte.js`).

## Maison : pages (`web/src/maison/pages/`)

| Page | Route | Contenu |
|---|---|---|
| Accueil | `#/` | date, salut, heure, météo ; alertes seulement quand il y a un souci (fumée, eau, portes, sonnette, piscine, approbations) ; lumières favorites, et dessous **Ambiance** : le choix de la pièce (pastilles, la dernière choisie est retenue sur l'appareil) et le bandeau d'ambiances ; électricité (jauge du jour vs hier, et à côté la jauge **en direct** du Linky sur l'abonnement, `[energy.live]` : sa couleur dit combien on tire, pas la distance à l'abonnement, verte au calme, orange dès `warn` (par exemple 2 kVA), rouge au double ; mot « calme / modérée / soutenue / forte ») ; clims (cadran **tactile** : on fait glisser le doigt sur l'anneau pour la consigne, le point gris = la température de la pièce, la perle = la consigne, visibles même clim éteinte ; une consigne glissée clim éteinte part tout de suite et la clim démarre dessus ; marche/arrêt, consigne ± 0,5 °C regroupée en un seul ordre, mode, dès que le profil sait écrire) ; sonnette (image + dernier passage réel, lu dans l'historique) ; télécommande ; dehors en bref **Mise à jour du 7 oct.** : une tuile de lumière peut en commander plusieurs (`favorites.lights[].also` : par exemple une pièce et son couloir) ; Électricité : les plus gros consommateurs du moment en légende ; Climatisation : coût du jour et du mois par clim (compteur de son circuit, sinon « coût non mesuré ») ; Dehors : l'humidité à la place de la couleur Tempo. |
| Moli | `#/moli` | l'assistant plein écran : la conversation à gauche, **l'écran se compose** à droite avec les cartes choisies par Moli (voir [moli.md](moli.md)) |
| Salon | `#/salon` | Éclairage et ambiances, température/humidité intérieures, volets des pièces Salon et Cuisine (liste personnalisable par `salon.cover_rooms`). |
| Pièces | `#/pieces` | une carte par pièce : groupe Hue, lampes (curseur de luminosité ; lampe couleur : pastille qui ouvre une palette, trois blancs par température et huit couleurs + « autre couleur » ; en mode couleur l'ampoule de la tuile prend la teinte ; la tuile « Toute la pièce » a la même palette, appliquée à chaque lampe de la pièce qui sait la prendre) ; bouton **Ambiances** : un bandeau (Feu de cheminée, Bougies, Fin de journée, Cocooning, Fête, Océan, Forêt, Aurore boréale, Romantique, Cinéma, Lecture, Veilleuse), aussi sur la page Salon, volets, clims, prises, médias, capteurs en pastilles ; « Protégée » sur les chambres ; « Tout éteindre » **Depuis le 7 oct., cartes compactes** (`ui/RoomCard.svelte`) : en-tête = température de la pièce (moyenne de ses sondes et clims, jauge froid → chaud), ambiances (pastille), petit interrupteur de toutes ses lumières (jamais télé ni prises) ; lumières, télé, enceinte, prises en petites pastilles côte à côte, icônes parlantes (suspension, spot, lampe, ventilateur, prise…), ⚙ pour les réglages d'une lampe (luminosité, couleurs, ampoule par ampoule, ventilateur) ; une clim sur une ligne (le cadran au toucher) ; affichage « Aa » (noms) ou icônes seules, retenu par appareil (`pieces-look`). |
| Lumières (D12) | partout | une lampe injoignable est « Coupée à l'interrupteur » (jamais comptée allumée, pas de halo ; un appui l'explique) ; une pièce Hue est « Toute la pièce », jamais une lampe ni une pastille du plan ; un **luminaire** (`lumieres:…`) remplace ses ampoules et son relais dans les vues de la famille, avec un bouton « ampoule par ampoule » (une couleur chacune) ; « Tout éteindre » une pièce = ses lampes et luminaires une à une, jamais la télé, l'enceinte ni les prises ; les ambiances visent les ampoules d'un luminaire. `home.svelte.js` : `isOn` (injoignable = éteint, groupe = une de ses lampes joignables), `isFixture`, `isGroup`, `fixtureOf`, `unpowered` |
| Plan | `#/plan` | **le plan vivant** : un onglet par étage. Étage **dessiné** (`plan/Drawing.svelte`) : la maison redessinée en SVG depuis la donnée, zoom molette / pincement / double tap / boutons, 4 habillages (Plan, Blueprint, Nuit, Aquarelle ; choix par appareil, défaut de la maison en modification), chaque lampe allumée projette un halo découpé par sa pièce, les pièces se teintent ; bouton **3D** (`plan/House3D.svelte` + `plan/scene3d.js`, three.js chargé à la demande) : étages empilés, murs coupés à 60 % sur l'étage regardé, meubles modélisés (`plan/furniture3d.js` : lits, canapés, chaises, voitures, trampoline, transat…), murs de biais et murets, toit de tuiles sur l'étage qui en a un (le regarder = la maison de dehors), vraies lumières, toucher une perle = l'appareil. En modification d'un étage dessiné : ajouter / déplacer / tourner / redimensionner un élément (lit, canapé, voiture…), « poser les appareils dans leur pièce ». Étage **image** (ancien mode) : l'image du plan, une lumière allumée brille, une porte ouverte ou une présence se colore, une fumée ou une fuite clignote ; toucher = allumer/éteindre (lumières, prises), caméras et télé ouvrent leur page, le reste ouvre une fiche. « Modifier » : ajouter un étage avec son image (photo du plan acceptée), poser un appareil (le choisir dans la liste, toucher le plan), glisser pour déplacer ; enregistrer demande le code. Données : `data/plan.json`, images dans `data/plan/` (voir [api.md](api.md)). |
| Caméras | `#/cameras` | mur vidéo automatique et muet pour les caméras vidéo ; images rafraîchies si visibles pour les autres, batterie au toucher seulement ; plein écran avec les passages du jour |
| Énergie | `#/energie` | en ce moment (Linky en direct coloré selon la conso, repère à `warn`, prochain changement heures creuses / pleines), aujourd'hui (au rythme d'hier), mois + projection vs mois dernier + part en heures creuses, Tempo ; **conseils du moment** (`lib/energy-advice.js` : porte ouverte clim allumée, grosse conso en heures pleines et qui tire, clim alors qu'il fait plus frais dehors, consignes, pompe de piscine en heures pleines, heures creuses en cours ou proches, talon (heure la plus calme de chaque jour, médiane sur 7 jours) en €/an) ; qui consomme maintenant ; où ça part (aujourd'hui / hier / ce mois, par circuit, %) ; **journée type** (moyenne heure par heure sur 7 jours, heures creuses **apprises** de l'historique du point PTEC, talon) ; graphe 24 h / 7 j / 30 j / 12 mois. Mobile : une colonne, chiffres deux par deux **Journée type** : la journée en cours par-dessus (repère sur chaque heure passée, « aujourd'hui jusqu'à X h : … kWh, d'habitude … (± %) ») ; lumières estimées « ≈ estimé » dans « qui consomme » et « où ça part ». |
| Impression 3D | `#/impression` | une carte par imprimante (Klipper, Bambu) : état, vignette du travail, progression, couche, temps restant et heure de fin, buse / plateau / enceinte, têtes et filaments (couleurs ; AMS pour Bambu), lumière, **pause / reprise / annulation** (annuler demande une confirmation) ; totaux (impressions, heures, km de filament). Sur l'Accueil, une pastille tant qu'une impression tourne, est en pause ou en erreur ; dans Pièces, une pastille sur la carte de la pièce |
| Dehors | `#/dehors` | météo (soleil calculé), air/UV, piscine (pH, chlore, filtration), jardin, caméras extérieures |
| Automatismes | `#/automatismes`, `#/automatismes/<id>` | liste, idées, éditeur à canevas (voir [automatismes.md](automatismes.md)) |
| Système | `#/systeme` | **carte vivante** du système et comparaison avec HA (voir plus bas) |

Navigation : rail à gauche (bas de l'écran sur téléphone, défilable) ; pied de rail : thème,
Système, Atelier. Bulle « Demander à Moli » sur toutes les pages sauf Moli.

## La mise en page de la maison : `data/home.json`

Servie telle quelle par `/api/home` depuis le dossier de données (`data/home.json`, écrite à
la main ou par un agent pour chaque maison). Sans elle, le dashboard reste utilisable (pièces
telles que les appareils les nomment).

```json
{
  "title": "Maison",
  "rooms": [{ "name": "Chambre des enfants", "icon": "bed-outline", "aliases": ["Chambre 2"] }],
  "favorites": {
    "lights":  [{ "id": "hue:…", "name": "Salon", "icon": "sofa-outline" }],
    "climate": [{ "id": "clim-salon:…", "name": "Salon" }],
    "cameras": [{ "id": "reolink:…", "name": "Sonnette", "refresh": 10 }, { "id": "reolink:…", "name": "Allée", "refresh": 60, "battery": true }]
  },
  "salon":   { "tv": "tele:…", "speaker": "sonos:…", "room_light": "hue:…", "lamps": ["hue:…"] },
  "doors":   [{ "id": "tuya:…", "name": "Entrée" }],
  "outdoor": { "weather": "meteo:…", "air": "air:…", "tempo": "tempo:…", "pool": "piscine:…", "pool_pump": "tuya:…", "garden_lights": [{ "id": "tuya:…", "name": "Lumière du jardin" }] },
  "safety":  ["z2m:…"],
  "hidden":  ["tuya:…", "compteur:…"],
  "quiet":   ["tuya:…"],
  "outlets": { "tuya:…": { "switch_1": "Prise 1", "switch_2": "Prise 2", "switch_3": "Prise 3" } }
}
```

- `rooms[].aliases` : les noms que HA, Hue ou Tuya donnent à la même pièce (le dashboard et
  Moli les ramènent au nom canonique).
- `hidden` : masqué du dashboard **et** de l'inventaire de la conversation (pas de celui
  qui sert à créer les automatismes). Par exemple un compteur EM06 (sinon Moli montre une fiche
  « em06p » brute).
- `quiet` : rangé dans sa pièce mais **jamais un bouton** (lave-vaisselle, machine à laver,
  recharge de la voiture : un appui de travers coupe un cycle). La carte de la pièce dit ce qu'il
  consomme (« Lave-vaisselle · 1 850 W », dès 3 W) ; débranché, il ne dit rien (sauf dans une
  vue filtrée). Moli et les automatismes peuvent toujours le commander.
- `outlets` : une multiprise dont chaque prise a son bouton, dans l'ordre donné, plus un bouton
  pour toute la multiprise (« 2 sur 3 » ; un appui allume tout, ou éteint tout si tout est
  allumé). Seuls les points inscriptibles comptent.

## Conception visuelle

- Thème **porcelaine** le jour, **nuit** après le coucher du soleil (point `daylight` de la
  météo) ; bascule Auto / Jour / Nuit (mémorisée dans le navigateur).
- Ambre = allumé, bleu = climat/froid, corail = attention. Grandes cibles tactiles, peu de mots,
  aucun jargon. Police Figtree Variable, icônes Material Design (sélection dans
  `lib/icons.js` : seules celles-ci sont dans le paquet).
- Jetons CSS sous `.maison` dans `maison.css` (l'Atelier garde son propre style).
- Règle de style : **pas de bordure gauche colorée** sur les cartes.
- Une lampe injoignable n'est jamais affichée allumée (sa dernière valeur est une supposition).
- Les ordres retenus par la garde ouvrent la feuille « C'est moi » (code PIN), qui sert aussi
  à valider, lancer ou supprimer un automatisme.

## Bibliothèques partagées (`web/src/maison/lib/`)

- `home.svelte.js` : état de la maison (config, ordres retenus, toasts, thème), lecture des
  valeurs, `powerKey` (le point qui allume/éteint), `deviceKind`, `roomList`, `act` (envoie
  un ordre, gère le 202 de la garde), `confirmHeld` (code PIN), formats français.
- `moli.svelte.js` : la conversation (partagée bulle/page), voix (enregistrement si contexte
  sécurisé, lecture des réponses à voix haute), approbations depuis la conversation,
  suggestions selon l'heure et la maison.
- `auto.svelte.js` : catalogue des nœuds, textes des nœuds, mise en page automatique, API des
  automatismes (avec empreintes), `asHuman`.
- `energy-live.svelte.js` : résumé énergie partagé (une seule requête par minute quel que soit
  le nombre de cartes).

## La carte vivante « Comment ça marche » (`pages/Systeme.svelte`)

Construite **uniquement** à partir de `/api/system` et du flux d'événements : pilotes (état,
appareils, injoignables) → cœur (appareils, valeurs, changements par minute) → briques
(actives ou non, chiffres clés) → surfaces. Les liens d'un pilote s'allument quand ses
appareils changent. En tête : Moli contre HA (mémoire, processeur, taille, réponse, budget,
tendance des livraisons) lu dans `data/perf.jsonl`. Un nouveau pilote ou une nouvelle brique y
apparaît sans toucher à la page.

## Vérifier visuellement

Pour un agent, plutôt que son navigateur intégré (chaque clic y demande une autorisation) :
`puppeteer-core` sur le Chrome installé, avec de petits scripts de capture écrits pour la
session ; `chrome --screenshot` ne rend jamais la main à cause du flux SSE.

## Limites connues

- **Voix** : le micro exige HTTPS ; en `http://<hôte>:8790` le bouton est grisé.
- Télé : applis et clavier non disponibles (JointSpace ne le permet pas ; protocole
  Android TV Remote v2 à faire).
- Clims : lecture seule tant que les écritures Daikin n'ont pas été testées physiquement.

### Personnalisation des surfaces

La navigation Pièces utilise une icône de plan de maison (Material Design Icons).
La section géographique Carte et Leaflet sont retirés ; les données de présence,
les personnes et les composants partagés avec la météo restent conservés.
Atelier garde ses cartes et commandes, avec retour explicite à l’accueil, recherche
multi-mots et filtre de disponibilité. Le journal reste accessible et repliable.
Les imprimantes réelles ont une synthèse état/progression/températures/temps
restant et leurs emplacements AMS reçus ; les champs inconnus restent absents ou « — ».

## Estimation, air et volets

Sans point de puissance, Accueil et Énergie peuvent montrer la moyenne entre les deux
changements récents du compteur total (`CounterPower`, `/api/history`, Wh ou kWh).
Les heures énergétiques importées ne servent jamais à ce calcul. Durée et ancienneté sont
visibles ; au-delà de 90 minutes sans changement, l'estimation est signalée ancienne.
Réinitialisation, saut incohérent, agrégation ou intervalle supérieur à 24 heures : aucune
estimation. La protection utilise le plafond configuré du collecteur (36 kW par défaut).

Dehors réutilise le profil `open-meteo-air` pour les données régionales CAMS, créditées
à Open-Meteo et CAMS ENSEMBLE. `outdoor.air` désigne son appareil ; `outdoor.sun` désigne le
capteur d'éclairement (`illuminance`, lx). Sa courbe utilise l'historique local et se
rafraîchit chaque minute. Lux et UV ne sont pas des mesures de puissance solaire.

Les volets ont une représentation tactile de l'ouverture : 0 % fermé, 100 % ouvert.
Le geste prépare une cible puis envoie un ordre unique après relâchement (clavier regroupé
sur 300 ms). La position rapportée reste distincte de la cible et ne change que sur retour
réel du pilote. Contrôles désactivés hors ligne, en lecture seule ou sans capacité annoncée.
La garde et les confirmations existantes sont conservées.

### Caméras Tapo

Le formulaire **Comptes caméra Tapo** de `#/parametres` liste les caméras déclarées
par l’installation. Le propriétaire saisit lui-même leur compte caméra local,
distinct du compte cloud Tapo. Après vérification, le formulaire efface les champs ;
le coffre chiffré conserve le compte. Une connexion échouée ne remplace pas le
compte existant.

Les cartes vidéo s’ouvrent avec **Voir en direct**. Safari utilise HLS natif ;
Chrome et les autres navigateurs MSE chargent hls.js light à la demande. La vidéo
est muette, sur le sous-flux H.264, sans enregistrement ni transcodage. Fermer la
vue ou masquer l’onglet arrête le lecteur ; le relais libère la source après son
délai de cinq secondes sans segment demandé. Les caméras à instantanés existantes
conservent leur comportement.

## Réglages, vidéo et température intérieure

`#/parametres`, accessible dans le rail et le pied mobile, rassemble les comptes caméra
Tapo et le tarif fixe. Les formulaires conservent leur protection propriétaire.

Les flux vidéo du mur Caméras démarrent muets dès l'entrée et au retour de l'onglet.
Ils s'arrêtent à la sortie, en arrière-plan ou quand le plein écran prend le relais.
Le bouton Arrêter suspend un flux jusqu'à une reprise manuelle ou au retour de l'onglet.
Les autres pages ne lancent pas automatiquement les caméras ; les instantanés et caméras
sur batterie conservent leur comportement existant.

L'Accueil montre température et humidité intérieures : une sonde unique par grandeur,
hors appareils explicitement déclarés dans `outdoor`. Plusieurs sondes : aucun choix
arbitraire, valeur inconnue ; `home.json` peut alors sélectionner les points :
`indoor.temperature = "sonde:piece/temperature"`, `indoor.humidity = "sonde:piece/humidity"`.
Une sonde hors ligne n'affiche pas sa dernière valeur comme température actuelle.

Journée type : les points représentent les relevés horaires d'aujourd'hui, les barres
la moyenne. Toucher ou glisser horizontalement sélectionne une tranche sans sélection
de texte ni menu d'appui long ; le défilement vertical reste autorisé. Même sélection
tactile sur l'historique. Les boutons de la journée type restent accessibles au clavier.

### Éclairage en direct et favori personnel

Les curseurs de luminosité, blanc et saturation envoient pendant le geste. Une
seule commande par lampe reste en vol ; les valeurs intermédiaires sont remplacées
par la plus récente, avec 400 ms entre les réponses et le prochain envoi. Quitter
la page annule les valeurs encore en attente. Les erreurs et demandes du garde
vident la file, sans répétition tardive. Passer l’application en arrière-plan vide
aussi la file ; un geste en attente depuis plus de deux secondes est abandonné.

**Perso**, première ambiance, rappelle le favori enregistré pour les lampes de la
pièce. **Mémoriser** capture les derniers réglages connus du serveur, après les
commandes en cours. La sauvegarde exige le propriétaire ; le rappel conserve les
protections et les limites des invités. Le favori est partagé entre appareils et
inclus dans la sauvegarde. Une lampe mémorisée éteinte reçoit seulement l'arrêt,
car régler ses paramètres pourrait la faire clignoter. Les changements faits dans
une application externe restent inconnus lorsqu'un pont ne fournit aucun retour.

### Xiaomi Home

**Paramètres** accueille la connexion Xiaomi et le captcha/second facteur.
Les commandes sont disponibles après vérification du compte et du modèle par le pilote,
sans option de déplacement supplémentaire. **Aspirateur** lit sa télémétrie,
affiche la carte sur demande et mémorise un rectangle cuisine confirmé. Le bouton
**Démarrer le nettoyage** lance un nettoyage classique indépendant de cette carte. Aucun
nettoyage ne démarre lors de la connexion ni de l'enregistrement du rectangle.
La caméra Xiaomi réutilise la page caméras et son démarrage à l'ouverture.
Voir `integrations/xiaomi-vacuum/` et `integrations/xiaomi-camera/` pour les limites
cloud, HEVC, firmware et stabilité du repère.

### Réseau Freebox et habillage bleu nuit

`#/reseau` reprend la topologie Freebox : branches des ports et du Wi-Fi Pop,
répéteurs rattachés à leur port Ethernet, appareils, signal, bande et vitesses
PHY. Les flux verts descendants et rouges montants restent animés entre les
relevés de cinq secondes ; débit, largeur et densité sont interpolés sans changer
l'amplitude d'ondulation. Les éléments gardent une identité MAC lors des
changements de point d'accès. Un second clic sur le même port ferme son panneau.

Le rendu est chargé à la visite et isolé dans un Shadow DOM pour ne pas modifier
les interactions des autres pages. Quitter la page libère le canevas et son
animation ; les lectures partagées s'arrêtent quand aucune vue ne les utilise,
ou quand l'onglet est masqué. Une erreur efface les valeurs courantes et indique
l'indisponibilité, sans afficher une ancienne mesure comme actuelle.

L'Accueil conserve seulement les deux valeurs de trafic Freebox, téléchargement
et envoi, avec un lien vers Réseau. Paramètres propose l'appairage propriétaire ;
aucun secret n'est saisi ou retourné dans le navigateur. `night.css` partage le
fond bleu nuit, les surfaces, couleurs, bordures et rayons avec Maison et Atelier.
Les schémas et preuves de développement utilisent des appareils fictifs.

## Ajustements des pages — novembre 2026

L’accueil utilise une grille responsive sans fragmentation multicolonne. Les éclairages
et ambiances restent dans Salon et Pièces ; les raccourcis Salon et Dehors sont retirés.
Pièces masque la pièce nommée Électricité, sans modifier ses appareils ni leur historique.
Le Salon reprend les références `indoor` ou les sondes intérieures uniques, et les volets
attribués aux pièces `salon.cover_rooms` (par défaut Salon et Cuisine).

Chaque montage d’une tuile de volet demande l’actualisation des états Overkiz. La requête
est regroupée par instance et limitée à une toutes les 30 secondes, côté navigateur et
pilote. Le point `refresh` reste soumis aux droits et à la garde ; il ne lance aucun
mouvement. Le retour arrive par les événements cloud, sans état de position optimiste.

Les flux vidéo n’ont plus de bandeau de commande. Cliquer ouvre le lecteur ; un clic
dans le lecteur révèle les commandes natives. Les proportions natives sont conservées.
Paramètres propose une rotation 0/90/180/270°, mémorisée dans ce navigateur uniquement.
Les flux s’arrêtent hors écran et lorsque l’onglet est masqué.

La journée type sélectionne l’heure courante en l’absence d’interaction. L’éclairement
utilise les bornes lever moins une heure / coucher plus une heure, lorsque disponibles ;
il conserve la fenêtre de 24 heures en leur absence. Aucun point futur n’est inventé.

Mi-Light conserve les valeurs déjà restaurées par le cache serveur au lieu de les effacer
au redémarrage du pilote. Elles restent des états supposés, sans retour physique des lampes.
Une commande incertaine continue d’invalider les valeurs concernées.

## Courbes intérieures et eau

La page Salon superpose l'historique local de température, d'humidité (axe séparé
0–100 %) et de `reported_setpoint` (paliers), sur 24 h, 7 ou 30 jours. Les choix
`indoor.temperature` et `indoor.humidity` servent aussi aux courbes ; sans choix,
seul un capteur intérieur unique est retenu. Une consigne ambiguë reste absente.
Il s'agit de la consigne rapportée par Overkiz, pas d'une garantie sur une dérogation
active dans l'application du fabricant. La sélection temporelle fonctionne au
clavier et au toucher. Aucun ancien historique Somfy n'est téléchargé.

L'eau utilise `GET /api/water` et un import propriétaire `PUT /api/water`
(`csv`, `unit` valant `m3` ou `L`). Paramètres accepte l'export **journalier** de
Tout sur mon eau : `Date;Consommation;Index`, séparateur point-virgule, avec ou sans
guillemets et BOM. L'unité doit correspondre à celle choisie sur le site. Un import
mensuel, une date invalide, un doublon, un nombre négatif/non fini ou un fichier
trop volumineux est refusé sans écriture partielle. Les jours sans index (exportés
avec index et volume nuls) ne remplacent pas une mesure par zéro. Un zéro avec
index valide reste une consommation mesurée de zéro.

Les imports d'un seul compteur fusionnent par date, dans la limite de 3 660 jours.
Les valeurs sont stockées dans le coffre chiffré existant, inclus dans les
sauvegardes natives. Aucun index ni historique électrique n'est modifié.
L'accueil montre la somme disponible pour le mois civil courant ; Énergie affiche
les jours du mois choisi, avec les dates manquantes explicitement distinctes des
zéros. La dernière date mesurée est indiquée.

La connexion So’EAU dans Paramètres utilise le formulaire HTTPS normal (CSRF et
cookies limités à cet hôte), puis `meters-list` et `telemetry`, directement en Rust
via `moli-net`. Aucun navigateur automatisé, Home Assistant ou MQTT n'est requis.
Les identifiants ne sont conservés qu'après une récupération réussie, dans le coffre
chiffré natif. Un compte avec plusieurs compteurs exige de choisir un identifiant
qui lui appartient. Le compteur lié à l'historique ne peut pas changer silencieusement.
Les trois derniers mois sont lus à chaque synchronisation, au démarrage puis toutes
les 12 heures ; une lecture manuelle est disponible, avec une minute entre tentatives.
Les erreurs conservent les relevés existants et sont affichées sans données de connexion.
Un contrôle humain imposé par le fournisseur reste un échec explicite, sans contournement.
Le CSV reste utilisable. Référence de protocole :
[py-mon-eau](https://github.com/laurent-martin/py-mon-eau).

`GET /api/water/account`, `POST /api/water/account` (`email`, `password`, `meter`
facultatif) et `POST /api/water/sync` (`{}`) exigent une session propriétaire.
Aucune réponse ne retourne les identifiants ou cookies. `GET /api/water` ajoute
`synchronized_at` et `sync` (`configured`, `error`) aux relevés ; l'identifiant du
compteur reste dans le coffre. Les routes d'écriture partagent la limite eau de
cinq requêtes par dix secondes. Aucun ancien historique électrique n'est modifié.

Les coordonnées précises de la zone aspirateur sont affichées en mètres depuis
la base orange `(0, 0)`. La conversion vers les coordonnées absolues du protocole
reste interne ; le mouvement du robot violet n'entre jamais dans le recalage.

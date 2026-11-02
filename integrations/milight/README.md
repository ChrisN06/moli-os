# Mi-Light RGBW, pont UDP v6

Pilote natif `moli-milight`, sans dépendance réseau supplémentaire. Un appareil par
zone déclarée (1 à 4), aucune commande globale, aucun appairage automatique.
Marche/arrêt, luminosité 0–100 %, couleur RGB (teinte seulement) et blanc dédié
(`#ffffff`). Le mode RGBW simple ne permet pas de régler la température du blanc ni la saturation.
`rgbcct = true` sélectionne les commandes v6 RGB+CCT (famille 0x08) : blanc
2700–6500 K et saturation. Le type doit être confirmé pour les zones configurées.
Le nom du pont ne suffit pas à identifier ses ampoules.

Références de protocole :
- [Commandes RGBW v6](https://github.com/mwittig/node-milight-promise/blob/master/src/commandsV6.js)
- [Session, séquence et checksum](https://github.com/mwittig/node-milight-promise/blob/master/src/milight-v6-mixin.js)

Implémentation indépendante : en-têtes, somme modulo 256 et commandes sont les
valeurs du protocole ; aucun code constructeur ou de bibliothèque recopié.

```toml
[[driver]]
id = "lumieres_radio"
kind = "milight"
[driver.options]
host = "192.168.1.30"
port = 5987
zones = [1, 2]
room = "Pièce principale"
# Pour des ampoules RGB+CCT compatibles, uniquement :
rgbcct = true
```

**État supposé** : l'accusé du pont confirme seulement la réception UDP. Les ampoules
ne renvoient pas leur état ; une télécommande ou l'application peut donc désynchroniser
l'affichage. `state_estimated=true`, description explicite, valeurs inconnues au démarrage.
`online` décrit le pont joignable, jamais la présence radio de chaque ampoule.
Une nouvelle session toutes les 60 s maintient l'accès et vérifie la disponibilité.
Pas de rejeu après erreur ou échéance ; les accusés sont liés à la séquence.
Pas de support des anciens ponts v4/v5, des modes disco ni d'appairage.

Tests : validation de configuration, trames RGBW/RGB+CCT, couleurs primaires, blancs, saturation, bornes, checksum et serveur UDP
fictif (session, accusé correct et mauvais numéro). Recette humaine d'une zone neutre
obligatoire avant d'affirmer la compatibilité d'une installation.

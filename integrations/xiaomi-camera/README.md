# Xiaomi Mi Camera 2K Magnetic Mount

Cible attendue : **MJSXJ03HL / `isa.camera.hlc7`**, à confirmer par la découverte
Xiaomi. Le firmware réel doit être essayé ; aucun RTSP, flash ou root n'est supposé.
Go2rtc 1.9.15 documente ce modèle en CS2/UDP avec vidéo HEVC. Le flux HD est utilisé
pour éviter un problème SPS documenté du sous-flux. Safari doit être testé sur les
appareils réels ; un navigateur sans HEVC reste incompatible, sans encodeur de repli.

```toml
[[driver]]
id = "camera_annexe"
kind = "xiaomi-camera"
[driver.options]
host = "192.168.1.61"
room = "Annexe"
relay_host = "moli-go2rtc"
relay_port = 1984
```

Le compte se saisit dans **Paramètres > Compte Xiaomi Home**. L'adresse et le modèle
doivent correspondre à la découverte cloud. Aucun mot de passe, URL signée ou jeton
n'est renvoyé au navigateur. Le lecteur réutilise le transport HLS protégé des
caméras Tapo, avec HEVC au lieu de H.264. Aucun enregistrement ni transcodage.
Internet reste nécessaire pour obtenir les clés au démarrage de chaque connexion.

## Relais go2rtc

Réutiliser le relais isolé, sans port publié et sans enregistrements. Conserver
sa racine et le fichier de base en lecture seule. Ajouter un premier fichier de
configuration **sur tmpfs**, accessible à l'utilisateur du relais :

```yaml
command: ["go2rtc", "-config", "/tmp/xiaomi.yaml", "-config", "/config/go2rtc.yaml"]
tmpfs:
  - /tmp:size=8m,mode=1777
logging:
  driver: none
```

Le premier chemin est le seul fichier modifiable par l'API go2rtc. Les routes
internes `/api/xiaomi`, `/api/config` et `/api/restart` doivent être autorisées en
plus des routes HLS et streams existantes. Ne pas exposer ces routes directement.
Le compte écrit par go2rtc reste en RAM ; Moli en conserve une copie chiffrée et
la rétablit après redémarrage du relais. Cette restauration redémarre go2rtc et
peut interrompre brièvement les autres lecteurs : vérifier les Tapo après recette.
Les logs du relais sont désactivés car les erreurs amont d'authentification ne
garantissent pas une suppression de tous les secrets. Les erreurs Moli restent
expurgées. Ne jamais placer le compte Xiaomi dans le YAML persistant.

Si la découverte UDP répond depuis l'hôte mais expire depuis Docker, un essai
réversible en `network_mode: host` peut être nécessaire. Ce mode retire l'isolation
réseau du conteneur et doit être approuvé avant application. Pour garder le relais
privé, faire écouter son API sur `127.0.0.1:1984`, désactiver ses écoutes RTSP/WebRTC
et placer aussi Moli sur le réseau hôte, avec `relay_host = "127.0.0.1"`. Retirer
les publications `ports` et choisir explicitement l'adresse LAN et le port HTTP
de Moli, sans conflit avec les services existants. Vérifier les autres caméras
et revenir au réseau précédent si l'essai ne produit aucune amélioration.

Sources : [module Xiaomi go2rtc](https://github.com/AlexxIT/go2rtc/tree/v1.9.15/internal/xiaomi),
[preuve de protocole pour hlc7](https://github.com/AlexxIT/go2rtc/issues/1982).
Compatibilité du matériel non démontrée par ces seuls liens.

Une connexion Xiaomi réussie redémarre aussi le relais pour actualiser son cache de
session cloud. Les lecteurs déjà ouverts peuvent être interrompus brièvement.

## Recadrage HEVC au démarrage

Certains démarrages du modèle `isa.camera.hlc7` envoient d'abord un SPS SD
640 × 360, puis des images HD 2304 × 1296. Un lecteur natif peut conserver les
mauvaises dimensions de l'initialisation HLS. Agrandir le conteneur CSS ou appliquer
une rotation ne corrige pas cette contradiction.

Le correctif ciblé du relais, ses tests synthétiques, sa construction et sa recette
sont décrits dans [docker/go2rtc](../../docker/go2rtc/README.md). Il attend le SPS HD
avant de publier le codec, sous le délai de probe existant. Il n'est actif que si
l'image du relais est remplacée par l'image corrigée ; une mise à jour du seul
binaire Moli ne suffit pas. Pas de transcodage, de port supplémentaire ou de
modification des paramètres de sécurité.

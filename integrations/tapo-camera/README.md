# Caméras Tapo

Caméras C200 et C320WS : RTSP local vers go2rtc isolé, HLS H.264 du sous-flux
`stream2`. Cette implémentation ne commande pas la caméra, ne l'oriente pas,
n'enregistre rien et ne démarre aucun encodeur vidéo.

Le compte caméra local est saisi dans **Caméras**, par un propriétaire du foyer.
Il est vérifié par une connexion RTSP en lecture seule puis conservé dans le
coffre chiffré (`camera_account`). Aucun identifiant RTSP n'est transmis au lecteur.
Les sources go2rtc sont créées en mémoire par PATCH, sans écriture de configuration.
Le relais ne doit publier aucun port sur le LAN ou Internet. Sa configuration et
sa racine restent en lecture seule ; ne jamais activer les logs debug/trace.

```toml
[[driver]]
id = "camera_entree"
kind = "tapo-camera"
[driver.options]
host = "192.168.1.50"
model = "C200"
relay_host = "moli-go2rtc"
relay_port = 1984
```

`relay_host` désigne exclusivement le relais administré par l'installation. Le
navigateur ne choisit aucune adresse réseau ; l'API refuse les instances inconnues.
Les fichiers HLS restent liés à leur caméra et reprennent les restrictions des
instantanés (invités refusés, pièce protégée réservée à une session humaine).

Lecture à la demande, muette, arrêt au changement de page, à la fermeture ou au
passage en arrière-plan. Safari utilise HLS natif ; les autres navigateurs chargent
hls.js light uniquement lors de la lecture. go2rtc ferme sa session après cinq
secondes sans segment demandé. Une carte inactive n'ouvre pas de flux RTSP.

Compatibilité matérielle et firmware à vérifier avec les appareils réels. Aucun
flux ni identifiant réel dans les fixtures ou dans le dépôt. Utiliser un sous-flux
H.264 ; les codecs non pris en charge sont refusés, jamais transcodés en silence.

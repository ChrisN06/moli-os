# Bambu Lab (A1, P1, X1)

- MQTT local en TLS 8883, utilisateur `bblp`, mot de passe = code d'accès ; certificat
  épinglé avant l'envoi du code. Rapports en deltas fusionnés, `pushall` toutes les 5 min.
- Origine : rien de noté. Pas de fixture : le test de fusion utilise un rapport écrit dans
  le code.
- Par défaut : lumière et pause/reprise/arrêt sont disponibles sous la garde du hub.
- `read_only = true` refuse toutes les commandes physiques, lumière comprise, dans les spécifications des points et avant toute publication MQTT. Les requêtes de télémétrie `pushall` restent autorisées.
- Reste : pas encore vu répondre en vrai (A1, P1 et X1) ; le format suit le rapport MQTT
  documenté que lit ha-bambulab.

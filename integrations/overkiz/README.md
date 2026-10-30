# Overkiz cloud — première connexion expérimentale

Pilote natif Rust, désactivé tant qu'aucune instance n'est configurée. Lecture seule par
défaut. Session historique `login`/`JSESSIONID` sur le serveur Somfy Europe uniquement.
**Les comptes Somfy récents utilisant OAuth ou le parcours multi-site ne sont pas pris en
charge.** Valider le parcours du compte avant d'activer cette intégration ; ne pas tenter
plusieurs modes de connexion automatiquement. Aucune preuve matérielle TaHoma V1 à ce jour.

- Écoute des événements toutes les 10 secondes ; minimum 5 secondes. Après échec : appareils
  hors ligne et attente de cinq minutes avant reconnexion, sans commande mise en attente.
  Un login refusé attend une intervention humaine et un redémarrage, sans retenter le mot de passe.
- Identité dérivée de l'URL native encodée sans ambiguïté, jamais du nom.
- Volets `io://`, classe `RollerShutter` uniquement : ouvrir/fermer/arrêter si déclarés sans
  paramètre par le serveur. `read_only = false` exige une validation humaine préalable.
- Position : `core:ClosureState` inversé en ouverture %, nombre JSON ou texte numérique
  compris entre 0 et 100. Aucune position inventée.
- Autres états scalaires exposés sous `raw.<nom>` sans unité supposée. Pas de consigne
  thermostat ni d'énergie configurée automatiquement. Les états découverts ultérieurement
  nécessitent un redémarrage pour obtenir de nouveaux points.
- `last_received` date la réception, pas la mesure par l'appareil. Un cloud joignable ne
  garantit pas une mesure récente. Aucun rafraîchissement radio forcé.
- Aucun ancien historique énergétique récupéré. Aucun cookie ni corps d'erreur journalisé.
- La fixture est **synthétique** ; elle ne prouve aucune compatibilité d'équipement.

Références protocolaires :
https://github.com/iMicknl/python-overkiz-api
https://github.com/Somfy-Developer/Somfy-TaHoma-Developer-Mode

# Overkiz cloud — première connexion expérimentale

Pilote natif Rust, désactivé tant qu'aucune instance n'est configurée. Lecture seule par
défaut. Session historique `login`/`JSESSIONID` sur le serveur Somfy Europe uniquement.
**Les comptes Somfy récents utilisant OAuth ou le parcours multi-site ne sont pas pris en
charge.** Valider le parcours du compte avant d'activer cette intégration ; ne pas tenter
plusieurs modes de connexion automatiquement. Une connexion legacy a été vérifiée ; la génération matérielle exacte reste non confirmée.

- Écoute des événements toutes les 10 secondes ; minimum 5 secondes. Après échec : appareils
  hors ligne et attente de cinq minutes avant reconnexion, sans commande mise en attente.
  Un login refusé attend une intervention humaine et un redémarrage, sans retenter le mot de passe.
- Identité dérivée de l'URL native encodée sans ambiguïté, jamais du nom.
- Volets `io://`, classe `RollerShutter` uniquement : ouvrir/fermer/arrêter si déclarés sans
  paramètre par le serveur. `read_only = false` exige une validation humaine préalable.
- Position : `core:ClosureState` inversé en ouverture %, nombre JSON ou texte numérique
  compris entre 0 et 100. Aucune position inventée.
- Alias en lecture seule pour les modèles identifiés : sondes Somfy Thermostat (`temperature`
  °C, `humidity` %), capteur io Light (`illuminance` lx), cinq modèles de compteurs io
  (`energy` Wh). Les index sont cumulés : le premier relevé sert de référence, jamais de
  consommation du jour. Les sous-index tarifaires ne sont pas additionnés à l'index complet.
- Thermostat : consigne **rapportée**, distincte de la consigne effective d'une dérogation,
  et mode. Zones Atlantic : niveau demandé, pas une preuve de chauffe réelle. Chauffe-eau :
  état Somfy de marche forcée, sans déduction sur sa consommation. Télécommande io : dernière
  touche reçue, sans exécuter un scénario. Ces alias restent non inscriptibles même si les
  commandes de volets sont activées.
- Tous les états scalaires restent sous `raw.<nom>` sans unité supposée. Un modèle inconnu
  ne reçoit aucun alias physique. Les états découverts ultérieurement nécessitent un
  redémarrage pour obtenir de nouveaux points.
- `last_received` date la réception, pas la mesure par l'appareil. Un cloud joignable ne
  garantit pas une mesure récente. Aucun rafraîchissement radio forcé.
- Aucun ancien historique énergétique récupéré. Aucun cookie ni corps d'erreur journalisé.
- La fixture est **synthétique** ; elle ne prouve aucune compatibilité d'équipement.

Références protocolaires :
https://github.com/iMicknl/python-overkiz-api
https://github.com/Somfy-Developer/Somfy-TaHoma-Developer-Mode

Unités des états Overkiz et exceptions :
https://github.com/home-assistant/core/blob/dev/homeassistant/components/overkiz/sensor.py

Position intermédiaire : `percent_control` devient inscriptible seulement si un volet io
annonce `setClosure` à un paramètre et que la lecture seule est désactivée. Seuls les
entiers de 0 à 100 sont acceptés ; ouverture convertie une seule fois en fermeture
(`100 - ouverture`). Aucun autre type d'équipement ne gagne de commande.

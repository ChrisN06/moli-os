# Connexion

1. Dans Tapo, créer un **Compte caméra** local (réglages avancés). Ce compte
   est distinct du compte cloud. Ne pas transmettre de mot de passe dans un chat.
2. L'administrateur déclare l'adresse et le modèle, puis un relais go2rtc interne
   au réseau Docker, sans port publié, sans enregistrement et sans transcodage. Désactiver ses logs
   (`log.level: disabled`) ; Moli fournit le diagnostic utilisateur.
3. Ouvrir **Caméras** dans Moli. Choisir l'appareil et saisir son compte local.
   Si demandé, le propriétaire saisit lui-même son code Moli. Le compte n'est
   enregistré qu'après la vérification de lecture RTSP.
4. Ouvrir **Voir en direct**. Vérifier Safari Mac/iPhone, puis fermer le lecteur
   et confirmer que le relais n'a plus de consommateur après cinq secondes.

Sur une connexion HTTP au NAS, le formulaire n'est pas chiffré sur le LAN. Pour
la saisie des comptes, préférer un accès HTTPS validé ou un tunnel SSH local.
Une panne ne doit jamais afficher une URL RTSP ou un message du relais contenant
les identifiants. Ne jamais activer le mode debug du relais en production.

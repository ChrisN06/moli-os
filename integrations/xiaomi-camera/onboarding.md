# Connexion de la caméra Xiaomi

1. Relever référence, firmware, région et IP fixe, sans supposer RTSP disponible.
2. Préparer le relais go2rtc isolé : premier YAML sur tmpfs, fichier de base en lecture
   seule, API interne autorisée, logs désactivés, aucun port publié.
3. Déclarer le pilote et son IP. Le propriétaire saisit lui-même le compte Xiaomi
   dans **Paramètres**, de préférence en HTTPS. Ne demander aucun secret en chat.
4. Confirmer le modèle trouvé. Un autre modèle bloque la connexion, sans flash ou
   modification du firmware pour contourner l'échec.
5. Sur la page caméras, tester Safari Mac puis iPhone, démarrage automatique, reprise
   après veille et arrêt du flux en quittant la page. Vérifier HEVC, sans encodeur.
6. Vérifier les caméras Tapo après connexion/restauration : le relais est redémarré
   brièvement pour actualiser la session Xiaomi. Tester ensuite le redémarrage complet
   avec restauration du compte depuis le coffre.

La compatibilité n'est acquise qu'après cet essai matériel. Aucun enregistrement.

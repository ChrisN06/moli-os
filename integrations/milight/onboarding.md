# Connexion MI-Light

1. Vérifier que le pont accepte UDP v6 sur 5987 par une ouverture de session sans commande.
   Le nom du module Wi-Fi seul ne prouve pas le protocole des ampoules.
2. Demander le type RGBW et les numéros de zones réellement appairées. Ne pas utiliser 0.
3. Configurer seulement ces zones, avec l'adresse privée du pont et la pièce voulue.
4. Vérifier l'état inconnu et l'avertissement d'état supposé avant la première commande.
5. Avec un humain présent, essayer une zone neutre : marche/arrêt, luminosité, couleur,
   blanc dédié. Remettre son état initial. Jamais une chambre protégée.
6. Vérifier l'autre zone séparément ; une réponse du pont n'atteste pas l'effet radio.
7. Ajouter les identifiants des zones à `home.json` (`salon.lamps`, favoris si souhaité),
   en préservant toutes les étiquettes et pièces existantes. Données privées hors de Git.

Aucun mot de passe cloud. Aucun changement du firmware ou de l'appairage des ampoules.

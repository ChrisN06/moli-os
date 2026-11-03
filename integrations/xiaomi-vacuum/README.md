# Xiaomi Mi Robot Vacuum

Le pilote cible uniquement **SDJQR01RR / `rockrobo.vacuum.v1`**. Le firmware réel
et ses capacités restent à valider : « à jour » ne garantit pas les commandes de
zone ni l'accès à la carte. Aucun flash, root ou nettoyage général de secours.

Connexion dans **Paramètres > Compte Xiaomi Home**, région France/Europe (`de`).
Le propriétaire saisit lui-même ses identifiants ; captcha et double authentification
restent manuels. Le mot de passe est envoyé en HTTPS à Xiaomi puis oublié. Le
jeton local et le compte de session sont conservés dans le coffre chiffré Moli,
jamais dans Git, les réponses API ou les journaux. Privilégier HTTPS pour la saisie.

Le relais go2rtc doit être préparé comme décrit dans l'intégration caméra Xiaomi.
La découverte accepte uniquement l'adresse configurée et le modèle attendu. Elle
vérifie `miIO.info` et `get_status`, sans déplacement, avant d'enregistrer le compte.

```toml
[[driver]]
id = "robot_rdc"
kind = "xiaomi-vacuum"
[driver.options]
host = "192.168.1.60"
poll_s = 30
relay_host = "moli-go2rtc"
relay_port = 1984
```

## Lecture et déplacements

Batterie, état, surface, durée et erreurs sont lus localement en UDP miIO/AES-CBC.
Les valeurs absentes restent absentes. Les commandes deviennent disponibles après connexion du compte et vérification
du modèle par le pilote. Aucun réglage supplémentaire de déplacement n’est requis.
Pause, retour à la base et cuisine passent par la garde de commandes Moli, avec échéance. Une
commande moteur sans réponse n'est jamais répétée automatiquement.

La page **Aspirateur** charge la carte RR sur demande, via le cloud Xiaomi, puis
permet de dessiner et confirmer un rectangle. Les coordonnées sont celles du robot,
en millimètres ; la capture d'écran seule ne permet pas de les déduire. Avant tout
nettoyage, la carte et la base sont vérifiées à nouveau. Un changement impose une
nouvelle confirmation. La zone utilise un seul passage et `app_zoned_clean` ; aucun
appel `app_start` ou nettoyage global de repli. Le robot peut traverser le salon
pour rejoindre la cuisine : le rectangle n'est pas une barrière virtuelle.

La carte nécessite Internet et un format RR reconnu. Un échec de carte bloque le
nettoyage de cuisine mais ne désactive pas la télémétrie locale. La zone est stockée
dans le coffre et incluse dans les sauvegardes existantes.

Sources de protocole : [python-miio](https://github.com/rytilahti/python-miio),
[RR-map](https://github.com/PiotrMachowski/Python-package-vacuum-map-parser-roborock),
[go2rtc Xiaomi](https://github.com/AlexxIT/go2rtc/tree/v1.9.15/internal/xiaomi).
Ces interfaces ne sont pas une API officielle Xiaomi garantie.

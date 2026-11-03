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
Démarrage classique, pause, retour à la base et cuisine passent par la garde de
commandes Moli, avec échéance. Le bouton de démarrage classique envoie explicitement
`app_start`, sans dépendre de la carte ni de la zone cuisine. Une
commande moteur sans réponse n'est jamais répétée automatiquement.

La page **Aspirateur** demande la référence de carte via le RPC cloud Xiaomi
`get_map_v1`, puis télécharge la carte RR sur demande. Elle ne dépend pas d’une
nouvelle connexion miIO locale pour cette lecture et
permet de dessiner et confirmer un rectangle. Les coordonnées sont celles du robot,
en millimètres ; la capture d'écran seule ne permet pas de les déduire. Avant tout
nettoyage de cuisine, la carte et la base sont vérifiées à nouveau. Un changement impose une
nouvelle confirmation. La zone utilise un seul passage et `app_zoned_clean` ; aucun
appel `app_start` ni nettoyage global de repli dans le parcours cuisine. Le robot peut traverser le salon
pour rejoindre la cuisine : le rectangle n'est pas une barrière virtuelle.

La carte nécessite Internet et un format RR reconnu. Un échec de carte bloque le
nettoyage de cuisine mais ne désactive pas la télémétrie locale. La zone est stockée
dans le coffre et incluse dans les sauvegardes existantes.

Sources de protocole : [python-miio](https://github.com/rytilahti/python-miio),
[RR-map](https://github.com/PiotrMachowski/Python-package-vacuum-map-parser-roborock),
[go2rtc Xiaomi](https://github.com/AlexxIT/go2rtc/tree/v1.9.15/internal/xiaomi).
Ces interfaces ne sont pas une API officielle Xiaomi garantie.

La route cloud RPC est limitée à cette requête de lecture, avec identifiant numérique
borné et paramètres vides. Aucune commande moteur ne passe par cette route.
Source du transport : [openHAB MiCloudConnector](https://github.com/openhab/openhab-addons/blob/main/bundles/org.openhab.binding.miio/src/main/java/org/openhab/binding/miio/internal/cloud/MiCloudConnector.java).

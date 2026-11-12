# Moli OS sur Android TV

Moli OS apparaît dans les applications Android TV. Les flèches déplacent la
sélection, OK active l’élément sélectionné et Retour ferme la fiche ouverte
ou revient à la page précédente. Le mode TV s’ouvre aussi dans un navigateur
avec `/?tv=1#/` lorsque le serveur utilise ce dashboard.

L’APK embarque le dashboard issu de ce checkout. Les requêtes d’API, les
sessions et les flux passent par l’instance Moli indiquée à la construction.
L’adresse figure dans l’APK. Garder un APK configuré pour une maison dans
ses fichiers privés. Pour mettre à jour le dashboard embarqué, reconstruire
l’APK et le signer avec le même certificat.

Sur un Mac équipé de Node et des dépendances de `web/`, préparer un JDK 17+
et les outils Android (plateforme 35 ou plus, SDK Build Tools). Indiquer
`JAVA_HOME`, `ANDROID_JAR` et `ANDROID_BUILD_TOOLS`. Indiquer également
`MOLI_TV_URL`, `MOLI_TV_KEYSTORE` et `MOLI_TV_STOREPASS_FILE`. Garder le
coffre de signature et le fichier de mot de passe hors du dépôt.

Depuis la racine du dépôt, construire et installer sur la TV choisie :

```sh
sh mobile/tv/build.sh /tmp/moli-tv.apk
adb -s "$TV_SERIAL" install --no-incremental -r /tmp/moli-tv.apk
```

L’URL doit être une origine HTTP(S) sans chemin. Préférer HTTPS si
l’instance dispose déjà d’un certificat reconnu par la TV. En HTTP,
l’application autorise le trafic en clair pour l’hôte configuré.

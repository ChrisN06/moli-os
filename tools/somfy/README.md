# Préparer les historiques énergétiques TaHoma

`somfy-energy-export.py` utilise seulement la bibliothèque standard Python 3.9+.
Il ne se connecte à aucun service et n'ouvre jamais `energy.db`.
Les CSV personnels et les résultats restent hors du dépôt public.

## Format observé

L'export `state-values` commence par le nom du capteur, puis
`core:ElectricEnergyConsumptionState in Wh`. Il contient deux séries distinctes :

- relevés récents : `YYYY-MM-DD HH:MM:SS,index` ;
- historique quotidien : en-tête vide suivi de `device` et de catégories,
  puis `YYYY-MM-DD,index,...`.

La colonne **device** représente le compteur exporté. La colonne `electric`
des exports par circuit peut répéter le compteur général : ne pas l'importer
comme une consommation supplémentaire. Ne pas mélanger les deux séries.
Température, hygrométrie et luminosité sont refusées par cet outil énergétique.

## Prévisualiser sans importer

```sh
python3 tools/somfy/somfy-energy-export.py general=/chemin/prive/general.csv
```

La sortie standard reste vide. Le rapport sur stderr décrit les deux séries.
Le mapping est explicite : utiliser les identifiants configurés dans
`[[energy.meter]]` (`general`, `chauffage`, `chauffe_eau`, `prises`, `eclairage`, selon la configuration).
L'outil ne devine jamais une catégorie d'après le nom ou le numéro du fichier.

Avant toute conversion quotidienne, confirmer avec la source le **fuseau horaire**
et si la date signifie l'index en **début** ou **fin** de journée. Le CSV ne le dit
pas. Une heure locale ambiguë ou inexistante au changement d'heure est refusée.

Après confirmation seulement, `--timezone ZONE --daily-anchor start|end` permet
une prévisualisation chiffrée ; ajouter `--emit` produit du JSONL compatible avec
l'importeur existant. Aucun import n'est déclenché. Pour les relevés précis,
`--series recent --timezone ZONE --emit` ne nécessite pas de convention de date.

## Conservation et limites

- Les différences d'index Wh deviennent des kWh. Les heures entre deux index sont
  **estimées au prorata du temps**, pas des mesures de puissance. Les journées de
  changement d'heure ont réellement 23 ou 25 heures.
- Les index identiques au même instant et les mêmes heures répétées sont
  dédupliqués ; les conflits sont refusés avant toute émission.
- Baisse d'index, intervalle de plus de 48 h, débit invraisemblable : intervalle
  signalé et ignoré, ainsi que les heures incomplètes correspondantes. Ne pas
  corriger automatiquement un reset. Examiner le rapport avant tout import.
- Première et dernière heures partielles exclues. Pas d'extrapolation avant le
  premier relevé ni après le dernier. Le premier index n'est pas une consommation.
- Aucun prix ou tarif inventé. Les totaux et les circuits conservent leur rôle.
- L'étape d'écriture future réutilise `moli-os energy import` : transaction SQLite,
  clé `(meter, hour)`, doublons ignorés et protection des heures déjà collectées
  en direct. Faire une sauvegarde cohérente avant cette étape ; traiter des lots
  bornés (par exemple 10 000 lignes) pour préserver la RAM du serveur.
- Ne pas écraser une heure existante pour combler un export partiel. Les lacunes
  restent des lacunes. Une restauration de sauvegarde serait une opération distincte.

Tests synthétiques : `python3 -m unittest discover -s tools/somfy -p 'test_*.py'`.

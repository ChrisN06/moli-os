# Première connexion Overkiz

1. Identifier la box et l'application. La première TaHoma carrée n'a pas l'API locale
   Developer Mode. Ne jamais réinitialiser la box ni remplacer son application pour ce test.
2. Vérifier que le compte utilise le login historique ; les parcours OAuth et multi-site
   ne sont pas encore implémentés. Un refus ne prouve pas un mauvais mot de passe.
3. L'humain saisit directement username et password dans le coffre de l'instance `somfy`,
   par l'entrée standard de `moli-os secrets set somfy <nom>`, sans historique shell ni
   affichage. Aucun secret ne passe dans la conversation, les captures ou le dépôt.
4. Ajouter la configuration ci-dessous au fichier privé, puis redémarrer cette instance.
5. Comparer les états avec l'application pendant 48 h. Vérifier l'ancienneté et les valeurs
   après coupure réseau. Ne jamais prendre `last_received` pour une date de mesure.
6. Seulement ensuite, activer `read_only = false` et tester un volet neutre avec un humain
   présent. Restaurer sa position. L'acceptation cloud n'est pas la fin du mouvement.

```toml
[[driver]]
id = "somfy"
kind = "overkiz"
[driver.options]
read_only = true
poll_seconds = 10
```

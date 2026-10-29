# Essai gratuit de Clippy (Cloudflare Worker)

Quelques réécritures offertes à chaque nouvelle installation de Rafiq, avant de mettre sa propre clé
d'API. Le Worker garde la clé DeepSeek en secret et compte les essais dans un KV :

- `PER_INSTALL` essais par installation (identifiant aléatoire tiré au premier lancement, `install_id`
  dans `%APPDATA%\rafiq\ai.json`) ;
- `PER_IP_DAY` par adresse IP et par jour ; `GLOBAL_DAY` pour tout le monde et par jour ;
- textes de `MAX_CHARS` caractères au plus. Rien du texte n'est journalisé ni gardé.

Épuisé : réponse 429 avec un message clair, que Clippy affiche.

## Déployer

```powershell
cd server\clippy-trial
npx wrangler login                          # ouvre le navigateur, compte Cloudflare
npx wrangler kv namespace create TRIAL      # copier l'id affiché dans wrangler.toml
npx wrangler secret put DEEPSEEK_API_KEY    # coller la clé quand il la demande
npx wrangler deploy                         # affiche l'URL https://rafiq-clippy-trial.<compte>.workers.dev
```

En service : `https://rafiq-clippy-trial.smrafiq.workers.dev/v1/clippy`, l'adresse de `TRIAL_URL`
(`windows/rafiq/src/ai.rs`) ; un autre déploiement demande d'y mettre sa propre adresse et de reconstruire Rafiq. Changer un plafond : modifier `[vars]` dans `wrangler.toml`, puis `npx wrangler deploy`.

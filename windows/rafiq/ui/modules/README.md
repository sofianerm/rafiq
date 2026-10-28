# Modules du notch

Un **module** est une cellule du notch qui n'est pas un provider d'usage IA : un anneau dans le pill,
un petit texte dessous, et une card au survol. `notch.html` s'occupe du dessin, du survol, du
placement de la card et du repli ; le module ne décrit que son contenu.

## Ajouter un module

1. Créer `modules/mon-module.js` :

   ```js
   registerModule({
     id:'mon-module',          // unique, [a-z0-9-]
     name:'Mon module',        // nom court
     title:'Mon module',       // titre de la card (défaut : name)
     glyph:'Mm',               // 1–2 lettres dans l'anneau si pas d'icône
     icon:'<svg viewBox="0 0 24 24">…</svg>', // optionnel, suit currentColor ; ou icon(){…} si elle change

     init(ctx){ this.ctx=ctx; },       // une fois au démarrage
     badge(){ return '3'; },           // texte sous l'anneau (optionnel)
     progress(){ return {value:.4,color:'#4DA3FF'}; }, // remplit l'anneau (0–1, ou un nombre seul)
     activity(){ return 'attention'; }, // arc intérieur : 'running' (tourne) ou 'attention' (pulse)
     render(){ return '<div class="c-note">…</div>'; }, // HTML du corps de la card
     mount(el){ /* brancher des événements sur la card rendue */ },
     onClick(){ /* clic sur l'anneau */ },
   });
   ```

2. Ajouter une ligne dans `notch.html`, après `modules/core.js` :

   ```html
   <script src="modules/mon-module.js"></script>
   ```

Les modules s'affichent sous les providers, dans l'ordre des balises `<script>`.

## `ctx`

| Champ | Rôle |
|---|---|
| `ctx.update()` | À appeler quand l'état change : redessine l'anneau, et la card si elle est ouverte sur ce module |
| `ctx.invoke(cmd, args)` | Commande Tauri côté Rust (`#[tauri::command]`) |
| `ctx.listen(event, cb)` | Événement émis par Rust (`app.emit(...)`) |
| `ctx.esc(s)` | Échappe du texte avant de l'injecter en HTML — **obligatoire** pour tout contenu externe (presse-papier, fichiers…) |
| `ctx.lang()` | Langue de l'UI (`en`, `ru`, `zh`…) |
| `ctx.visible()` | Vrai si la card de ce module est ouverte (pour rafraîchir plus souvent) |

## Règles

- Une exception dans un module est attrapée et loguée : elle ne casse pas le notch.
- `render()` peut être rappelée souvent (à chaque événement d'usage pendant que la card est ouverte) :
  pas d'effet de bord dedans.
- Classes CSS réutilisables : `c-note` (texte), `c-sub` (texte gris), `win`/`w-row`/`w-label`,
  `c-sessions`/`s-row`/`s-dot` (listes).
- Pour des données système (presse-papier Windows, etc.), la lecture se fait côté Rust
  (`src/`), exposée par une commande et/ou un événement, puis consommée ici via `ctx`.

Exemples : [`hello.js`](hello.js) (minimal, non chargé), [`prayer.js`](prayer.js), [`tpg.js`](tpg.js), [`meteo.js`](meteo.js), [`clipboard.js`](clipboard.js).

Une API web avec CORS ouvert (Open-Meteo, transport.opendata.ch) peut être appelée directement en `fetch` depuis le module ; sinon, passer par Rust.

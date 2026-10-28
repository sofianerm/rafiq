/* Module de démonstration : un texte simple dans la card, et un compteur de clics sur l'anneau
   pour montrer le cycle init → onClick → ctx.update() → render. */
registerModule({
  id:'hello',
  name:'Hello',
  title:'Module de test',
  glyph:'Hi',
  clicks:0,

  init(ctx){ this.ctx=ctx; },

  badge(){ return this.clicks?String(this.clicks):'Hi'; },

  render(){
    return `<div class="c-note">Bonjour 👋<br>Ceci est un module bidon affiché dans le notch.</div>
      <div class="c-sub" style="margin-top:8px">Clique sur l'anneau : ${this.clicks} clic${this.clicks>1?'s':''}</div>`;
  },

  onClick(){ this.clicks++; this.ctx.update(); },
});

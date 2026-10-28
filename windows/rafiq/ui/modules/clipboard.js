/* Historique du presse-papier (texte). Sous l'anneau, le nombre d'éléments ; au survol, les épinglés
   puis les derniers copiés : un clic sur une ligne la recopie, 📌 l'épingle (gardée au redémarrage),
   × la retire. Les données viennent de src/clipboard.rs, qui ignore ce que les gestionnaires de mots
   de passe marquent comme privé. */
(()=>{
const SHOWN=15;
const PIN='<svg viewBox="0 0 24 24"><path d="M15 3l6 6-2 1-3 3 .5 4.5L15 19l-3.5-3.5L6 21l-1-1 5.5-5.5L7 11l1.5-1.5L13 10l3-3z"/></svg>';

function ago(ms){
  const m=Math.round((Date.now()-ms)/60000);
  if(m<1) return "à l'instant";
  if(m<60) return m+' min';
  const h=Math.round(m/60); if(h<24) return h+' h';
  return Math.round(h/24)+' j';
}

const STYLE=`
.cb-sec{font-size:11px;font-weight:700;color:#808080;text-transform:uppercase;letter-spacing:.4px;margin:10px 0 4px}
.cb-sec:first-child{margin-top:0}
.cb-row{display:flex;align-items:center;gap:6px;padding:5px 6px;margin:0 -6px;border-radius:6px;cursor:pointer;font-size:12px}
.cb-row:hover{background:#1c1c1c}
.cb-txt{flex:1;min-width:0;white-space:nowrap;overflow:hidden;text-overflow:ellipsis;color:#e8e8ea}
.cb-row.copied .cb-txt{color:#00FF88}
.cb-at{color:#606060;font-size:10px;white-space:nowrap}
.cb-btn{width:18px;height:18px;border:none;background:none;padding:0;color:#606060;cursor:pointer;display:flex;align-items:center;justify-content:center;font-size:14px;line-height:1;border-radius:4px}
.cb-btn svg{width:13px;height:13px;fill:currentColor}
.cb-btn:hover{color:#fff;background:#2a2a2a}
.cb-btn.on{color:#F2FF00}
.cb-row:not(:hover) .cb-btn:not(.on){visibility:hidden}
.cb-foot{display:flex;justify-content:space-between;margin-top:10px;font-size:10px;color:#606060}
.cb-clear{cursor:pointer;text-decoration:underline}
`;

registerModule({
  id:'clipboard',
  name:'Presse-papier',
  title:'Presse-papier',
  glyph:'⎘',
  icon:'<svg viewBox="0 0 24 24"><path d="M9 2h6a1 1 0 0 1 1 1v1h2a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2V6a2 2 0 0 1 2-2h2V3a1 1 0 0 1 1-1zm0 2v2h6V4zM8 11v1.6h8V11zm0 4v1.6h6V15z"/></svg>',
  items:[],
  copied:null,

  init(ctx){
    this.ctx=ctx;
    const st=document.createElement('style'); st.textContent=STYLE; document.head.appendChild(st);
    const take=l=>{ this.items=Array.isArray(l)?l:[]; ctx.update(); };
    ctx.listen('clipboard',e=>take(e.payload)).catch(()=>{});
    ctx.invoke('get_clipboard').then(take).catch(()=>{});
  },

  badge(){ return String(this.items.length); },

  row(i){
    const esc=this.ctx.esc;
    const preview=i.text.replace(/\s+/g,' ').trim().slice(0,200);
    return `<div class="cb-row${this.copied===i.id?' copied':''}" data-id="${i.id}" title="${esc(i.text.slice(0,500))}">`
      +`<span class="cb-txt">${this.copied===i.id?'Copié ✓':esc(preview)}</span><span class="cb-at">${ago(i.at)}</span>`
      +`<button class="cb-btn cb-pin${i.pinned?' on':''}" title="${i.pinned?'Désépingler':'Épingler'}">${PIN}</button>`
      +`<button class="cb-btn cb-del" title="Retirer">×</button></div>`;
  },

  render(){
    if(!this.items.length) return `<div class="c-note">Rien pour l'instant. Copie du texte (Ctrl+C) : il apparaîtra ici.</div>`;
    const pinned=this.items.filter(i=>i.pinned), recent=this.items.filter(i=>!i.pinned);
    let html='';
    if(pinned.length) html+=`<div class="cb-sec">Épinglés</div>`+pinned.map(i=>this.row(i)).join('');
    if(recent.length) html+=`<div class="cb-sec">Récents</div>`+recent.slice(0,SHOWN).map(i=>this.row(i)).join('');
    html+=`<div class="cb-foot"><span>${recent.length>SHOWN?`+ ${recent.length-SHOWN} plus anciens`:''}</span>${recent.length?'<span class="cb-clear">Effacer l\'historique</span>':''}</div>`;
    return html;
  },

  mount(el){
    const on=(sel,fn)=>el.querySelectorAll(sel).forEach(n=>n.addEventListener('mousedown',ev=>{ ev.stopPropagation(); if(ev.button===0) fn(n,ev); }));
    const idOf=n=>Number(n.closest('.cb-row').dataset.id);
    on('.cb-pin',n=>{ const id=idOf(n), it=this.items.find(i=>i.id===id); this.ctx.invoke('clipboard_pin',{id,pinned:!(it&&it.pinned)}).catch(()=>{}); });
    on('.cb-del',n=>this.ctx.invoke('clipboard_delete',{id:idOf(n)}).catch(()=>{}));
    on('.cb-clear',()=>this.ctx.invoke('clipboard_clear').catch(()=>{}));
    on('.cb-row',(n,ev)=>{
      if(ev.target.closest('.cb-btn')) return;
      const id=Number(n.dataset.id);
      this.ctx.invoke('clipboard_copy',{id}).then(()=>{
        this.copied=id; this.ctx.update();
        setTimeout(()=>{ if(this.copied===id){ this.copied=null; this.ctx.update(); } },1200);
      }).catch(()=>{});
    });
  },
});
})();

/* Clippy dans le notch : relire, traduire, reformuler un texte avec une IA (src/ai.rs).
   - Ctrl+Alt+C n'importe où : la sélection est remplacée par le résultat (raccourci géré en Rust).
   - La card : choisir l'action, coller ou taper un texte, Ctrl+Entrée ; le résultat remplace le texte
     et part dans le presse-papier. « Presse-papier » traite ce qui y est déjà.
   - Réglages : fournisseur, modèle, clé API (jamais renvoyée à la page), raccourci, collage auto.
   Taper demande le focus clavier, que le notch ne prend jamais d'habitude : il est emprunté le temps
   de l'édition (ai_focus) et la card reste ouverte tant qu'on écrit (keepOpen). */
(()=>{
const STYLE=`
/* Une seule grammaire pour la card (214 px de large) : pas de 8 px, une hauteur (28 px) et un rayon
   (8 px) pour les tags et les boutons, aucun libellé coupé (nowrap + points de suspension). */
.cl-chips{display:grid;grid-template-columns:1fr 1fr;gap:6px;margin-bottom:8px}
.cl-chip{height:26px;display:flex;align-items:center;justify-content:center;box-sizing:border-box;font:600 11px "Segoe UI",sans-serif;color:#a8a8ac;background:#141414;border:1px solid #242424;border-radius:8px;padding:0 8px;cursor:pointer;white-space:nowrap;overflow:hidden;text-overflow:ellipsis;min-width:0}
.cl-chip:last-child:nth-child(odd){grid-column:1/-1}
.cl-chip:hover{border-color:#3a3a3a;color:#e8e8ea}
.cl-chip.on{background:rgba(0,255,136,.12);border-color:#00FF88;color:#00FF88}
.cl-text{display:block;width:100%;min-height:96px;max-height:220px;resize:none;box-sizing:border-box;background:#111;border:1px solid #242424;border-radius:8px;color:#e8e8ea;font:12px/1.45 "Segoe UI",sans-serif;padding:8px;outline:none;user-select:text}
.cl-text:focus{border-color:rgba(0,255,136,.55)}
.cl-text::placeholder{color:#5a5a5a}
.cl-row{display:flex;gap:6px;margin-top:8px;align-items:center}
.cl-btn{height:28px;box-sizing:border-box;display:inline-flex;align-items:center;justify-content:center;gap:6px;font:600 11px "Segoe UI",sans-serif;border-radius:8px;padding:0 10px;cursor:pointer;border:1px solid #2e2e2e;background:transparent;color:#c8c8c8;white-space:nowrap;flex:none}
.cl-btn:hover{border-color:#555;color:#fff}
.cl-btn svg{width:14px;height:14px;fill:none;stroke:currentColor;stroke-width:2;stroke-linecap:round;stroke-linejoin:round}
.cl-btn.go{flex:1;min-width:0;background:#00FF88;border-color:#00FF88;color:#000}
.cl-btn.go span{overflow:hidden;text-overflow:ellipsis}
.cl-btn.go:hover{filter:brightness(1.1);color:#000}
.cl-btn.icon{width:28px;padding:0}
.cl-btn.ghost{border-color:transparent;color:#808080}
.cl-btn.ghost:hover{border-color:#2e2e2e;color:#e8e8ea}
.cl-btn:disabled{opacity:.45;cursor:wait}
.cl-status{font-size:11px;color:#808080;margin-top:8px;height:15px;line-height:15px;white-space:nowrap;overflow:hidden;text-overflow:ellipsis}
.cl-status.ok{color:#00FF88}.cl-status.err{color:#FF8A70}.cl-status.busy{color:#F2FF00}
.cl-hint{font-size:11px;color:#808080;margin-top:8px;padding-top:8px;border-top:1px solid #1a1a1a;display:flex;justify-content:space-between;align-items:center;gap:8px;white-space:nowrap}
.cl-hint > span{overflow:hidden;text-overflow:ellipsis}
.cl-hint kbd{font:600 10px "Segoe UI",sans-serif;color:#c8c8c8;background:#161616;border:1px solid #2a2a2a;border-radius:5px;padding:1px 5px;margin-right:6px}
.cl-hint .cl-btn{height:24px;padding:0 8px}
.cl-set{margin-top:10px;border-top:1px solid #1e1e1e;padding-top:10px;display:grid;grid-template-columns:70px 1fr;gap:6px 8px;align-items:center;font-size:11px;color:#9a9a9a}
.cl-set input,.cl-set select{font:12px "Segoe UI",sans-serif;background:#111;border:1px solid #2a2a2a;border-radius:6px;color:#e8e8ea;padding:4px 6px;outline:none;min-width:0;user-select:text}
.cl-set input:focus,.cl-set select:focus{border-color:rgba(0,255,136,.55)}
.cl-set .full{grid-column:1/-1;display:flex;gap:6px;align-items:center}
.cl-set .model{display:flex;gap:4px;min-width:0}
.cl-set .model>*:first-child{flex:1;min-width:0}
.cl-set .ref{font:13px "Segoe UI";background:#161616;border:1px solid #2a2a2a;border-radius:6px;color:#9a9a9a;cursor:pointer;padding:0 7px}
.cl-set .ref:hover{color:#fff;border-color:#555}
.cl-set .note{grid-column:2;font-size:10px;color:#606060;margin-top:-3px}
.cl-set .note.err{color:#FF8A70}
.cl-set label.chk{grid-column:1/-1;display:flex;gap:6px;align-items:center;cursor:pointer}
`;
// Les libellés de tags tiennent sur une demi-largeur : « Traduire en anglais » devient « ⇄ Anglais »
const chipLabel=n=>{ const m=/^(?:traduire en|translate (?:to|into)) (.+)$/i.exec(n); return m?'⇄ '+m[1].charAt(0).toUpperCase()+m[1].slice(1):n; };
const CLIP_ICON='<svg viewBox="0 0 24 24"><rect x="8" y="3" width="8" height="4" rx="1.5"/><path d="M16 5h1.5A1.5 1.5 0 0 1 19 6.5v13a1.5 1.5 0 0 1-1.5 1.5h-11A1.5 1.5 0 0 1 5 19.5v-13A1.5 1.5 0 0 1 6.5 5H8"/></svg>';
const UNDO_ICON='<svg viewBox="0 0 24 24"><path d="M9 14 4 9l5-5"/><path d="M4 9h10.5a5.5 5.5 0 0 1 0 11H11"/></svg>';
const SPARK='<svg viewBox="0 0 24 24"><path d="M12 1.5c.6 5.3 3.4 8.6 9.5 10.5-6.1 1.9-8.9 5.2-9.5 10.5-.6-5.3-3.4-8.6-9.5-10.5C8.6 10.1 11.4 6.8 12 1.5z"/><path d="M19.5 1.5c.2 1.6 1 2.5 2.5 3-1.5.5-2.3 1.4-2.5 3-.2-1.6-1-2.5-2.5-3 1.5-.5 2.3-1.4 2.5-3z"/></svg>';

registerModule({
  id:'clippy',
  name:'Clippy',
  title:'Clippy',
  glyph:'✦',
  icon:SPARK,
  v:null,            // la vue renvoyée par Rust (sans les clés)
  draft:'',          // le texte de la card
  prev:null,         // le texte avant la dernière action, pour « Annuler »
  busy:false,
  status:null,       // {text, kind:'ok'|'err'|'busy'}
  flash:null,        // ✓ ou ! sous l'anneau quelques secondes après un raccourci
  editing:false,
  settings:false,
  form:null,
  focus:null,        // {field, start, end} : où remettre le curseur après un rendu
  models:{},         // fournisseur → {list, loading, error} : la vraie liste, demandée à l'API

  init(ctx){
    this.ctx=ctx;
    const st=document.createElement('style'); st.textContent=STYLE; document.head.appendChild(st);
    ctx.invoke('ai_get').then(v=>{ this.v=v; ctx.update(); }).catch(()=>{});
    ctx.listen('ai_state',e=>{
      const s=e.payload||{}; this.busy=!!s.busy;
      if(s.busy) this.status={text:s.message,kind:'busy'};
      else if(s.ok!=null){
        if(s.origin==='hotkey'){ this.flash=s.ok?'✓':'!'; clearTimeout(this._f); this._f=setTimeout(()=>{ this.flash=null; ctx.update(); },4000); }
        if(s.ok) ctx.invoke('ai_get').then(v=>{ this.v=v; ctx.update(); }).catch(()=>{}); // essais restants à jour
        this.status=s.ok?{text:s.origin==='hotkey'?LT('Sélection remplacée','Selection replaced'):LT('Copié · Ctrl+V pour coller','Copied · Ctrl+V to paste'),kind:'ok'}:{text:s.message,kind:'err'};
      }
      ctx.update();
    }).catch(()=>{});
  },

  provider(){ const v=this.v; return v&&v.providers.find(p=>p.id===v.provider); },

  badge(){ return this.busy?'…':this.flash||'IA'; },
  activity(){ return this.busy?'running':null; },
  keepOpen(){ return this.editing; },
  onHide(){ if(this.editing) this.release(); },

  // Le focus clavier n'est emprunté que pour écrire, et rendu dès qu'on a fini
  grab(){ if(!this.editing){ this.editing=true; this.ctx.invoke('ai_focus',{on:true}).catch(()=>{}); } },
  release(){ this.editing=false; this.focus=null; this.ctx.invoke('ai_focus',{on:false}).catch(()=>{}); },

  render(){
    const esc=this.ctx.esc, v=this.v;
    if(!v) return `<div class="c-note">${LT('Chargement…','Loading…')}</div>`;
    const p=this.provider();
    let html=`<div class="cl-chips">`+v.actions.map(a=>`<span class="cl-chip${a.name===v.action?' on':''}" data-action="${esc(a.name)}" title="${esc(a.name)} — ${esc(a.prompt.slice(0,160))}…">${esc(chipLabel(a.name))}</span>`).join('')+`</div>`;
    html+=`<textarea class="cl-text" data-f="text" spellcheck="false" placeholder="${LT('Colle ou tape un texte…  Ctrl+Entrée pour appliquer','Paste or type some text…  Ctrl+Enter to apply')}">${esc(this.draft)}</textarea>`;
    html+=`<div class="cl-row"><button class="cl-btn go" data-do="go"${this.busy?' disabled':''} title="${esc(v.action)} (${LT('Ctrl+Entrée','Ctrl+Enter')})"><span>✦ ${esc(v.action)}</span></button>`
      +`<button class="cl-btn icon" data-do="clip"${this.busy?' disabled':''} title="${LT('Appliquer au texte du presse-papier','Apply to the clipboard text')}" aria-label="${LT('Presse-papier','Clipboard')}">${CLIP_ICON}</button>`
      +(this.prev!=null&&!this.busy?`<button class="cl-btn icon ghost" data-do="undo" title="${LT("Annuler : revenir au texte d'avant",'Undo: back to the previous text')}" aria-label="${LT('Annuler','Undo')}">${UNDO_ICON}</button>`:'')+`</div>`;
    // Sans clé : l'essai gratuit tant qu'il en reste, sinon on demande une clé
    const trial=v.trial;
    const st=this.status||(p&&!p.has_key?(trial>0?{text:LT(`${trial} essai${trial>1?'s':''} gratuit${trial>1?'s':''} restant${trial>1?'s':''}`,`${trial} free ${trial>1?'tries':'try'} left`),kind:'ok'}
      :{text:trial===0?LT('Essais gratuits utilisés · ajoute ta clé (Réglages)','Free tries used · add your key (Settings)'):LT(`Ajoute une clé API ${p.name} dans les réglages`,`Add a ${p.name} API key in the settings`),kind:'err'}):null);
    html+=`<div class="cl-status ${st?st.kind:''}">${st?esc(st.text):esc(p?`${p.name} · ${p.model}`:'')}</div>`;
    html+=`<div class="cl-hint"><span title="${LT("Sur un texte sélectionné, dans n'importe quelle app",'On selected text, in any app')}"><kbd>${esc(v.hotkey||'—')}</kbd>${v.hotkey&&!v.hotkey_ok?'<span style="color:#FF8A70">'+LT('indisponible','unavailable')+'</span>':LT('sur une sélection','on a selection')}</span><button class="cl-btn ghost" data-do="settings">${this.settings?LT('Fermer','Close'):LT('Réglages','Settings')}</button></div>`;
    if(this.settings) html+=this.renderSettings();
    return html;
  },

  renderSettings(){
    const esc=this.ctx.esc, v=this.v;
    const f=this.form||(this.form={provider:v.provider,model:this.provider().model,key:'',hotkey:v.hotkey,auto:v.auto_paste,msg:null});
    const p=v.providers.find(x=>x.id===f.provider)||v.providers[0];
    return `<div class="cl-set">`
      +`<span>${LT('Fournisseur','Provider')}</span><select data-f="provider">${v.providers.map(x=>`<option value="${x.id}"${x.id===f.provider?' selected':''}>${esc(x.name)}</option>`).join('')}</select>`
      +`<span>${LT('Modèle','Model')}</span><div class="model">${this.modelField(f,p)}<button class="ref" data-do="models" title="${LT('Recharger la liste depuis','Reload the list from')} ${esc(p.name)}">↻</button></div>`+this.modelNote(f)
      +`<span>${LT('Clé API','API key')}</span><input data-f="key" type="password" value="${esc(f.key)}" placeholder="${p.has_key?LT('•••••• enregistrée','•••••• saved'):LT('Colle ta clé','Paste your key')}">`
      +`<span>${LT('Raccourci','Shortcut')}</span><input data-f="hotkey" value="${esc(f.hotkey)}" placeholder="Ctrl+Alt+C">`
      +`<label class="chk"><input type="checkbox" data-f="auto"${f.auto?' checked':''}> ${LT('Coller le résultat à la place de la sélection','Paste the result over the selection')}</label>`
      +`<div class="full"><button class="cl-btn go" data-do="save">${LT('Enregistrer','Save')}</button><span class="cl-status ${f.msg?f.msg.kind:''}" style="margin:0">${f.msg?esc(f.msg.text):''}</span></div>`
      +`</div>`;
  },

  // La liste de l'API quand on l'a ; sinon un champ libre avec les modèles connus en suggestion
  modelField(f,p){
    const esc=this.ctx.esc, m=this.models[f.provider];
    if(m&&m.list){
      const opts=m.list.includes(f.model)?m.list:[f.model,...m.list];
      return `<select data-f="model">${opts.map(id=>`<option value="${esc(id)}"${id===f.model?' selected':''}>${esc(id)}${m.list.includes(id)?'':LT(' (indisponible)',' (unavailable)')}</option>`).join('')}</select>`;
    }
    return `<input data-f="model" list="cl-models" value="${esc(f.model)}"><datalist id="cl-models">${p.models.map(x=>`<option value="${esc(x)}">`).join('')}</datalist>`;
  },
  modelNote(f){
    const esc=this.ctx.esc, m=this.models[f.provider];
    if(!m) return '';
    if(m.loading) return `<span class="note">${LT('Chargement des modèles…','Loading models…')}</span>`;
    if(m.error) return `<span class="note err">${esc(m.error)}</span>`;
    if(m.list&&!m.list.includes(f.model)) return `<span class="note err">${LT("Ce modèle n'est plus proposé : choisis-en un autre",'This model is no longer offered: pick another one')}</span>`;
    return `<span class="note">${LT(`${m.list.length} modèle${m.list.length>1?'s':''} disponible${m.list.length>1?'s':''}`,`${m.list.length} model${m.list.length>1?'s':''} available`)}</span>`;
  },
  async loadModels(provider,key){
    this.models[provider]={loading:true};
    this.ctx.update();
    try{
      const list=await this.ctx.invoke('ai_list_models',{provider,apiKey:key||null});
      this.models[provider]={list};
      // Le modèle choisi n'existe plus : on prend le premier proposé, à confirmer en enregistrant
      if(this.form&&this.form.provider===provider&&!list.includes(this.form.model)&&!this.form.touched) this.form.model=list[0];
    }catch(e){ this.models[provider]={error:String(e)}; }
    this.ctx.update();
  },

  async go(){
    const text=this.draft.trim();
    if(!text){ this.status={text:LT("Colle d'abord un texte",'Paste some text first'),kind:'err'}; return this.ctx.update(); }
    const before=this.draft;
    try{ const out=await this.ctx.invoke('ai_run',{text}); this.prev=before; this.draft=out; }
    catch(e){ this.status={text:String(e),kind:'err'}; }
    this.ctx.update();
  },

  async fromClipboard(){
    try{ const r=await this.ctx.invoke('ai_apply_clipboard'); this.prev=r.input; this.draft=r.output; }
    catch(e){ this.status={text:String(e),kind:'err'}; }
    this.ctx.update();
  },

  async save(){
    const f=this.form;
    try{
      this.v=await this.ctx.invoke('ai_save',{s:{provider:f.provider,model:f.model,api_key:f.key?f.key:null,hotkey:f.hotkey,auto_paste:f.auto}});
      this.form=null; this.settings=false; this.status={text:LT('Réglages enregistrés','Settings saved'),kind:'ok'};
      this.release();
    }catch(e){ f.msg={text:String(e),kind:'err'}; }
    this.ctx.update();
  },

  mount(el){
    const ctx=this.ctx;
    const press=(sel,fn)=>el.querySelectorAll(sel).forEach(n=>n.addEventListener('mousedown',ev=>{ if(ev.button!==0) return; ev.preventDefault(); ev.stopPropagation(); fn(n); }));
    press('.cl-chip',n=>{ ctx.invoke('ai_select_action',{name:n.dataset.action}).then(v=>{ this.v=v; this.status=null; ctx.update(); }).catch(()=>{}); });
    press('[data-do="go"]',()=>this.go());
    press('[data-do="clip"]',()=>this.fromClipboard());
    press('[data-do="undo"]',()=>{ this.draft=this.prev; this.prev=null; this.status=null; ctx.update(); });
    press('[data-do="settings"]',()=>{ this.settings=!this.settings; this.form=null; ctx.update(); if(this.settings) this.loadModels(this.v.provider); });
    press('[data-do="models"]',()=>{ if(this.form) this.loadModels(this.form.provider,this.form.key); });
    press('[data-do="save"]',()=>this.save());
    // Où remettre le curseur : lu avant que le focus rendu ci-dessous ne le remplace
    const restore=this.editing&&this.focus?{...this.focus}:null;
    el.querySelectorAll('[data-f]').forEach(n=>{
      const f=n.dataset.f;
      // Un clic dans un champ emprunte le focus clavier au système
      n.addEventListener('mousedown',ev=>{ ev.stopPropagation(); this.grab(); });
      const keep=()=>{ this.focus={field:f,start:n.selectionStart,end:n.selectionEnd}; };
      const sync=()=>{
        if(f==='text') this.draft=n.value;
        else if(this.form){
          if(f==='auto') this.form.auto=n.checked; else this.form[f]=n.value;
          if(f==='model') this.form.touched=true;
          if(f==='provider'){ const p=this.v.providers.find(x=>x.id===n.value); this.form.model=p?p.model:''; this.form.key=''; this.form.touched=false; ctx.update(); if(!this.models[n.value]||this.models[n.value].error) this.loadModels(n.value); }
        }
        keep();
      };
      n.addEventListener('input',sync); n.addEventListener('change',sync);
      n.addEventListener('keyup',keep); n.addEventListener('mouseup',keep);
      n.addEventListener('keydown',ev=>{
        if(ev.key==='Escape'){ n.blur(); this.release(); ctx.update(); }
        else if(f==='text'&&ev.key==='Enter'&&ev.ctrlKey){ ev.preventDefault(); this.go(); }
        else if(f!=='text'&&f!=='auto'&&ev.key==='Enter'){ ev.preventDefault(); this.save(); }
      });
      // Quitter tous les champs (clic ailleurs, autre appli) rend le focus et laisse la card se fermer
      n.addEventListener('blur',()=>setTimeout(()=>{
        const a=document.activeElement;
        if(this.editing&&!(a&&a.dataset&&a.dataset.f&&el.contains(a))) this.release();
      },120));
    });
    // La card est redessinée par d'autres événements : on remet le focus et la sélection où ils étaient
    if(restore){
      const n=el.querySelector(`[data-f="${restore.field}"]`);
      if(n){ n.focus({preventScroll:true}); try{ if(restore.start!=null) n.setSelectionRange(restore.start,restore.end); }catch(_){} this.focus=restore; }
    }
  },
});
})();

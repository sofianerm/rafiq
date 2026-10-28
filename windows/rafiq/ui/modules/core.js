/* Notch modules: cells in the pill that are not usage providers (clipboard, notes, …).
   Each module lives in its own file next to this one, calls registerModule(), and is loaded by a
   <script src> line in notch.html. notch.html draws the ring, the text under it and the hover card;
   the module only says what goes in them. See README.md in this folder for the contract. */
const NOTCH_MODULES=[];

function registerModule(m){
  if(!m||typeof m.id!=='string'||!/^[a-z0-9-]+$/.test(m.id)){ console.error('registerModule: invalid id',m&&m.id); return; }
  if(NOTCH_MODULES.some(x=>x.id===m.id)){ console.error('registerModule: duplicate id',m.id); return; }
  m.name=m.name||m.id;
  m.glyph=m.glyph||m.name.slice(0,2);
  NOTCH_MODULES.push(m);
}

// `icon` may be a fixed SVG string, or a function for an icon that changes (the weather)
function moduleIcon(m){ return typeof m.icon==='function'?moduleCall(m,'icon'):m.icon; }

// A module that throws must not take the notch down with it: the error is logged and the call yields undefined
function moduleCall(m,fn,...args){
  if(typeof m[fn]!=='function') return undefined;
  try{ return m[fn](...args); }
  catch(e){ console.error(`module ${m.id}.${fn}:`,e); return undefined; }
}

// Le choix des Settings (notch_modules) : [{id,on}] dans l'ordre voulu. Un module absent de la liste
// (ajouté depuis) suit, affiché, dans l'ordre des <script src>. Partagé par notch.html et settings.html.
function arrangeModules(prefs){
  const out=[];
  for(const p of Array.isArray(prefs)?prefs:[]){
    const m=NOTCH_MODULES.find(x=>p&&x.id===p.id);
    if(m&&!out.some(o=>o.module===m)) out.push({module:m,on:p.on!==false});
  }
  for(const m of NOTCH_MODULES) if(!out.some(o=>o.module===m)) out.push({module:m,on:true});
  return out;
}

// Les liens « Données : … » des cards, que les sources demandent d'afficher. mousedown : la card est
// redessinée souvent. La commande open_link n'ouvre que les pages de sources connues (setup.rs).
function bindSourceLinks(el,ctx){
  el.querySelectorAll('[data-link]').forEach(a=>a.addEventListener('mousedown',ev=>{
    ev.stopPropagation(); ctx.invoke('open_link',{url:a.dataset.link}).catch(()=>{});
  }));
}

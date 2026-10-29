/* Temps d'écran du jour, par app, calculé en local par src/screentime.rs (rien ne sort du PC). Sous
   l'anneau, le total d'aujourd'hui ; au survol, le top des apps, la comparaison avec hier et les 7
   derniers jours. Le temps ne compte pas après 5 min sans souris ni clavier. */
(()=>{
const TOP=6;
// Quelques noms plus lisibles que la description de l'exe
const NICE={code:'VS Code',chrome:'Chrome',msedge:'Edge',firefox:'Firefox',explorer:'Explorer',
  windowsterminal:'Terminal',discord:'Discord',spotify:'Spotify',slack:'Slack',teams:'Teams','ms-teams':'Teams',
  claude:'Claude',cursor:'Cursor',whatsapp:'WhatsApp',applicationframehost:'Apps Windows'};
const COLORS=['#00FF88','#4DA3FF','#F2FF00','#FF8A3D','#C77DFF','#808080'];

function dur(s){
  const m=Math.round(s/60);
  if(m<1) return '<1 min';
  if(m<60) return m+' min';
  return Math.floor(m/60)+'h'+String(m%60).padStart(2,'0');
}
const dayKey=d=>{ const x=new Date(d); return `${x.getFullYear()}-${String(x.getMonth()+1).padStart(2,'0')}-${String(x.getDate()).padStart(2,'0')}`; };
const sum=o=>Object.values(o||{}).reduce((a,b)=>a+b,0);

const STYLE=`
.st-total{font-size:28px;font-weight:600;letter-spacing:-.5px}
.st-cmp{font-size:11px;color:#808080;margin-bottom:8px}
.st-cmp .up{color:#FF8A3D}
.st-bar{display:flex;height:6px;border-radius:3px;overflow:hidden;background:#2d2d2d;margin:6px 0 8px}
.st-row{display:grid;grid-template-columns:8px 1fr auto;column-gap:8px;align-items:center;font-size:12px;padding:3px 0}
.st-row .d{width:8px;height:8px;border-radius:2px}
.st-row .n{color:#e8e8ea;white-space:nowrap;overflow:hidden;text-overflow:ellipsis}
.st-row .t{color:#c8c8c8;font-variant-numeric:tabular-nums}
.st-week{display:grid;grid-template-columns:repeat(7,1fr);gap:4px;align-items:end;height:64px;margin-top:12px;border-top:1px solid #1e1e1e;padding-top:8px}
.st-week div{display:flex;flex-direction:column;align-items:center;justify-content:end;height:100%;font-size:9px;color:#606060}
.st-week i{display:block;width:100%;max-width:18px;background:#3a3a3a;border-radius:2px 2px 0 0;margin-bottom:3px;min-height:2px}
.st-week .today i{background:#00FF88}.st-week .today{color:#e8e8ea}
`;

registerModule({
  id:'screentime',
  get name(){ return LT("Temps d'écran",'Screen time'); },
  get title(){ return LT("Temps d'écran",'Screen time'); },
  glyph:'⏱',
  icon:'<svg viewBox="0 0 24 24"><path d="M3 5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2v10a2 2 0 0 1-2 2h-6v2h3v2H8v-2h3v-2H5a2 2 0 0 1-2-2zm2 0v10h14V5zm7 1.5a3.5 3.5 0 1 1 0 7 3.5 3.5 0 0 1 0-7zm-.7 1.3v2.5l1.9 1.2.7-1-1.3-.8V7.8z"/></svg>',
  data:{days:{},names:{}},

  init(ctx){
    this.ctx=ctx;
    const st=document.createElement('style'); st.textContent=STYLE; document.head.appendChild(st);
    const take=d=>{ if(d&&d.days){ this.data=d; ctx.update(); } };
    ctx.listen('screentime',e=>take(e.payload)).catch(()=>{});
    ctx.invoke('get_screentime').then(take).catch(()=>{});
  },

  today(){ return this.data.days[dayKey(Date.now())]||{}; },
  label(app){ return NICE[app]||this.data.names[app]||app; },

  badge(){ return dur(sum(this.today())); },

  render(){
    const esc=this.ctx.esc, today=this.today(), total=sum(today);
    const yesterday=sum(this.data.days[dayKey(Date.now()-86400000)]);
    let html=`<div class="st-total">${dur(total)}</div>`;
    if(yesterday){
      const diff=total-yesterday;
      html+=`<div class="st-cmp">${LT('Hier : ','Yesterday: ')}${dur(yesterday)} ${LT('au total','in total')}${diff>=60?` · <span class="up">${LT('déjà','already')} +${dur(diff)}</span>`:''}</div>`;
    }else html+=`<div class="st-cmp">${LT("Aujourd'hui, hors inactivité",'Today, idle time left out')}</div>`;
    const apps=Object.entries(today).sort((a,b)=>b[1]-a[1]);
    if(!apps.length) return html+`<div class="c-note">${LT("Pas encore de mesure aujourd'hui.",'Nothing measured yet today.')}</div>`;
    const top=apps.slice(0,TOP-1), rest=apps.slice(TOP-1).reduce((a,[,s])=>a+s,0);
    const rows=top.map(([app,s])=>[this.label(app),s]); if(rest) rows.push([LT('Autres','Others'),rest]);
    html+=`<div class="st-bar">`+rows.map(([,s],i)=>`<span style="width:${(s/total*100).toFixed(1)}%;background:${COLORS[i]}"></span>`).join('')+`</div>`;
    html+=rows.map(([n,s],i)=>`<div class="st-row"><span class="d" style="background:${COLORS[i]}"></span><span class="n">${esc(n)}</span><span class="t">${dur(s)}</span></div>`).join('');
    // Les 7 derniers jours
    const week=[...Array(7)].map((_,i)=>{ const t=Date.now()-(6-i)*86400000; return {t,s:sum(this.data.days[dayKey(t)])}; });
    const max=Math.max(...week.map(w=>w.s),1);
    html+=`<div class="st-week">`+week.map((w,i)=>`<div class="${i===6?'today':''}" title="${dur(w.s)}"><i style="height:${Math.round(w.s/max*40)}px"></i>${new Date(w.t).toLocaleDateString(moduleLocale(),{weekday:'narrow'})}</div>`).join('')+`</div>`;
    return html;
  },
});
})();

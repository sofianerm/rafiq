/* Prochains départs des arrêts choisis dans le profil (commande get_setup / événement `setup`), depuis
   l'une de deux sources :
   - Transitous (MOTIS 2, monde entier), lu côté Rust (src/transit.rs : User-Agent, cache) ; ses
     conditions demandent un lien vers ses sources et l'attribution OpenStreetMap, en bas de la card ;
   - transport.opendata.ch (Suisse, sans clé, CORS ouvert), lu directement.
   Sous l'anneau, le prochain départ ; au survol, un tableau par arrêt. Les minutes sont recalculées
   localement à partir des horaires (retards compris), donc peu de requêtes : toutes les 3 min (5 min
   pour Transitous), 45 s si la card est ouverte. */
(()=>{
const API='https://transport.opendata.ch/v1/stationboard';
const ROWS=6;
const IDLE_MS=3*60*1000, IDLE_TRANSITOUS_MS=5*60*1000, OPEN_MS=45*1000;
const MODE_COLORS={TRAM:'#F7A600',SUBWAY:'#7A3E9D',BUS:'#0078BF',COACH:'#0078BF',FERRY:'#00A0B0'};

// L'API écrit +0200 : Date.parse veut +02:00
const ms=v=>{ if(!v) return null; const t=Date.parse(String(v).replace(/([+-]\d\d)(\d\d)$/,'$1:$2')); return Number.isFinite(t)?t:null; };
const hhmm=t=>new Date(t).toLocaleTimeString('fr-FR',{hour:'2-digit',minute:'2-digit'});
const minutes=d=>Math.max(0,Math.floor(d/60000));
function modeColor(mode){ return MODE_COLORS[mode]||(/RAIL|TRAIN|LONG_DISTANCE|HIGHSPEED/.test(mode)?'#E3000F':'#0078BF'); }
function lineColor(cat,num){
  if(cat==='T') return '#F7A600';                  // tram
  if(/^L\d/.test(num)) return '#00A651';           // Léman Express
  if(['R','RE','IR','IC','S','EC','TGV'].includes(cat)) return '#E3000F'; // CFF
  return '#0078BF';                                // bus
}

const STYLE=`
.tp-st{font-size:11px;font-weight:700;color:#808080;text-transform:uppercase;letter-spacing:.4px;margin:12px 0 4px;display:flex;justify-content:space-between}
.tp-st:first-of-type{margin-top:4px}
.tp-st small{font-weight:400;text-transform:none;letter-spacing:0}
.tp-row{display:grid;grid-template-columns:34px 1fr auto;column-gap:8px;align-items:center;font-size:12px;padding:3px 0}
.tp-line{font-weight:700;font-size:11px;color:#000;border-radius:4px;text-align:center;padding:1px 0}
.tp-to{color:#e8e8ea;white-space:nowrap;overflow:hidden;text-overflow:ellipsis}
.tp-to small{color:#808080;margin-left:4px}
.tp-in{color:#e8e8ea;font-weight:600;font-variant-numeric:tabular-nums;text-align:right;white-space:nowrap}
.tp-in.now{color:#00FF88}
.tp-late{color:#FF3F00;font-size:10px;margin-left:3px}
`;

registerModule({
  id:'tpg',
  name:'Transports',
  title:'Départs',
  glyph:'T',
  icon:'<svg viewBox="0 0 24 24"><path d="M8 2h8v2h-3v2h2.5A3.5 3.5 0 0 1 19 9.5v7a2.5 2.5 0 0 1-1.6 2.3L19 21v1h-2.2l-1.5-2H8.7l-1.5 2H5v-1l1.6-2.2A2.5 2.5 0 0 1 5 16.5v-7A3.5 3.5 0 0 1 8.5 6H11V4H8zM7.5 9a.5.5 0 0 0-.5.5V13h10V9.5a.5.5 0 0 0-.5-.5zM8 15.2a1.2 1.2 0 1 0 0 2.4 1.2 1.2 0 0 0 0-2.4zm8 0a1.2 1.2 0 1 0 0 2.4 1.2 1.2 0 0 0 0-2.4z"/></svg>',
  boards:{},   // id → [{color,num,to,at,delay,platform}]
  stops:[],
  source:'opendata',
  error:'',
  fetchedAt:0,

  init(ctx){
    this.ctx=ctx;
    const st=document.createElement('style'); st.textContent=STYLE; document.head.appendChild(st);
    const take=s=>{
      if(!s) return;
      this.source=s.transit_source==='opendata'?'opendata':'transitous';
      this.stops=Array.isArray(s.transit_stops)?s.transit_stops:[];
      this.title=this.stops.length?'Départs · '+this.stops[0].name:'Départs';
      this.boards={}; this.fetchedAt=0; this.load();
    };
    ctx.listen('setup',e=>take(e.payload)).catch(()=>{});
    ctx.invoke('get_setup').then(take).catch(()=>{});
    let last='';
    setInterval(()=>{
      const age=Date.now()-this.fetchedAt;
      const idle=this.source==='transitous'?IDLE_TRANSITOUS_MS:IDLE_MS;
      if(this.stops.length&&age>(ctx.visible()?OPEN_MS:idle)) this.load();
      const key=this.badge();
      if(key!==last||ctx.visible()){ last=key; ctx.update(); }
    },5000);
  },

  async load(){
    if(this.loading) return;
    this.loading=true; this.fetchedAt=Date.now();
    try{
      const source=this.source;
      await Promise.all(this.stops.map(async s=>{
        if(source==='transitous'){
          const deps=await this.ctx.invoke('transit_departures',{stopId:s.id});
          this.boards[s.id]=(deps||[]).filter(d=>!d.cancelled).map(d=>({
            color:d.color||modeColor(d.mode),num:d.line,to:d.to,at:d.at,delay:d.delay,platform:d.track,
          }));
          return;
        }
        const r=await fetch(`${API}?id=${encodeURIComponent(s.id)}&limit=20`);
        if(!r.ok) throw new Error('HTTP '+r.status);
        const j=await r.json();
        this.boards[s.id]=(j.stationboard||[]).map(e=>{
          const st=e.stop||{}, planned=ms(st.departure), real=ms(st.prognosis&&st.prognosis.departure)||planned;
          const cat=e.category||'', num=e.number||'';
          return {color:lineColor(cat,num),num:num||cat,to:e.to||'',at:real,delay:real&&planned?Math.round((real-planned)/60000):0,platform:st.platform||''};
        }).filter(d=>d.at).sort((a,b)=>a.at-b.at); // par heure réelle, retards compris
      }));
      this.error='';
    }catch(e){ this.error=String(e&&e.message||e); }
    this.loading=false;
    this.ctx.update();
  },

  upcoming(id){ const t=Date.now()-30000; return (this.boards[id]||[]).filter(d=>d.at>=t); },

  badge(){
    const next=this.stops.flatMap(s=>this.upcoming(s.id)).sort((a,b)=>a.at-b.at)[0];
    if(!next) return this.error?'!':'—';
    const m=minutes(next.at-Date.now());
    return `${m} min`;
  },

  onClick(){ this.fetchedAt=0; this.load(); },

  render(){
    const esc=this.ctx.esc;
    if(!this.stops.length) return `<div class="c-note">Choisis tes arrêts dans les Settings → Profil.</div>`;
    let html='';
    for(const s of this.stops){
      html+=`<div class="tp-st">${esc(s.name)}<small>${esc(s.sub)}</small></div>`;
      const deps=this.upcoming(s.id).slice(0,ROWS);
      if(!deps.length){ html+=`<div class="c-sub">${this.fetchedAt?'Aucun départ':'Chargement…'}</div>`; continue; }
      for(const d of deps){
        const m=minutes(d.at-Date.now());
        const inTxt=m<1?'maintenant':m<60?`${m} min`:hhmm(d.at);
        html+=`<div class="tp-row"><span class="tp-line" style="background:${esc(d.color)}">${esc(d.num)}</span>`
          +`<span class="tp-to">${esc(d.to)}${d.platform?`<small>${esc(/^\d+$/.test(d.platform)?'voie '+d.platform:'quai '+d.platform)}</small>`:''}</span>`
          +`<span class="tp-in${m<1?' now':''}">${inTxt}${d.delay>0?`<span class="tp-late">+${d.delay}</span>`:''}</span></div>`;
      }
    }
    if(this.error) html+=`<div class="c-sub" style="margin-top:8px">Erreur : ${esc(this.error)}</div>`;
    if(this.source==='transitous') html+=`<div class="c-sub src-link">Données : <span data-link="https://transitous.org/sources/">Transitous et ses sources</span> · <span data-link="https://www.openstreetmap.org/copyright">© OpenStreetMap</span></div>`;
    return html;
  },

  mount(el){ bindSourceLinks(el,this.ctx); },
});
})();

/* Météo de la ville choisie à l'accueil (Open-Meteo, sans clé, CORS ouvert ; la ville vient du profil,
   commande get_setup / événement `setup`). Sous l'anneau, la température ; l'icône suit le temps qu'il
   fait ; l'anneau se remplit en bleu avec le risque de pluie des 3 prochaines heures. Au survol :
   conditions actuelles, les prochaines heures et les 4 jours. Relu toutes les 15 min. */
(()=>{
const urlFor=c=>'https://api.open-meteo.com/v1/forecast?latitude='+c.lat+'&longitude='+c.lon
  +'&current=temperature_2m,apparent_temperature,weather_code,is_day,wind_speed_10m,relative_humidity_2m'
  +'&hourly=temperature_2m,precipitation_probability,weather_code,is_day'
  +'&daily=weather_code,temperature_2m_max,temperature_2m_min,precipitation_probability_max,sunrise,sunset'
  +'&timezone=auto&forecast_days=4';
const EVERY=15*60*1000;
const RAIN='#4DA3FF';

// Codes WMO → [libellé, famille d'icône]
function describe(code){
  if(code===0) return ['Ciel clair','clear'];
  if(code===1) return ['Plutôt dégagé','clear'];
  if(code===2) return ['Partiellement nuageux','partly'];
  if(code===3) return ['Couvert','cloud'];
  if(code===45||code===48) return ['Brouillard','fog'];
  if(code>=51&&code<=57) return ['Bruine','rain'];
  if(code>=61&&code<=67) return [code>=65?'Forte pluie':code>=63?'Pluie':'Pluie faible','rain'];
  if(code>=71&&code<=77) return ['Neige','snow'];
  if(code>=80&&code<=82) return ['Averses','rain'];
  if(code===85||code===86) return ['Averses de neige','snow'];
  if(code>=95) return ['Orage','storm'];
  return ['—','cloud'];
}
const CLOUD='<path d="M7 19a4.5 4.5 0 0 1-.7-8.95 6 6 0 0 1 11.6 1.5A3.75 3.75 0 0 1 17.5 19z"/>';
const SUN_RAYS='<g fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><path d="M12 2.5v2M12 19.5v2M2.5 12h2M19.5 12h2M5.3 5.3l1.4 1.4M17.3 17.3l1.4 1.4M5.3 18.7l1.4-1.4M17.3 6.7l1.4-1.4"/></g>';
const ICONS={
  sun:`<svg viewBox="0 0 24 24"><circle cx="12" cy="12" r="4.5"/>${SUN_RAYS}</svg>`,
  moon:'<svg viewBox="0 0 24 24"><path d="M14.5 3a8.5 8.5 0 1 0 6.5 13.9A7 7 0 0 1 14.5 3z"/></svg>',
  partly:`<svg viewBox="0 0 24 24"><g transform="translate(-3 -4) scale(.7)"><circle cx="12" cy="12" r="4.5"/>${SUN_RAYS}</g><g transform="translate(2 3) scale(.9)">${CLOUD}</g></svg>`,
  cloud:`<svg viewBox="0 0 24 24"><g transform="translate(0 -1)">${CLOUD}</g></svg>`,
  rain:`<svg viewBox="0 0 24 24"><g transform="translate(0 -4)">${CLOUD}</g><g fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round"><path d="M8 17.5l-1 3M12 17.5l-1 3M16 17.5l-1 3"/></g></svg>`,
  snow:`<svg viewBox="0 0 24 24"><g transform="translate(0 -4)">${CLOUD}</g><circle cx="8" cy="19" r="1.2"/><circle cx="12" cy="21" r="1.2"/><circle cx="16" cy="19" r="1.2"/></svg>`,
  storm:`<svg viewBox="0 0 24 24"><g transform="translate(0 -4)">${CLOUD}</g><path d="M12.5 14l-3 5h2.5l-1 4 4-6h-2.5l1.5-3z"/></svg>`,
  fog:'<svg viewBox="0 0 24 24"><g fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><path d="M4 8h16M3 12h18M5 16h14M7 20h10"/></g></svg>',
};
function iconOf(code,isDay){
  const f=describe(code)[1];
  if(f==='clear') return isDay?ICONS.sun:ICONS.moon;
  if(f==='partly'&&!isDay) return ICONS.cloud;
  return ICONS[f]||ICONS.cloud;
}
const hh=iso=>String(iso).slice(11,13)+'h';
const deg=v=>Math.round(v)+'°';

const STYLE=`
.mt-now{display:flex;align-items:center;gap:12px;margin:2px 0 4px}
.mt-now .ic svg{width:40px;height:40px;fill:#e8e8ea;display:block}
.mt-now .t{font-size:30px;font-weight:600;letter-spacing:-.5px}
.mt-now .d{font-size:12px;color:#c8c8c8}
.mt-meta{font-size:11px;color:#808080;margin-bottom:6px}
.mt-alert{font-size:12px;color:${RAIN};margin:6px 0}
.mt-hours{display:grid;grid-template-columns:repeat(6,1fr);gap:2px;margin:8px 0 4px;text-align:center;font-size:11px}
.mt-hours svg,.mt-days svg{width:18px;height:18px;fill:#e8e8ea;display:block;margin:2px auto}
.mt-hours .h{color:#808080}.mt-hours .p{color:${RAIN};font-size:10px;min-height:12px}
.mt-days{margin-top:8px;border-top:1px solid #1e1e1e;padding-top:6px}
.mt-day{display:grid;grid-template-columns:1fr 22px 40px 64px;column-gap:6px;align-items:center;font-size:12px;padding:2px 0;font-variant-numeric:tabular-nums}
.mt-day .n{color:#e8e8ea;text-transform:capitalize}.mt-day .p{color:${RAIN};text-align:right;font-size:11px}.mt-day .r{text-align:right;color:#e8e8ea}.mt-day .r small{color:#808080}
`;

registerModule({
  id:'meteo',
  name:'Météo',
  title:'Météo',
  glyph:'°',
  city:null,
  data:null,
  error:'',
  fetchedAt:0,

  icon(){ const c=this.data&&this.data.current; return c?iconOf(c.weather_code,c.is_day):ICONS.cloud; },

  init(ctx){
    this.ctx=ctx;
    const st=document.createElement('style'); st.textContent=STYLE; document.head.appendChild(st);
    const take=s=>{
      const c=s&&s.city, same=c&&this.city&&c.lat===this.city.lat&&c.lon===this.city.lon;
      this.city=c||null; this.title=c?'Météo · '+c.name:'Météo';
      if(!same){ this.data=null; this.load(); }
    };
    ctx.listen('setup',e=>take(e.payload)).catch(()=>{});
    ctx.invoke('get_setup').then(take).catch(()=>{});
    setInterval(()=>{ if(Date.now()-this.fetchedAt>EVERY) this.load(); },60000);
  },

  async load(){
    if(!this.city){ this.ctx.update(); return; }
    this.fetchedAt=Date.now();
    try{
      const r=await fetch(urlFor(this.city));
      if(!r.ok) throw new Error('HTTP '+r.status);
      this.data=await r.json(); this.error='';
    }catch(e){ this.error=String(e&&e.message||e); }
    this.ctx.update();
  },

  // Les heures à venir, à partir de l'heure en cours
  hours(n){
    const h=this.data&&this.data.hourly; if(!h) return [];
    const nowIso=(this.data.current&&this.data.current.time||'').slice(0,13);
    let i=h.time.findIndex(t=>t.slice(0,13)>=nowIso); if(i<0) i=0;
    return h.time.slice(i,i+n).map((t,k)=>({t,temp:h.temperature_2m[i+k],p:h.precipitation_probability[i+k],code:h.weather_code[i+k],day:h.is_day[i+k]}));
  },

  badge(){ const c=this.data&&this.data.current; return c?deg(c.temperature_2m):(this.error?'!':'—'); },

  progress(){
    const next=this.hours(3); if(!next.length) return null;
    return {value:Math.max(...next.map(h=>h.p||0))/100,color:RAIN};
  },

  onClick(){ this.load(); },

  render(){
    const esc=this.ctx.esc;
    if(!this.city) return `<div class="c-note">Choisis ta ville dans les Settings → Profil.</div>`;
    if(!this.data) return `<div class="c-note">${esc(this.error?'Météo indisponible : '+this.error:'Chargement…')}</div>`;
    const c=this.data.current, d=this.data.daily, [label]=describe(c.weather_code);
    let html=`<div class="mt-now"><span class="ic">${iconOf(c.weather_code,c.is_day)}</span><div><div class="t">${deg(c.temperature_2m)}</div><div class="d">${esc(label)}</div></div></div>`;
    html+=`<div class="mt-meta">Ressenti ${deg(c.apparent_temperature)} · Vent ${Math.round(c.wind_speed_10m)} km/h · Humidité ${Math.round(c.relative_humidity_2m)} %</div>`;
    // Le premier créneau où la pluie devient probable dans les 12 h
    const wet=this.hours(12).find(h=>h.p>=50);
    if(wet) html+=`<div class="mt-alert">☂ Pluie probable vers ${hh(wet.t)} (${wet.p} %)</div>`;
    html+=`<div class="mt-hours">`+this.hours(6).map(h=>`<div><div class="h">${hh(h.t)}</div>${iconOf(h.code,h.day)}<div>${deg(h.temp)}</div><div class="p">${h.p?h.p+' %':''}</div></div>`).join('')+`</div>`;
    html+=`<div class="mt-days">`+d.time.map((t,i)=>{
      const name=i===0?"Aujourd'hui":new Date(t+'T12:00').toLocaleDateString('fr-FR',{weekday:'long'});
      return `<div class="mt-day"><span class="n">${name}</span>${iconOf(d.weather_code[i],1)}<span class="p">${d.precipitation_probability_max[i]?d.precipitation_probability_max[i]+' %':''}</span><span class="r"><small>${deg(d.temperature_2m_min[i])}</small> ${deg(d.temperature_2m_max[i])}</span></div>`;
    }).join('')+`</div>`;
    html+=`<div class="c-sub" style="margin-top:8px">Soleil ${d.sunrise[0].slice(11,16)} → ${d.sunset[0].slice(11,16)}${this.error?' · '+esc(this.error):''}</div>`;
    html+=`<div class="c-sub src-link">Données : <span data-link="https://open-meteo.com/">Open-Meteo.com</span></div>`;
    return html;
  },

  mount(el){ bindSourceLinks(el,this.ctx); },
});
})();

/* Horaires de prière (ceux d'une mosquée Masjidbox, ou calculés par Aladhan pour la ville du profil) : décompte jusqu'à la prochaine prière sous l'anneau, l'anneau se
   remplit entre deux prières, et pulse entre l'adhan et l'iqama. La card montre la journée entière.
   Les données viennent de src/prayer.rs (événement `prayer`, commandes get_prayer / refresh_prayer). */
(()=>{
const NAMES=()=>LT({fajr:'Fajr',sunrise:'Chourouq',dhuhr:'Dohr',asr:'Asr',maghrib:'Maghrib',isha:'Icha',jumuah:'Joumouâa'},
  {fajr:'Fajr',sunrise:'Sunrise',dhuhr:'Dhuhr',asr:'Asr',maghrib:'Maghrib',isha:'Isha',jumuah:'Jumu‘ah'});
// Le nom affiché, dans la langue du moment (les événements gardent seulement la clé)
const lbl=e=>NAMES()[e.key]+(e.n?' '+e.n:'');
const ORDER=['fajr','sunrise','dhuhr','asr','maghrib','isha'];
const CHECKABLE=['fajr','dhuhr','asr','maghrib','isha'];
const SOON=15*60*1000; // l'anneau passe au jaune dans le dernier quart d'heure

const ms=v=>{ const t=Date.parse(v); return Number.isFinite(t)?t:null; };
const hhmm=t=>new Date(t).toLocaleTimeString(moduleLocale(),{hour:'2-digit',minute:'2-digit'});
const sameDay=(a,b)=>new Date(a).toDateString()===new Date(b).toDateString();

// Décompte court pour sous l'anneau (70 px de large) : 2h05, 45 min, puis 8:42 sur les dix dernières minutes
function shortLeft(d){
  const s=Math.max(0,Math.round(d/1000)), m=Math.floor(s/60);
  if(m>=60) return `${Math.floor(m/60)}h${String(m%60).padStart(2,'0')}`;
  if(m>=10) return `${m} min`;
  return `${m}:${String(s%60).padStart(2,'0')}`;
}
function longLeft(d){
  const s=Math.max(0,Math.round(d/1000)), h=Math.floor(s/3600), m=Math.floor(s/60)%60, x=s%60;
  return (h?`${h}:${String(m).padStart(2,'0')}`:String(m))+':'+String(x).padStart(2,'0');
}

// Tous les moments des jours connus, dans l'ordre. `prayer` = compte pour le décompte (Chourouq non)
function eventsOf(days){
  const out=[];
  for(const d of days||[]){
    const iq=d.iqamah||{};
    for(const k of ORDER){
      const at=ms(d[k]); if(at==null) continue;
      out.push({key:k,at,iqamah:ms(iq[k]),prayer:k!=='sunrise',day:d});
    }
    (Array.isArray(d.jumuah)?d.jumuah:[]).forEach((v,i,all)=>{
      const at=ms(v); if(at==null) return;
      const iqs=Array.isArray(iq.jumuah)?iq.jumuah:[];
      out.push({key:'jumuah',n:all.length>1?i+1:0,at,iqamah:ms(iqs[i]),prayer:true,day:d});
    });
  }
  return out.sort((a,b)=>a.at-b.at);
}

const STYLE=`
.pr-sub{font-size:11px;color:#808080;margin-bottom:10px}
.pr-next{font-size:12px;color:#c8c8c8}
.pr-count{font-size:28px;font-weight:600;font-variant-numeric:tabular-nums;letter-spacing:-.5px;margin:2px 0 10px}
.pr-count.soon{color:#F2FF00}
.pr-day{font-size:11px;font-weight:700;color:#808080;text-transform:uppercase;letter-spacing:.4px;margin:10px 0 4px}
.pr-row{display:grid;grid-template-columns:10px 1fr auto auto 20px;column-gap:8px;align-items:center;font-size:12px;padding:3px 0;font-variant-numeric:tabular-nums}
.pr-row .n{color:#e8e8ea;font-weight:600}
.pr-row .a{color:#e8e8ea;text-align:right}
.pr-row .i{color:#808080;text-align:right;min-width:34px}
.pr-row .d{width:6px;height:6px;border-radius:3px}
.pr-row.past{opacity:.4}
.pr-row.next .d{background:#00FF88}
.pr-row.next .n,.pr-row.next .a{color:#00FF88}
.pr-head{font-size:10px;color:#606060}
.pr-chk{width:16px;height:16px;border-radius:8px;border:1.5px solid #555;background:none;padding:0;cursor:pointer;color:#000;font:700 10px/13px "Segoe UI";justify-self:end}
.pr-chk.on{background:#00FF88;border-color:#00FF88}
.pr-chk:hover{border-color:#aaa}
.pr-row.past.done{opacity:.55}
.pr-foot{display:flex;justify-content:space-between;align-items:center;margin-top:10px}
.pr-test{font-size:10px;color:#808080;cursor:pointer;text-decoration:underline}
.pr-hijri{font-family:Cairo,"Segoe UI",sans-serif;font-size:12px;color:#b0b0b3;direction:rtl}
`;

registerModule({
  id:'prayer',
  get name(){ return LT('Prière','Prayer'); },
  get title(){ return LT('Horaires de prière','Prayer times'); },
  glyph:'☾',
  icon:'<svg viewBox="0 0 24 24"><path d="M15.2 2.6a9.6 9.6 0 1 0 6.3 14.6A7.9 7.9 0 0 1 15.2 2.6z"/><path d="M19 3.2l.6 1.6 1.7.1-1.3 1.1.4 1.6-1.4-.9-1.4.9.4-1.6-1.3-1.1 1.7-.1z"/></svg>',
  snap:{status:'loading',days:[],name:'',note:''},
  events:[],
  done:new Set(), // "AAAA-MM-JJ:dhuhr" des prières cochées : leur rappel ne part pas

  init(ctx){
    this.ctx=ctx;
    const st=document.createElement('style'); st.textContent=STYLE; document.head.appendChild(st);
    const take=s=>{ if(s){ this.snap=s; this.events=eventsOf(s.days); ctx.update(); } };
    ctx.listen('prayer',e=>take(e.payload)).catch(()=>{});
    ctx.invoke('get_prayer').then(take).catch(()=>{});
    const takeDone=l=>{ this.done=new Set(Array.isArray(l)?l:[]); ctx.update(); };
    ctx.listen('prayer_done',e=>takeDone(e.payload)).catch(()=>{});
    ctx.invoke('get_prayed').then(takeDone).catch(()=>{});
    // Chaque seconde, mais on ne redessine que si ce qui s'affiche change (ou si la card est ouverte)
    let last='';
    setInterval(()=>{
      const key=[this.badge(),JSON.stringify(this.progress()),this.activity()].join('|');
      if(key!==last||ctx.visible()){ last=key; ctx.update(); }
    },1000);
  },

  // Où on en est : la dernière prière passée et la prochaine
  now(){
    const t=Date.now(), ps=this.events.filter(e=>e.prayer);
    const next=ps.find(e=>e.at>t)||null;
    let prev=null; for(const e of ps){ if(e.at<=t) prev=e; else break; }
    // Entre l'adhan et l'iqama de la prière qui vient de commencer
    const iqama=prev&&prev.iqamah&&prev.iqamah>t?prev:null;
    return {t,next,prev,iqama};
  },

  badge(){
    const {t,next,iqama}=this.now();
    if(iqama) return 'Iqama';
    if(!next) return this.snap.status==='error'?'!':'—';
    return shortLeft(next.at-t);
  },

  progress(){
    const {t,next,prev}=this.now();
    if(!next||!prev) return null;
    return {value:(t-prev.at)/(next.at-prev.at),color:next.at-t<=SOON?'#F2FF00':'#00FF88'};
  },

  activity(){ return this.now().iqama?'attention':null; },

  // mousedown plutôt que click : la card est redessinée chaque seconde, un click à cheval serait perdu
  mount(el){
    el.querySelectorAll('.pr-chk').forEach(b=>b.addEventListener('mousedown',ev=>{
      ev.stopPropagation();
      const key=b.dataset.key, on=!this.done.has(key);
      on?this.done.add(key):this.done.delete(key); this.ctx.update();
      this.ctx.invoke('set_prayed',{key,done:on}).catch(()=>{});
    }));
    bindSourceLinks(el,this.ctx);
    const test=el.querySelector('.pr-test');
    if(test) test.addEventListener('mousedown',ev=>{ ev.stopPropagation(); this.ctx.invoke('prayer_test_notify').catch(()=>{}); });
  },

  onClick(){ this.ctx.invoke('refresh_prayer').catch(()=>{}); },

  render(){
    const esc=this.ctx.esc, s=this.snap;
    const calc=String(s.slug||'').startsWith('aladhan:');
    let html=`<div class="pr-sub">${esc(s.name||(calc?LT('Horaires calculés','Calculated times'):'Masjidbox'))}${calc&&s.address?' · '+esc(s.address):''}${s.status==='stale'&&s.fetched_at?` · ${LT('mis à jour le','updated')} ${new Date(s.fetched_at).toLocaleDateString(moduleLocale())}`:''}</div>`;
    if(!this.events.length){
      return html+`<div class="c-note">${esc(s.status==='error'?LT('Impossible de lire les horaires : ','Could not read the times: ')+(s.note||''):LT('Chargement des horaires…','Loading the times…'))}</div>`;
    }
    const {t,next,prev,iqama}=this.now();
    if(iqama){
      html+=`<div class="pr-next">${LT('Iqama de '+esc(lbl(iqama))+' dans','Iqama of '+esc(lbl(iqama))+' in')}</div><div class="pr-count soon">${longLeft(iqama.iqamah-t)}</div>`;
    }else if(next){
      html+=`<div class="pr-next">${LT(esc(lbl(next))+' à '+hhmm(next.at)+' dans',esc(lbl(next))+' at '+hhmm(next.at)+' in')}</div><div class="pr-count${next.at-t<=SOON?' soon':''}">${longLeft(next.at-t)}</div>`;
    }
    // La journée de la prière en cours : celle d'Icha reste affichée jusqu'à Fajr, pour pouvoir la cocher
    const ref=prev?prev.day:(next?next.day:null), refAt=ms(ref&&ref.date)||t;
    const day=this.events.filter(e=>e.day===ref);
    const when=sameDay(refAt,t)?LT("Aujourd'hui",'Today'):refAt<t?LT('Hier','Yesterday'):LT('Demain','Tomorrow');
    html+=`<div class="pr-day">${when} · ${new Date(refAt).toLocaleDateString(moduleLocale(),{weekday:'long',day:'numeric',month:'long'})}</div>`;
    const iqamas=day.some(e=>e.iqamah);
    html+=`<div class="pr-row pr-head"><span></span><span></span><span class="a">Adhan</span><span class="i">${iqamas?'Iqama':''}</span><span class="i" title="${LT('Prié','Prayed')}">✓</span></div>`;
    for(const e of day){
      const key=CHECKABLE.includes(e.key)?e.day.date.slice(0,10)+':'+e.key:null, on=key&&this.done.has(key);
      const cls=(e===next?'next':e.at<=t&&e!==iqama?'past':'')+(on?' done':'');
      // Une prière ne se coche qu'une fois son heure arrivée
      const chk=key&&e.at<=t?`<button class="pr-chk${on?' on':''}" data-key="${key}" title="${on?LT('Décocher','Untick'):LT('Marquer comme priée','Mark as prayed')}">${on?'✓':''}</button>`:'<span></span>';
      html+=`<div class="pr-row ${cls}"><span class="d"></span><span class="n">${esc(lbl(e))}</span><span class="a">${hhmm(e.at)}</span><span class="i">${e.iqamah?hhmm(e.iqamah):''}</span>${chk}</div>`;
    }
    const hijri=day[0]&&day[0].day.hijri&&day[0].day.hijri.formatted;
    html+=`<div class="pr-foot"><span class="pr-test" title="${LT("Envoie une notification d'essai",'Sends a test notification')}">${LT('Tester une notif','Test a notification')}</span>${hijri?`<span class="pr-hijri">${esc(hijri)}</span>`:''}</div>`;
    if(s.status!=='ok'&&s.note) html+=`<div class="c-sub" style="margin-top:6px">${esc(s.note)}</div>`;
    if(calc) html+=`<div class="c-sub src-link">${LT('Horaires','Times')} : <span data-link="https://aladhan.com/">Aladhan</span></div>`;
    return html;
  },
});
})();

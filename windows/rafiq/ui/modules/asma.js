/* Un des 99 noms d'Allah (al-asmâ' al-husnâ) par jour, dans l'ordre de la liste rapportée par
   at-Tirmidhî. Sous l'anneau, le nom en arabe ; au survol, l'arabe, la translittération et le sens.
   Un clic sur l'anneau (ou ‹ ›) passe au suivant ; « Aujourd'hui » revient au nom du jour. */
(()=>{
const NAMES=[
  ['الرَّحْمَنُ','Ar-Rahmân','Le Tout Miséricordieux'],
  ['الرَّحِيمُ','Ar-Rahîm','Le Très Miséricordieux'],
  ['الْمَلِكُ','Al-Malik','Le Roi, le Souverain'],
  ['الْقُدُّوسُ','Al-Quddûs','Le Pur, le Saint'],
  ['السَّلَامُ','As-Salâm','La Paix, Celui qui est exempt de tout défaut'],
  ['الْمُؤْمِنُ','Al-Mu\'min','Celui qui accorde la sécurité'],
  ['الْمُهَيْمِنُ','Al-Muhaymin','Celui qui veille sur tout'],
  ['الْعَزِيزُ','Al-\'Azîz','Le Tout-Puissant, l\'Invincible'],
  ['الْجَبَّارُ','Al-Jabbâr','Le Contraignant, Celui qui répare'],
  ['الْمُتَكَبِّرُ','Al-Mutakabbir','Le Suprême dans Sa grandeur'],
  ['الْخَالِقُ','Al-Khâliq','Le Créateur'],
  ['الْبَارِئُ','Al-Bâri\'','Celui qui donne l\'existence'],
  ['الْمُصَوِّرُ','Al-Musawwir','Celui qui donne forme'],
  ['الْغَفَّارُ','Al-Ghaffâr','Celui qui pardonne abondamment'],
  ['الْقَهَّارُ','Al-Qahhâr','Le Dominateur suprême'],
  ['الْوَهَّابُ','Al-Wahhâb','Le Donateur généreux'],
  ['الرَّزَّاقُ','Ar-Razzâq','Celui qui pourvoit à la subsistance'],
  ['الْفَتَّاحُ','Al-Fattâh','Celui qui ouvre, le Juge'],
  ['الْعَلِيمُ','Al-\'Alîm','L\'Omniscient'],
  ['الْقَابِضُ','Al-Qâbid','Celui qui restreint'],
  ['الْبَاسِطُ','Al-Bâsit','Celui qui étend Ses dons'],
  ['الْخَافِضُ','Al-Khâfid','Celui qui abaisse'],
  ['الرَّافِعُ','Ar-Râfi\'','Celui qui élève'],
  ['الْمُعِزُّ','Al-Mu\'izz','Celui qui honore'],
  ['الْمُذِلُّ','Al-Mudhill','Celui qui humilie'],
  ['السَّمِيعُ','As-Samî\'','Celui qui entend tout'],
  ['الْبَصِيرُ','Al-Basîr','Celui qui voit tout'],
  ['الْحَكَمُ','Al-Hakam','L\'Arbitre'],
  ['الْعَدْلُ','Al-\'Adl','Le Juste'],
  ['اللَّطِيفُ','Al-Latîf','Le Subtil, le Bienveillant'],
  ['الْخَبِيرُ','Al-Khabîr','Le Parfaitement Informé'],
  ['الْحَلِيمُ','Al-Halîm','Le Longanime, le Clément'],
  ['الْعَظِيمُ','Al-\'Azîm','L\'Immense, le Magnifique'],
  ['الْغَفُورُ','Al-Ghafûr','Le Pardonneur'],
  ['الشَّكُورُ','Ash-Shakûr','Le Reconnaissant'],
  ['الْعَلِيُّ','Al-\'Aliyy','Le Très-Haut'],
  ['الْكَبِيرُ','Al-Kabîr','Le Grand'],
  ['الْحَفِيظُ','Al-Hafîz','Le Gardien'],
  ['الْمُقِيتُ','Al-Muqît','Celui qui nourrit, le Soutien'],
  ['الْحَسِيبُ','Al-Hasîb','Celui qui tient compte de tout'],
  ['الْجَلِيلُ','Al-Jalîl','Le Majestueux'],
  ['الْكَرِيمُ','Al-Karîm','Le Généreux'],
  ['الرَّقِيبُ','Ar-Raqîb','L\'Observateur vigilant'],
  ['الْمُجِيبُ','Al-Mujîb','Celui qui exauce'],
  ['الْوَاسِعُ','Al-Wâsi\'','Celui qui englobe tout'],
  ['الْحَكِيمُ','Al-Hakîm','Le Sage'],
  ['الْوَدُودُ','Al-Wadûd','Le Bien-Aimant'],
  ['الْمَجِيدُ','Al-Majîd','Le Glorieux'],
  ['الْبَاعِثُ','Al-Bâ\'ith','Celui qui ressuscite'],
  ['الشَّهِيدُ','Ash-Shahîd','Le Témoin'],
  ['الْحَقُّ','Al-Haqq','La Vérité'],
  ['الْوَكِيلُ','Al-Wakîl','Le Garant, Celui à qui l\'on s\'en remet'],
  ['الْقَوِيُّ','Al-Qawiyy','Le Fort'],
  ['الْمَتِينُ','Al-Matîn','L\'Inébranlable'],
  ['الْوَلِيُّ','Al-Waliyy','Le Protecteur, l\'Allié'],
  ['الْحَمِيدُ','Al-Hamîd','Le Digne de louange'],
  ['الْمُحْصِي','Al-Muhsî','Celui qui dénombre tout'],
  ['الْمُبْدِئُ','Al-Mubdi\'','Celui qui commence la création'],
  ['الْمُعِيدُ','Al-Mu\'îd','Celui qui la recommence'],
  ['الْمُحْيِي','Al-Muhyî','Celui qui donne la vie'],
  ['الْمُمِيتُ','Al-Mumît','Celui qui donne la mort'],
  ['الْحَيُّ','Al-Hayy','Le Vivant'],
  ['الْقَيُّومُ','Al-Qayyûm','Celui qui subsiste par Lui-même'],
  ['الْوَاجِدُ','Al-Wâjid','Celui qui trouve tout'],
  ['الْمَاجِدُ','Al-Mâjid','Le Noble'],
  ['الْوَاحِدُ','Al-Wâhid','L\'Unique'],
  ['الْأَحَدُ','Al-Ahad','L\'Un'],
  ['الصَّمَدُ','As-Samad','L\'Absolu, Celui dont tout dépend'],
  ['الْقَادِرُ','Al-Qâdir','Le Capable'],
  ['الْمُقْتَدِرُ','Al-Muqtadir','Le Tout-Puissant'],
  ['الْمُقَدِّمُ','Al-Muqaddim','Celui qui fait avancer'],
  ['الْمُؤَخِّرُ','Al-Mu\'akhkhir','Celui qui fait reculer'],
  ['الْأَوَّلُ','Al-Awwal','Le Premier'],
  ['الْآخِرُ','Al-Âkhir','Le Dernier'],
  ['الظَّاهِرُ','Az-Zâhir','L\'Apparent'],
  ['الْبَاطِنُ','Al-Bâtin','Le Caché'],
  ['الْوَالِي','Al-Wâlî','Celui qui gouverne'],
  ['الْمُتَعَالِي','Al-Muta\'âlî','Le Sublime'],
  ['الْبَرُّ','Al-Barr','Le Bienfaisant'],
  ['التَّوَّابُ','At-Tawwâb','Celui qui accueille le repentir'],
  ['الْمُنْتَقِمُ','Al-Muntaqim','Celui qui châtie'],
  ['الْعَفُوُّ','Al-\'Afuww','Celui qui efface les fautes'],
  ['الرَّؤُوفُ','Ar-Ra\'ûf','Le Très Bon, le Compatissant'],
  ['مَالِكُ الْمُلْكِ','Mâlik al-Mulk','Le Possesseur du Royaume'],
  ['ذُو الْجَلَالِ وَالْإِكْرَامِ','Dhul-Jalâli wal-Ikrâm','Le Détenteur de la Majesté et de la Générosité'],
  ['الْمُقْسِطُ','Al-Muqsit','L\'Équitable'],
  ['الْجَامِعُ','Al-Jâmi\'','Celui qui rassemble'],
  ['الْغَنِيُّ','Al-Ghaniyy','Le Riche, Celui qui se suffit à Lui-même'],
  ['الْمُغْنِي','Al-Mughnî','Celui qui enrichit'],
  ['الْمَانِعُ','Al-Mâni\'','Celui qui retient, le Protecteur'],
  ['الضَّارُّ','Ad-Dârr','Celui qui éprouve'],
  ['النَّافِعُ','An-Nâfi\'','Celui qui accorde le bien'],
  ['النُّورُ','An-Nûr','La Lumière'],
  ['الْهَادِي','Al-Hâdî','Le Guide'],
  ['الْبَدِيعُ','Al-Badî\'','Le Créateur sans modèle'],
  ['الْبَاقِي','Al-Bâqî','Celui qui demeure'],
  ['الْوَارِثُ','Al-Wârith','L\'Héritier'],
  ['الرَّشِيدُ','Ar-Rashîd','Celui qui guide sur la voie droite'],
  ['الصَّبُورُ','As-Sabûr','Le Patient'],
];
// Le sens en anglais, dans le même ordre
const EN=["The Most Compassionate", "The Most Merciful", "The King, the Sovereign", "The Most Holy", "The Source of Peace", "The Granter of Security", "The Guardian", "The Almighty", "The Compeller, the Restorer", "The Supreme in Greatness", "The Creator", "The Originator", "The Fashioner", "The Ever-Forgiving", "The Subduer", "The Bestower", "The Provider", "The Opener, the Judge", "The All-Knowing", "The Withholder", "The Extender", "The Abaser", "The Exalter", "The Honourer", "The Humiliator", "The All-Hearing", "The All-Seeing", "The Judge", "The Just", "The Subtle, the Gentle", "The All-Aware", "The Forbearing", "The Magnificent", "The All-Forgiving", "The Appreciative", "The Most High", "The Most Great", "The Preserver", "The Sustainer", "The Reckoner", "The Majestic", "The Generous", "The Watchful", "The Responsive", "The All-Encompassing", "The All-Wise", "The Loving", "The Most Glorious", "The Resurrector", "The Witness", "The Truth", "The Trustee", "The All-Strong", "The Firm", "The Protecting Friend", "The Praiseworthy", "The Accounter", "The Originator of creation", "The Restorer", "The Giver of Life", "The Taker of Life", "The Ever-Living", "The Self-Subsisting", "The Finder", "The Glorious", "The One", "The Unique", "The Eternal Refuge", "The All-Powerful", "The Determiner", "The Expediter", "The Delayer", "The First", "The Last", "The Manifest", "The Hidden", "The Governor", "The Most Exalted", "The Source of Goodness", "The Acceptor of Repentance", "The Avenger", "The Pardoner", "The Most Kind", "The Owner of Sovereignty", "The Lord of Majesty and Generosity", "The Equitable", "The Gatherer", "The Self-Sufficient", "The Enricher", "The Preventer", "The Distresser", "The Benefiter", "The Light", "The Guide", "The Incomparable Originator", "The Everlasting", "The Inheritor", "The Guide to the Right Path", "The Most Patient"];
// Le nom du jour : le même toute la journée, le suivant le lendemain
function todayIndex(){
  const d=new Date(), local=Date.UTC(d.getFullYear(),d.getMonth(),d.getDate());
  return Math.floor(local/86400000)%NAMES.length;
}
const bare=s=>s.replace(/[ً-ْٰ]/g,''); // sans voyelles, pour sous l'anneau

const STYLE=`
.as-ar{font-family:"Traditional Arabic","Arabic Typesetting","Segoe UI",sans-serif;font-size:40px;line-height:1.4;text-align:center;color:#fff;direction:rtl;margin:4px 0 2px}
.as-tr{text-align:center;font-size:15px;font-weight:700;color:#00FF88}
.as-fr{text-align:center;font-size:13px;color:#c8c8c8;margin-top:4px;line-height:1.4}
.as-nav{display:flex;justify-content:space-between;align-items:center;margin-top:14px;font-size:11px;color:#808080}
.as-nav button{font:inherit;color:#e8e8ea;background:#1c1c1c;border:1px solid #333;border-radius:6px;padding:3px 10px;cursor:pointer}
.as-nav button:hover{background:#2a2a2a}
.as-nav .today{background:none;border:none;text-decoration:underline;color:#808080;padding:0}
`;

registerModule({
  id:'asma',
  name:'Asmâ\' al-Husnâ',
  get title(){ return LT('Nom d\'Allah du jour','Name of Allah of the day'); },
  glyph:'۩',
  icon:'<svg viewBox="0 0 24 24"><path d="M12 1.5l2.6 3.6 4.4-.7-.7 4.4 3.6 2.6-3.6 2.6.7 4.4-4.4-.7L12 22.5l-2.6-3.6-4.4.7.7-4.4L2.1 12l3.6-2.6-.7-4.4 4.4.7zm0 5.5a5 5 0 1 0 0 10 5 5 0 0 0 0-10z"/></svg>',
  offset:0, // décalage par rapport au nom du jour, quand on feuillette

  init(ctx){
    this.ctx=ctx;
    const st=document.createElement('style'); st.textContent=STYLE; document.head.appendChild(st);
    // Le jour change à minuit : on revient au nom du jour
    let day=todayIndex();
    setInterval(()=>{ const t=todayIndex(); if(t!==day){ day=t; this.offset=0; ctx.update(); } },60000);
  },

  index(){ return ((todayIndex()+this.offset)%NAMES.length+NAMES.length)%NAMES.length; },

  badge(){
    const ar=bare(NAMES[this.index()][0]);
    return ar.length<=7?ar:'n°'+(this.index()+1);
  },

  onClick(){ this.offset++; this.ctx.update(); },

  render(){
    const i=this.index(), [ar,tr,frMeaning]=NAMES[i], fr=LT(frMeaning,EN[i]), esc=this.ctx.esc;
    return `<div class="as-ar">${ar}</div><div class="as-tr">${esc(tr)}</div><div class="as-fr">${esc(fr)}</div>`
      +`<div class="as-nav"><button class="prev" title="${LT('Précédent','Previous')}">‹</button>`
      +`<span>${i+1} / 99${this.offset?` · <button class="today">${LT("Aujourd'hui",'Today')}</button>`:''}</span>`
      +`<button class="next" title="${LT('Suivant','Next')}">›</button></div>`;
  },

  mount(el){
    const on=(sel,fn)=>{ const b=el.querySelector(sel); if(b) b.addEventListener('mousedown',ev=>{ ev.stopPropagation(); fn(); this.ctx.update(); }); };
    on('.prev',()=>this.offset--);
    on('.next',()=>this.offset++);
    on('.today',()=>this.offset=0);
  },
});
})();

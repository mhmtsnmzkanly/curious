/* Pure summaries for the recorded-life view. Never infers unrecorded perception or outcomes. */
(function (root, factory) {
  const api = factory();
  if (typeof module === "object" && module.exports) module.exports = api;
  else root.CuriousDocumentary = api;
})(typeof globalThis !== "undefined" ? globalThis : this, function () {
  "use strict";
  const finite = n => typeof n === "number" && Number.isFinite(n);
  const bands = ["düşük", "orta", "yüksek"];
  const healthNames = ["bilinmiyor", "tek vuruşluk", "iki vuruşluk", "ikiden fazla vuruşluk"];
  const band = (value, maximum) => finite(value) && finite(maximum) ? Math.min(2, Math.floor(value * 3 / Math.max(1, maximum))) : null;
  const distanceBand = n => !finite(n) ? null : n <= 1 ? 0 : n <= 3 ? 1 : 2;
  function huntContext(item) {
    const trace=item?.trace;
    if (!trace || !["HuntPrey","LearnedHunt","HuntLowEnergy","HuntTooCostly"].includes(trace.decision?.reason)) return null;
    const learner=item.actor?.hunt_learner;
    const ended=(item.events||[]).map(e=>e.HuntEpisodeEnded?.result).find(r=>r?.attempt?.context);
    // Only use a context the recorded learner itself stored. Perception alone
    // often omits prey health, and cannot recreate the learner's damage bands.
    const episode=learner?.episode?.context ? learner.episode : ended?.attempt;
    if(!episode?.context)return null;
    return {...episode.context,target:episode.target,ticks:episode.ticks,source:learner?.episode===episode?"active":"ended"};
  }
  function contextLabel(c) {
    if (!c) return "Karşılaşma bağlamı kayıtlı algıda yok.";
    return `enerji ${bands[c.energy] || "bilinmiyor"}, dayanıklılık ${bands[c.stamina] || "bilinmiyor"}, mesafe ${c.distance === null ? "bilinmiyor" : ["0–1","2–3","4+"][c.distance]} adım, av sağlığı ${healthNames[c.health] || "bilinmiyor"}`;
  }
  // Fixed matching rule: all four stored HuntContext bands must be known and
  // equal. Health band zero means unknown and therefore never matches.
  function sameContext(a,b) { return !!a && !!b && ["energy","stamina","distance","health"].every(k=>a[k]!==null && a[k]!==undefined && a[k]===b[k]) && a.health>0; }
  function ageOf(item) { return item.trace?.before?.age ?? item.actor?.life?.age ?? null; }
  function waterDelta(previous,current) {
    const before=previous?.actor?.learner?.values, after=current?.actor?.learner?.values;
    if(!Array.isArray(before)||!Array.isArray(after))return "Su Q farkı bilinmiyor: bu iki kaydın öğrenici tablolarından biri eksik.";
    const key=v=>v?.pos&&finite(v.pos.x)&&finite(v.pos.y)?`${v.pos.x},${v.pos.y}`:null;
    const old=new Map(before.map(v=>[key(v),v]).filter(([k])=>k!==null));
    const now=new Map(after.map(v=>[key(v),v]).filter(([k])=>k!==null));
    const changes=[];
    for(const [k,v] of now){const p=old.get(k);if(!p)changes.push(`(${k}) sınırlı tabloya yeni girdi`);else if(p.value!==v.value||p.visits!==v.visits)changes.push(`(${k}) Q Δ${finite(p.value)&&finite(v.value)?v.value-p.value:"bilinmiyor"}, ziyaret Δ${finite(p.visits)&&finite(v.visits)?v.visits-p.visits:"bilinmiyor"}`);}
    for(const k of old.keys())if(!now.has(k))changes.push(`(${k}) sınırlı tabloda artık görünmüyor`);
    return changes.length?`Kayıtlı su Q tablosu farkı: ${changes.slice(0,8).join("; ")}${changes.length>8?"; …":""}. Bu tablo farkı sonuç nedeni veya yaşam yararı kanıtı değildir.`:"Kayıtlı su Q tablosunda bu iki snapshot arasında fark yok (bu sonuç puanlanmış güncelleme yok anlamına gelmez).";
  }
  function encounterComparison(items, selectedItem) {
    const context = huntContext(selectedItem);
    if (!context) return {context:null,first:null,last:null,count:0};
    const groups=[];
    for(const item of items){
      const c=huntContext(item), prior=groups[groups.length-1];
      if(!c){if(prior)prior.closed=true;continue;}
      if(!sameContext(context,c)){if(prior)prior.closed=true;continue;}
      const continues=prior&&!prior.closed&&prior.target===c.target&&finite(prior.ticks)&&finite(c.ticks)&&c.ticks>prior.ticks;
      if(continues){prior.last=item;prior.ticks=c.ticks;if(c.source==="ended")prior.closed=true;}
      else {groups.push({first:item,last:item,target:c.target,ticks:c.ticks,closed:c.source==="ended"});}
    }
    return {context,first:groups[0]?.first||null,last:groups[groups.length-1]?.first||null,count:groups.length};
  }
  function displayData(item) {
    const t=item.trace||{}, d=t.decision||{}, p=t.perception, actor=item.actor, seen=t.seen;
    const events=Array.isArray(item.events)?item.events:[];
    const moves=events.filter(e=>e.EntityMoved), attacks=events.filter(e=>e.EntityAttacked?.attacker===t.id);
    const moved=moves.reduce((n,e)=>n+(finite(e.EntityMoved.energy_cost)?e.EntityMoved.energy_cost:0),0);
    const attacked=attacks.reduce((n,e)=>n+(finite(e.EntityAttacked.energy_cost)?e.EntityAttacked.energy_cost:0),0);
    const ends=events.filter(e=>e.HuntEpisodeEnded?.id===t.id).map(e=>e.HuntEpisodeEnded.result);
    const drank=events.filter(e=>e.EntityDrank?.id===t.id).map(e=>e.EntityDrank.amount);
    const target=t.target?.Entity ?? (t.target?.Position ? `(${t.target.Position.x}, ${t.target.Position.y})` : null);
    let perception;
    if (p) perception=`${p.entities?.length??0} canlı, ${p.foods?.length??0} yiyecek, ${p.waters?.length??0} su, ${p.shelters?.length??0} sığınak gördü.`;
    else if (seen) perception=`Özet kayıt: ${seen.entities?.length??0} canlı, ${seen.foods??"?"} yiyecek, ${seen.waters??"?"} su; ayrıntılı algı yok.`;
    else perception="Algı kaydı yok; ne gördüğü bilinmiyor.";
    const memory=actor?.memory;
    const memoryText=memory ? `Tick sonrası bellek: ${memory.sites?.length??0} hatırlanan kaynak, ${memory.dangers?.length??0} tehlike kaydı.` : "Bellek kaydı yok veya eski kayıt.";
    const risk=actor?.hunt_learner;
    const riskRows=Array.isArray(risk?.risk_values)?risk.risk_values.slice(0,8):[];
    const riskAttempt=risk?.episode??risk?.last_episode?.attempt;
    const riskEvidence={};for(const key of ["exposures","injuries","damage","prey_defense_damage"])if(Object.hasOwn(riskAttempt||{},key))riskEvidence[key]=riskAttempt[key];
    const riskStatus=risk?.risk_enabled===true?"Risk öğrenmesi açık":risk?.risk_enabled===false?"Risk öğrenmesi kapalı": "risk_enabled alanı yok (eski kayıt)";
    const riskTable=Array.isArray(risk?.risk_values)?`${risk.risk_values.length} kayıtlı risk bağlamı${riskRows.length?`: ${riskRows.map(v=>JSON.stringify(v)).join("; ")}`:""}`:"risk_values alanı yok";
    const riskText = `${riskStatus}; ${riskTable}${Object.keys(riskEvidence).length?`; kayıtlı episode riski ${JSON.stringify(riskEvidence)}`:""}.`;
    const huntResults=ends.map(r=>{const a=r.attempt||{};const defense=Object.hasOwn(a,"prey_defense_damage")?`av savunması hasarı ${a.prey_defense_damage}`:"av savunması hasarı kaydedilmemiş";return `${r.end??"sonuç bilinmiyor"}: ${a.movement_energy??"hareket maliyeti yok"} hareket + ${a.attack_energy??"saldırı maliyeti yok"} saldırı enerjisi, ${a.food??"besin miktarı yok"} besin, gelen hasar ${a.incoming_damage??"bilinmiyor"}, ${defense}${a.ambiguous===true?", nedensellik belirsiz":""}${r.reward==null?"; puan yok":`; puan ${r.reward}`}`;});
    const traceNote=item.trace_available===false?" Karar izi bulunamadı veya boyut sınırını aştı; gerekçe/algı bilinmiyor.":"";
    const partial=item.events_truncated===true;
    return {id:t.id??item.entity_id,shard:item.shard,tick:item.tick,age:ageOf(item),beforeAge:t.before?.age??null,reason:d.reason||"Bilinmiyor",perception,memory:memoryText,target,
      cost:`Bu kayıtta hareket ${partial||moves.some(e=>!finite(e.EntityMoved.energy_cost))?"eksik/eski olay var; toplam bilinmiyor":`${moved} enerji`}, saldırı ${partial||attacks.some(e=>!finite(e.EntityAttacked.energy_cost))?"eksik/eski olay var; toplam bilinmiyor":`${attacked} enerji`} (yalnız olayda yazan resolver masrafı).`,
      outcome:[drank.length?`Resolver olayı: ${drank.reduce((a,b)=>a+b,0)} birim su içti.`:"",ends.length?`Av episode sonucu: ${huntResults.join("; ")}.`:""].filter(Boolean).join(" ") || (actor?.learner?.last_episode ? "Son su denemesi alanı var; bu kayıt için yeni sonuç olayı yok." : "Bu tickte öğrenme sonucu olayı yok."),
      learned:ends.length?`Besin/enerji Q tablosu güncellemesi: ${ends.map(r=>r.updated===true?"güncellendi":r.updated===false?"güncellenmedi":"bilinmiyor").join(", ")}. Risk örnekleri ayrı ölçümlerdir; bu olay alanı risk güncellemesini belirtmez.`:
        "Bu tickte besin/enerji Q güncelleme olayı yok. Son deneme alanı tek başına yeni güncelleme kanıtı değildir.",risk:riskText,
      identity:`#${t.id??item.entity_id} · shard ${item.shard??"?"} / kayıt ${item.id} · tick ${item.tick} · karar öncesi yaş ${t.before?.age??"bilinmiyor"} · kayıt sonrası yaş ${actor?.life?.age??"birey kayıtta yok"}${traceNote}`,
      context:huntContext(item),decision:d,actor,events};
  }
  return {huntContext,contextLabel,sameContext,encounterComparison,displayData,ageOf,waterDelta};
});

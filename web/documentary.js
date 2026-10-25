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
  function riskSampleLabel(value) {
    const parts=[];
    if(finite(value?.exposures))parts.push(`${value.exposures} riskli karşılaşma`);
    if(finite(value?.injuries))parts.push(`${value.injuries} yaralanma`);
    if(finite(value?.damage))parts.push(`${value.damage} kayıtlı hasar`);
    return `${contextLabel(value?.context)}${parts.length?` · ${parts.join(", ")}`:""}`;
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
  function causalChain(item) {
    if (!item || typeof item !== "object") {
      return {
        evidence: "Unknown (kayıtta yok)",
        action: "Unknown (kayıtta yok)",
        resolverOutcome: "Unknown (kayıtta yok)",
        memoryUpdate: "Unknown (kayıtta yok)",
      };
    }
    const t = item.trace;
    const actor = item.actor;
    const events = Array.isArray(item.events) ? item.events : null;
    const actorId = t?.id ?? item.entity_id;

    // 1. Evidence: observed perception or remembered context at decision time
    let evidence;
    if (!t || (!t.perception && !t.seen)) {
      evidence = "Unknown (kayıtta yok)";
    } else if (t.perception) {
      const p = t.perception;
      evidence = `${p.entities?.length ?? 0} canlı, ${p.foods?.length ?? 0} yiyecek, ${p.waters?.length ?? 0} su, ${p.shelters?.length ?? 0} sığınak görüldü.`;
    } else {
      const seen = t.seen;
      evidence = `Özet algı: ${seen.entities?.length ?? 0} canlı, ${seen.foods ?? "?"} yiyecek, ${seen.waters ?? "?"} su.`;
    }

    // 2. Action: chosen reason and intent
    let action;
    if (!t || !t.decision || (!t.decision.reason && !t.decision.intent)) {
      action = "Unknown (kayıtta yok)";
    } else {
      const d = t.decision;
      const intentKey = d.intent && typeof d.intent === "object" ? Object.keys(d.intent)[0] : null;
      action = `Gerekçe: ${d.reason || "bilinmiyor"}${intentKey ? `, niyet: ${intentKey}` : ""}.`;
    }

    // 3. Resolver outcome: executed events and physical costs for this actor
    let resolverOutcome;
    if (!events || events.length === 0) {
      resolverOutcome = "Unknown (kayıtta yok)";
    } else {
      const actorEvents = events.filter(e => {
        if (e.EntityMoved && (actorId == null || e.EntityMoved.id === actorId)) return true;
        if (e.EntityAttacked && (actorId == null || e.EntityAttacked.attacker === actorId)) return true;
        if (e.EntityDrank && (actorId == null || e.EntityDrank.id === actorId)) return true;
        if (e.EntityAte && (actorId == null || e.EntityAte.id === actorId)) return true;
        if (e.HuntEpisodeEnded && (actorId == null || e.HuntEpisodeEnded.id === actorId)) return true;
        if (e.EntityDamaged && (actorId == null || e.EntityDamaged.id === actorId)) return true;
        if (e.EntityDied && (actorId == null || e.EntityDied.id === actorId)) return true;
        if (e.EntityMated && (actorId == null || e.EntityMated.parent1 === actorId || e.EntityMated.parent2 === actorId)) return true;
        return false;
      });
      if (actorEvents.length === 0) {
        resolverOutcome = "Unknown (kayıtta yok)";
      } else {
        const parts = [];
        for (const e of actorEvents) {
          if (e.EntityMoved) parts.push(`hareket (${finite(e.EntityMoved.energy_cost) ? `${e.EntityMoved.energy_cost} enerji` : "maliyet kayıtta yok"})`);
          if (e.EntityAttacked) parts.push(`saldırı (${finite(e.EntityAttacked.energy_cost) ? `${e.EntityAttacked.energy_cost} enerji` : "maliyet kayıtta yok"})`);
          if (e.EntityDrank) parts.push(`su içti (${finite(e.EntityDrank.amount) ? `${e.EntityDrank.amount} birim` : "miktar kayıtta yok"})`);
          if (e.EntityAte) parts.push(`besin aldı (${finite(e.EntityAte.amount) ? `${e.EntityAte.amount} birim` : "miktar kayıtta yok"})`);
          if (e.HuntEpisodeEnded) parts.push(`av sonlandı (${e.HuntEpisodeEnded.result?.end || "sonuç bilinmiyor"})`);
          if (e.EntityDamaged) parts.push(`hasar aldı (${finite(e.EntityDamaged.amount) ? e.EntityDamaged.amount : "bilinmiyor"})`);
          if (e.EntityDied) parts.push("yaşam sona erdi");
          if (e.EntityMated) parts.push("üreme gerçekleşti");
        }
        resolverOutcome = parts.length ? parts.join(", ") : "Unknown (kayıtta yok)";
      }
    }

    // 4. Memory update: observed memory or learner update
    let memoryUpdate;
    if (!actor) {
      memoryUpdate = "Unknown (kayıtta yok)";
    } else {
      const updates = [];
      const huntEnd = events?.find(e => e.HuntEpisodeEnded && (actorId == null || e.HuntEpisodeEnded.id === actorId));
      if (huntEnd?.HuntEpisodeEnded?.result?.updated === true) {
        updates.push("av Q tablosu güncellendi");
      }
      const drank = events?.find(e => e.EntityDrank && (actorId == null || e.EntityDrank.id === actorId));
      if (drank && actor.learner?.enabled) {
        updates.push("su denemesi sonuçlandı");
      }
      if (actor.memory?.refuge_outcomes_enabled && Array.isArray(actor.memory.refuge_outcomes) && actor.memory.refuge_outcomes.length > 0) {
        updates.push(`sığınak tablosunda ${actor.memory.refuge_outcomes.length} kayıt`);
      }
      memoryUpdate = updates.length ? updates.join("; ") : "Unknown (kayıtta yok)";
    }

    return { evidence, action, resolverOutcome, memoryUpdate };
  }

  function autoDirector(items) {
    if (!Array.isArray(items) || items.length === 0) return null;

    let best = null;

    for (const item of items) {
      if (!item || typeof item !== "object") continue;
      const t = item.trace;
      const actor = item.actor;
      const events = Array.isArray(item.events) ? item.events : [];
      const actorId = t?.id ?? item.entity_id;
      const reason = t?.decision?.reason;
      const life = actor?.life;
      const health = life?.health;
      const maxHealth = life?.max_health;
      const isLowHealth = finite(health) && finite(maxHealth) && maxHealth > 0 && health <= Math.floor(maxHealth / 4);
      const tookCombatDamage = events.some(e => e.EntityDamaged && (actorId == null || e.EntityDamaged.id === actorId));

      let priority = 0;
      let category = "routine_observation";
      let headline = `#${actorId ?? "?"} olağan gözlem kaydı (tick ${item.tick ?? "?"})`;

      // Priority 4: critical_escape
      const isEscapeOrShelter = reason === "SeekShelter" || reason === "EscapeThreat" || (typeof reason === "string" && (reason.includes("Shelter") || reason.includes("Escape") || reason.includes("Flee")));
      if (isEscapeOrShelter && (isLowHealth || tookCombatDamage)) {
        priority = 4;
        category = "critical_escape";
        headline = `#${actorId ?? "?"} kritik kaçışta: ${isLowHealth ? "düşük sağlık" : "hasar aldı"} (${reason})`;
      } else if (events.some(e => e.EntityAttacked || e.HuntEpisodeEnded)) {
        // Priority 3: active_predation
        priority = 3;
        category = "active_predation";
        const atk = events.find(e => e.EntityAttacked);
        const end = events.find(e => e.HuntEpisodeEnded);
        headline = `#${actorId ?? "?"} aktif av/saldırı olayında: ${atk ? "saldırı hamlesi" : end?.HuntEpisodeEnded?.result?.end || "av sonlanışı"}`;
      } else if (events.some(e => e.EntityMated || e.EntityBorn)) {
        // Priority 2: lineage_birth
        priority = 2;
        category = "lineage_birth";
        headline = `#${actorId ?? "?"} soy aktarımı / doğum olayı kaydedildi`;
      } else if (["LearnedWater", "LearnedHunt", "RecallWater", "RecallFood", "RecallMate"].includes(reason)) {
        // Priority 1: learned_resource
        priority = 1;
        category = "learned_resource";
        headline = `#${actorId ?? "?"} bireysel bellek/öğrenme kararı: ${reason}`;
      }

      if (!best || priority > best.priority) {
        best = { focalItem: item, category, headline, priority };
        if (priority === 4) break; // Highest possible priority found
      }
    }

    return best;
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
    const riskEvidence=[];
    if(finite(riskAttempt?.exposures))riskEvidence.push(`${riskAttempt.exposures} riskli karşılaşma`);
    if(finite(riskAttempt?.injuries))riskEvidence.push(`${riskAttempt.injuries} yaralanma`);
    if(finite(riskAttempt?.damage))riskEvidence.push(`${riskAttempt.damage} kayıtlı hasar`);
    if(finite(riskAttempt?.prey_defense_damage))riskEvidence.push(`avın verdiği hasar ${riskAttempt.prey_defense_damage}`);
    const riskStatus=risk?.risk_enabled===true?"Risk öğrenmesi açık":risk?.risk_enabled===false?"Risk öğrenmesi kapalı": "Eski kayıtta risk öğrenmesinin durumu bilinmiyor";
    const riskTable=Array.isArray(risk?.risk_values)?`${risk.risk_values.length} kayıtlı risk örneği${riskRows.length?`: ${riskRows.map(riskSampleLabel).join("; ")}`:""}`:"Risk örnekleri bu kayıtta yok";
    const riskText = `${riskStatus}; ${riskTable}${riskEvidence.length?`; kayıtlı karşılaşma: ${riskEvidence.join(", ")}`:""}.`;
    const huntResults=ends.map(r=>{const a=r.attempt||{};const defense=Object.hasOwn(a,"prey_defense_damage")?`av savunması hasarı ${a.prey_defense_damage}`:"av savunması hasarı kaydedilmemiş";return `${r.end??"sonuç bilinmiyor"}: ${a.movement_energy??"hareket maliyeti yok"} hareket + ${a.attack_energy??"saldırı maliyeti yok"} saldırı enerjisi, ${a.food??"besin miktarı yok"} besin, gelen hasar ${a.incoming_damage??"bilinmiyor"}, ${defense}${a.ambiguous===true?", nedensellik belirsiz":""}${r.reward==null?"; puan yok":`; puan ${r.reward}`}`;});
    const traceNote=item.trace_available===false?" Karar izi bulunamadı veya boyut sınırını aştı; gerekçe/algı bilinmiyor.":"";
    const partial=item.events_truncated===true;
    return {id:t.id??item.entity_id,shard:item.shard,tick:item.tick,age:ageOf(item),beforeAge:t.before?.age??null,reason:d.reason||"Bilinmiyor",perception,memory:memoryText,target,
      cost:`Bu kayıtta hareket ${partial||moves.some(e=>!finite(e.EntityMoved.energy_cost))?"eksik/eski olay var; toplam bilinmiyor":`${moved} enerji`}, saldırı ${partial||attacks.some(e=>!finite(e.EntityAttacked.energy_cost))?"eksik/eski olay var; toplam bilinmiyor":`${attacked} enerji`} (yalnız olayda yazan resolver masrafı).`,
      outcome:[drank.length?`Resolver olayı: ${drank.reduce((a,b)=>a+b,0)} birim su içti.`:"",ends.length?`Av episode sonucu: ${huntResults.join("; ")}.`:""].filter(Boolean).join(" ") || (actor?.learner?.last_episode ? "Son su denemesi alanı var; bu kayıt için yeni sonuç olayı yok." : "Bu tickte öğrenme sonucu olayı yok."),
      learned:ends.length?`Besin/enerji Q tablosu güncellemesi: ${ends.map(r=>r.updated===true?"güncellendi":r.updated===false?"güncellenmedi":"bilinmiyor").join(", ")}. Risk örnekleri ayrı ölçümlerdir; bu olay alanı risk güncellemesini belirtmez.`:
        "Bu tickte besin/enerji Q güncelleme olayı yok. Son deneme alanı tek başına yeni güncelleme kanıtı değildir.",risk:riskText,
      identity:`#${t.id??item.entity_id} · shard ${item.shard??"?"} / kayıt ${item.id} · tick ${item.tick} · karar öncesi yaş ${t.before?.age??"bilinmiyor"} · kayıt sonrası yaş ${actor?.life?.age??"birey kayıtta yok"}${traceNote}`,
      context:huntContext(item),decision:d,actor,events,causalChain:causalChain(item)};
  }
  return {huntContext,contextLabel,sameContext,encounterComparison,displayData,ageOf,waterDelta,causalChain,autoDirector};
});

"use strict";
const assert = require("node:assert/strict");
const experience = require("../web/experience.js");
const text = individual => experience.describe(individual).join("\n");
const attempt = {target:{x:3,y:4},ticks:5,movement_energy:7,incoming_damage:11};
const reached = {learner:{enabled:true,values:[{pos:attempt.target,visits:2,value:155}],episode:null,last_episode:{attempt,water:20,end:"Reached",reward:181}}};
assert.match(text(reached), /Su içti/);
assert.match(text(reached), /2 puanlanmış deneme/);
assert.match(text(reached), /tercih değeri 155/);
assert.match(text(reached), /11 \(su puanından düşülmez\)/);
assert.match(text(reached), /başarı sayısı değildir/);
const interrupted = structuredClone(reached);
interrupted.learner.last_episode = {attempt,water:0,end:"Interrupted",reward:null};
assert.match(text(interrupted), /Kesildi:/);
assert.match(text(interrupted), /Puan güncellemesi yok/);
assert.doesNotMatch(text(interrupted), /Deneme puanı:/);
const failed=structuredClone(interrupted);
failed.learner.last_episode.end="Failed";failed.learner.last_episode.reward=-29;
assert.match(text(failed), /Kaynak bulunamadı/);
assert.match(text(failed), /Deneme puanı: -29/);
assert.match(text({learner:{enabled:false,values:[]}}), /öğrenme: kapalı/);
assert.match(text({learner:{values:[]}}), /öğrenme: veri yok/);
assert.match(text({learner:{enabled:true,values:[]}}), /tercih tablosu boş/);
assert.match(text({genome:{mobility:5},life:{age:900},generation:40}), /su deneyimi verisi yok/);
assert.doesNotMatch(text({genome:{mobility:5},life:{age:900}}), /deneyim: 900/);
assert.match(text({learner:{enabled:true,episode:{target:{x:2,y:1},ticks:3,movement_energy:9}}}), /Henüz sonuçlanmadı/);
assert.doesNotThrow(()=>text({learner:{last_episode:{end:"FutureEnd"},values:[null]}}));
assert.match(text({learner:{last_episode:{end:"FutureEnd"}}}), /Sonuç türü bilinmiyor/);
assert.match(experience.describe(null).join(), /birey seç/);
assert.match(experience.describe(reached,{decision:{reason:"LearnedWater"}}).join(), /keşif ile en yüksek/);
// Rendering uses textContent; untrusted/unknown archive values never become markup.
const nodes=[];
const container={replaceChildren(){nodes.length=0;},append(p){nodes.push(p);},ownerDocument:{createElement(){return {textContent:""};}}};
experience.render(container,{learner:{enabled:true,values:[{pos:{x:1,y:2},visits:3,value:"<img onerror=alert(1)>"}]}});
assert.ok(nodes.length>0);
assert.ok(nodes.every(p=>!Object.hasOwn(p,"innerHTML")));
assert.ok(nodes.some(p=>p.textContent.includes("tercih değeri veri yok")));
console.log("Experience presenter: reached/interrupted/failed/missing/sparse/unknown/genetics/render checks passed");
const huntAttempt={target:19,context:{energy:0,stamina:2,distance:1,health:0},ticks:4,movement_energy:8,attack_energy:3,incoming_damage:0,food:12,own_kill:true,ambiguous:false};
const hunter={hunt_learner:{enabled:true,update_enabled:false,values:[{context:huntAttempt.context,visits:7,value:62}],episode:null,last_episode:{attempt:huntAttempt,end:"Fed",reward:62,updated:false}}};
assert.match(text(hunter), /Kendi avından besin aldı/);
assert.match(text(hunter), /tablo sabit tutulur/);
assert.match(text(hunter), /görünür av sağlığı bilinmiyor/);
assert.match(text(hunter), /7 puanlanmış deneme/);
assert.match(text(hunter), /8 hareket \+ 3 saldırı enerjisi/);
assert.match(text(hunter), /Tablo güncellendi: hayır/);
const uncertain=structuredClone(hunter);
uncertain.hunt_learner.last_episode.attempt.ambiguous=true;
uncertain.hunt_learner.last_episode.end="ForeignKill";
uncertain.hunt_learner.last_episode.reward=null;
assert.match(text(uncertain), /Avı başka birey öldürdü/);
assert.match(text(uncertain), /sonuçtan başarı çıkarılamaz/);
assert.match(text(uncertain), /eksik puan başarısızlık değildir/);
assert.doesNotMatch(text(uncertain), /Son av denemesi: Kendi avından/);
const budgetCutoff=structuredClone(hunter);
budgetCutoff.hunt_learner.last_episode={attempt:{...huntAttempt,incoming_damage:0},end:"Interrupted",reward:-19,updated:true};
assert.match(text(budgetCutoff), /Karar veya hedef değişti; varsa puan ayrıca gösterilir/);
assert.match(text(budgetCutoff), /Deneme puanı: -19/);
assert.match(text({hunt_learner:{enabled:false,values:[],episode:null,last_episode:null}}), /Av seçimini öğrenme: kapalı/);
assert.doesNotThrow(()=>text({hunt_learner:{values:[null],last_episode:{attempt:{},end:"FutureEnd"}}}));
assert.match(text({learner:{enabled:true,update_enabled:false,values:[]}}), /Su değerlerini güncelleme: kapalı/);
console.log("Hunt presenter: context/unknown health/frozen learning/foreign kill/uncertainty checks passed");

// Refuge presenter tests
const refugeMemory = {
  refuge_outcomes_enabled: true,
  refuge_generalization_enabled: true,
  refuge_generalization_update_enabled: false,
  refuge_episode: { target: { x: 5, y: 6 }, elapsed_ticks: 3, movement_energy: 9, threat_damage: 2 },
  refuge_outcomes: [{ key: { endpoint: { x: 5, y: 6 } }, completed: 2, safe_arrivals: 2, threat_damage: 0, movement_energy: 8 }],
  refuge_generalized_outcomes: [{ completed: 1, safe_arrivals: 1, threat_damage: 0, movement_energy: 4 }],
};
const refugeText = experience.refuge(refugeMemory, "FleeToRefuge").join("\n");
assert.match(refugeText, /Sığınak seçimini öğrenme: açık/);
assert.match(refugeText, /Sığınak genellemesi: açık \(tablo güncelleme kapalı\)/);
assert.match(refugeText, /Bu karar sığınak deneyimi veya kaçış gerekçesinden geldi/);
assert.match(refugeText, /Devam eden sığınak denemesi: hedef \(5, 6\) · 3 adım · 9 hareket enerjisi · 2 tehdit hasarı/);
assert.match(refugeText, /Kayıtlı exact sığınak sonuçları: 1 kayıt/);
assert.match(refugeText, /Sığınak hedefi \(5, 6\) · 2 tamamlanma · 2 güvenli varış/);
assert.match(refugeText, /Kayıtlı genelleştirilmiş sığınak sonuçları: 1 kayıt/);

const disabledRefugeText = experience.refuge({ refuge_outcomes_enabled: false }, null).join("\n");
assert.match(disabledRefugeText, /Sığınak seçimini öğrenme: kapalı/);
assert.match(disabledRefugeText, /Sığınak sonuç tablosu verisi yok/);

const nullRefugeText = experience.refuge(null, null).join("\n");
assert.match(nullRefugeText, /Sığınak seçimini öğrenme: veri yok/);
assert.match(nullRefugeText, /Bu kayıtta sığınak deneyimi verisi yok/);
console.log("Refuge presenter: enabled/disabled/outcomes/generalization checks passed");

// Render refuge wiring tests
const renderedRefugeNodes = [];
const refugeContainer = {
  replaceChildren() { renderedRefugeNodes.length = 0; },
  append(p) { renderedRefugeNodes.push(p); },
  ownerDocument: { createElement() { return { textContent: "" }; } },
};
experience.render(refugeContainer, {
  memory: {
    refuge_outcomes_enabled: true,
    refuge_outcomes: [{ completed: 1, safe_arrivals: 1, threat_damage: 0, movement_energy: 4 }],
  },
  refuge: {
    headline: "Kayıtlı sığınak tecrübesi",
    Rule: "Yalnızca doğrulanmış varışlar",
    rows: ["1 güvenli varış"],
  },
});
assert.ok(renderedRefugeNodes.some(p => p.textContent.includes("Sığınak deneyimi")));
assert.ok(renderedRefugeNodes.some(p => p.textContent.includes("Kayıtlı sığınak tecrübesi")));
assert.ok(renderedRefugeNodes.some(p => p.textContent.includes("Yalnızca doğrulanmış varışlar")));
assert.ok(renderedRefugeNodes.some(p => p.textContent.includes("1 güvenli varış")));

// Verify replay.js and replay.html wiring
const fs = require("node:fs");
const path = require("node:path");
const rHtml = fs.readFileSync(path.join(__dirname, "../web/replay.html"), "utf8");
const rJs = fs.readFileSync(path.join(__dirname, "../web/replay.js"), "utf8");
assert.ok(rHtml.includes('id="auto-director"'));
assert.ok(rJs.includes("CuriousDocumentary.autoDirector"));
assert.ok(rJs.includes("Kanıt -> Eylem -> Resolver -> Bellek"));
console.log("Render refuge & replay wiring checks passed");

"use strict";
const assert = require("node:assert/strict");
const D = require("../web/documentary.js");
const context={energy:1,stamina:0,distance:2,health:2};
function row(id,tick,ctx=context,events=[],episodeTicks=1) {
  return {id,tick,shard:0,events,actor:{life:{age:tick-10},memory:{sites:[],dangers:[]},hunt_learner:{risk_enabled:false,risk_values:[],episode:{target:44,context:ctx,ticks:episodeTicks,movement_energy:3},last_episode:{end:"Fed",reward:7}}},trace:{id:7,tick,before:{age:tick-11},decision:{reason:"LearnedHunt",intent:{Move:{steps:[]}}},target:{Position:{x:2,y:4}}}};
}
const rows=[row(1,11,context,[],1),row(2,12,{...context,health:0},[],2),row(3,13,{...context,distance:1},[],1),row(4,14,context,[],1),row(5,15,context,[],2)];
const comparison=D.encounterComparison(rows,rows[0]);
assert.equal(comparison.count,2,"actual recorded context matches only exact known classes");
assert.equal(comparison.first.id,1);
assert.equal(comparison.last.id,4,"last similar encounter is the latest attempt start, not a later tick in that attempt");
assert.equal(comparison.count,2,"repeated ticks within one target/context episode count as one encounter");
assert.equal(D.sameContext(context,{...context,health:0}),false,"health class 0 means unknown and must not match");
const rendered=D.displayData(rows[0]);
assert.match(rendered.identity,/karar öncesi yaş 0/);
assert.match(rendered.memory,/Tick sonrası bellek/);
assert.equal(rendered.target,"(2, 4)");
assert.match(rendered.learned,/Son deneme alanı tek başına yeni güncelleme kanıtı değildir/,"repeated last_episode is not counted as a new learning update");
assert.match(rendered.risk,/Risk öğrenmesi kapalı/);
const riskRow=row(5,15);riskRow.actor.hunt_learner.risk_enabled=true;riskRow.actor.hunt_learner.risk_values=[{context:{energy:1},exposures:3,injuries:1,damage:2}];riskRow.actor.hunt_learner.episode={...riskRow.actor.hunt_learner.episode,prey_defense_damage:4};
const riskText=D.displayData(riskRow).risk;
assert.match(riskText,/1 kayıtlı risk örneği/);
assert.match(riskText,/3 riskli karşılaşma, 1 yaralanma, 2 kayıtlı hasar/);
assert.match(riskText,/avın verdiği hasar 4/);
assert.doesNotMatch(riskText,/(risk_enabled|risk_values|prey_defense_damage|HuntContext)/,"product wording must use plain Turkish, not storage field names");
const noCost=row(6,16);noCost.events=[{EntityMoved:{id:7,from:{x:0,y:0},to:{x:1,y:0}}}];
assert.match(D.displayData(noCost).cost,/eksik\/eski olay var; toplam bilinmiyor/);
const waterBefore={actor:{learner:{values:[{pos:{x:1,y:2},value:12,visits:2}]}}};
const waterAfter={actor:{learner:{values:[{pos:{x:1,y:2},value:7,visits:3}]}}};
assert.match(D.waterDelta(waterBefore,waterAfter),/Q Δ-5, ziyaret Δ1/);
assert.match(D.waterDelta(null,waterAfter),/farkı bilinmiyor/);
const missing=D.displayData({id:2,tick:2,trace:{id:2,decision:{reason:"Explore"}},events:[]});
assert.match(missing.memory,/Bellek kaydı yok/);
assert.equal(D.huntContext({decision:{reason:"LearnedHunt"}}),null,"old sparse records do not invent a comparison context");

// Causal chain tests
const sparseItem = { id: 2, tick: 2, trace: { id: 2, decision: { reason: "Explore" } }, events: [] };
const sparseChain = D.causalChain(sparseItem);
assert.equal(sparseChain.evidence, "Unknown (kayıtta yok)", "missing perception/evidence is explicitly Unknown");
assert.match(sparseChain.action, /Explore/, "recorded action is shown");
assert.equal(sparseChain.resolverOutcome, "Unknown (kayıtta yok)", "empty events means resolver outcome is Unknown");
assert.equal(sparseChain.memoryUpdate, "Unknown (kayıtta yok)", "missing actor/learner update is Unknown");

const emptyItem = null;
const nullChain = D.causalChain(emptyItem);
assert.equal(nullChain.evidence, "Unknown (kayıtta yok)");
assert.equal(nullChain.action, "Unknown (kayıtta yok)");
assert.equal(nullChain.resolverOutcome, "Unknown (kayıtta yok)");
assert.equal(nullChain.memoryUpdate, "Unknown (kayıtta yok)");

const richRow = row(7, 17, context, [{ EntityMoved: { id: 7, energy_cost: 2 } }], 1);
richRow.trace.perception = { entities: [1], foods: [], waters: [], shelters: [] };
richRow.actor.learner = { enabled: true };
richRow.actor.memory = { refuge_outcomes_enabled: true, refuge_outcomes: [{ completed: 1 }] };
const richChain = D.causalChain(richRow);
assert.match(richChain.evidence, /1 canlı/);
assert.match(richChain.action, /LearnedHunt/);
assert.match(richChain.resolverOutcome, /hareket \(2 enerji\)/);
assert.match(richChain.memoryUpdate, /sığınak tablosunda 1 kayıt/);
assert.deepEqual(D.displayData(richRow).causalChain, richChain, "displayData exposes causalChain");

// Auto-Director documentary camera tests
assert.equal(D.autoDirector([]), null, "empty array returns null");
assert.equal(D.autoDirector(null), null, "null returns null");

const p0Item = { id: 10, tick: 10, trace: { id: 10, decision: { reason: "Explore" } }, events: [] };
const p0Res = D.autoDirector([p0Item]);
assert.equal(p0Res.priority, 0);
assert.equal(p0Res.category, "routine_observation");
assert.match(p0Res.headline, /#10 olağan gözlem kaydı/);

const p1Item = { id: 11, tick: 10, trace: { id: 11, decision: { reason: "LearnedWater" } }, events: [] };
const p2Item = { id: 12, tick: 10, trace: { id: 12, decision: { reason: "Explore" } }, events: [{ EntityBorn: { id: 99 } }] };
const p3Item = { id: 13, tick: 10, trace: { id: 13, decision: { reason: "HuntPrey" } }, events: [{ EntityAttacked: { attacker: 13 } }] };
const p4Item = { id: 14, tick: 10, trace: { id: 14, decision: { reason: "EscapeThreat" } }, actor: { life: { health: 10, max_health: 100 } }, events: [] };

const mixed1 = D.autoDirector([p0Item, p1Item]);
assert.equal(mixed1.priority, 1);
assert.equal(mixed1.category, "learned_resource");
assert.equal(mixed1.focalItem.id, 11);

const mixed2 = D.autoDirector([p0Item, p1Item, p2Item]);
assert.equal(mixed2.priority, 2);
assert.equal(mixed2.category, "lineage_birth");
assert.equal(mixed2.focalItem.id, 12);

const mixed3 = D.autoDirector([p0Item, p1Item, p2Item, p3Item]);
assert.equal(mixed3.priority, 3);
assert.equal(mixed3.category, "active_predation");
assert.equal(mixed3.focalItem.id, 13);

const mixed4 = D.autoDirector([p0Item, p1Item, p2Item, p3Item, p4Item]);
assert.equal(mixed4.priority, 4);
assert.equal(mixed4.category, "critical_escape");
assert.equal(mixed4.focalItem.id, 14);

// Replay HTML & JS wiring assertions
const fs = require("node:fs");
const path = require("node:path");
const replayHtml = fs.readFileSync(path.join(__dirname, "../web/replay.html"), "utf8");
const replayJs = fs.readFileSync(path.join(__dirname, "../web/replay.js"), "utf8");

assert.ok(replayHtml.includes('id="auto-director"'), "replay.html must declare #auto-director container");
assert.ok(replayJs.includes("CuriousDocumentary.autoDirector"), "replay.js must call CuriousDocumentary.autoDirector");
assert.ok(replayJs.includes("Kanıt -> Eylem -> Resolver -> Bellek"), "replay.js must render causalChain format");

console.log("documentary projection checks passed");

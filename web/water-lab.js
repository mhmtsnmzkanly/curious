(function (root) {
  "use strict";
  const ARMS = ["rule_based", "naive_online", "trained_online", "trained_frozen"];
  const MAX_FILE = 20 * 1024 * 1024;
  function validate(bundle) {
    if (!bundle || bundle.schema !== "water-adaptation-replay-v1") throw Error("Desteklenmeyen tekrar şeması.");
    if (!Array.isArray(bundle.arms) || bundle.arms.length !== 4) throw Error("Paket tam olarak dört kol içermeli.");
    for (const name of ARMS) {
      const arm = bundle.arms.find(x => x && x.name === name);
      if (!arm || !Array.isArray(arm.frames) || arm.frames.length > 401) throw Error(`Eksik veya geçersiz kol: ${name}`);
      if (!arm.frames.every(f => f && Number.isFinite(Number(f.tick)))) throw Error(`${name} kolunda geçersiz kareler var.`);
    }
    if (!bundle.sites || !bundle.sites.a || !bundle.sites.b) throw Error("Paket a ve b kaynak konumlarını içermeli.");
    if (!Number.isFinite(Number(bundle.fork_tick))) throw Error("Paket fork_tick değerini içermeli.");
    return bundle;
  }
  function project(bundle, armName, tick) {
    const arm = bundle.arms.find(a => a.name === armName);
    if (!arm) return null;
    const frame = arm.frames.find(f => Number(f.tick) === Number(tick));
    if (!frame) return null;
    const state = frame.state || frame.state_after || frame.state_before || null;
    const entities = state && Array.isArray(state.entities) ? state.entities : [];
    const decisions = Array.isArray(frame.decisions) ? frame.decisions : [];
    const events = Array.isArray(frame.events) ? frame.events : [];
    const forkEvent = events.filter(e => /fork|dry|source/i.test(String(e.type || e.kind || e.event || "")));
    return { arm, frame, state, entities, decisions, events, forkEvent, metrics: arm.metrics || {} };
  }
  function contextRows(entity) {
    const learner = entity && (entity.learning || entity.water_learner || entity.learner);
    if (!learner) return { contextual: [], legacy: [] };
    return { contextual: Array.isArray(learner.context_entries) ? learner.context_entries : [], legacy: Array.isArray(learner.values) ? learner.values : [] };
  }
  if (typeof module !== "undefined" && module.exports && typeof process !== "undefined" && process.argv.includes("--self-test")) {
    const assert = require("node:assert/strict");
    const bundle = { schema:"water-adaptation-replay-v1", seed:4, source:"test", fork_tick:200, sites:{a:{x:1,y:2},b:{x:8,y:9}}, arms: ARMS.map(name=>({name,metrics:{water:12},frames:[{tick:201,state:{entities:[{actor:true,learning:{context_entries:[{pos:{x:1,y:2}}],values:[{value:3}]}}]},decisions:[],events:[]}]})) };
    assert.equal(validate(bundle), bundle);
    assert.equal(project(bundle,"trained_online",201).entities.length,1);
    assert.equal(contextRows(project(bundle,"trained_online",201).entities[0]).contextual.length,1);
    assert.equal(contextRows(project(bundle,"trained_online",201).entities[0]).legacy.length,1);
    assert.equal(project(bundle,"trained_online",200),null);
    assert.throws(()=>validate({...bundle,schema:"other"}),/Desteklenmeyen/);
    process.stdout.write("water-lab projection self-test passed\n");
  }
  if (typeof module !== "undefined" && module.exports) module.exports = { ARMS, MAX_FILE, validate, project, contextRows };
  if (typeof document === "undefined") return;

  const $ = id => document.getElementById(id);
  let data = null;
  const colors = { water: "#70bdd5", actor: "#f1d28b", other: "#b3c2a5" };
  function text(node, value) { node.textContent = value == null ? "—" : String(value); }
  function safe(value) { return value == null ? "—" : typeof value === "object" ? JSON.stringify(value) : String(value); }
  function line(parent, label, value) { const p = document.createElement("p"); p.textContent = `${label}: ${safe(value)}`; parent.append(p); }
  function render(position) {
    if (!data) return;
    const tick = position;
    text($("tick-label"), tick);
    for (const name of ARMS) {
      const card = $("arm-" + name), view = project(data, name, tick);
      const canvas = card.querySelector("canvas"), ctx = canvas.getContext("2d");
      draw(ctx, canvas, view);
      const facts = card.querySelector(".facts"); facts.replaceChildren();
      if (!view) { line(facts, "Kare", "kayıt yok"); continue; }
      const actor = view.entities.find(e => e.actor || e.is_actor) || view.entities[0];
      const recordedDecision = view.decisions.find(d => actor && (d.id === actor.id || d.entity_id === actor.id)) || view.decisions[0];
      const decision = recordedDecision && (recordedDecision.decision || recordedDecision);
      line(facts, "Karar", decision && (decision.action || decision.intent || decision.kind));
      line(facts, "Gerekçe", decision && (decision.reason || decision.explanation));
      if (actor) {
        line(facts, "Konum", actor.pos && `${actor.pos.x}, ${actor.pos.y}`);
        for (const [k,label] of [["energy","Enerji"],["water","Su"],["health","Sağlık"]]) line(facts, label, actor.life ? actor.life[k] : actor[k]);
        const rows = contextRows(actor);
        const heading = document.createElement("h3"); heading.textContent = "Öğrenici kayıtları"; facts.append(heading);
        if (rows.contextual.length) rows.contextual.forEach(r => line(facts, "Bağlam · kayıtlı veri", r));
        else line(facts, "Bağlam", "kayıt yok");
        rows.legacy.forEach(r => line(facts, "Eski değer tablosu · kayıtlı veri", r));
      }
      const forkRecords = view.arm.frames.flatMap(f => (Array.isArray(f.events) ? f.events : []).filter(event => /fork|dry|source/i.test(String(event.type || event.kind || event.event || ""))).map(event => ({tick:f.tick,event})));
      line(facts, "Kaydedilmiş çatal / kaynak olayları", forkRecords.length ? forkRecords : "kayıtlı olay yok");
      line(facts, "TTL", "48 adımdan eski kayıtlar tahminde kullanılmaz; dondurulmuş tablolar saklanır, saat işler.");
    }
  }
  function draw(ctx, canvas, view) {
    const w = canvas.width = Math.max(260, canvas.clientWidth * devicePixelRatio), h = canvas.height = Math.max(220, canvas.clientWidth * devicePixelRatio * .9);
    ctx.scale(devicePixelRatio, devicePixelRatio); const cw = w / devicePixelRatio, ch = h / devicePixelRatio;
    ctx.fillStyle = "#14251a"; ctx.fillRect(0,0,cw,ch); ctx.strokeStyle = "#293d30";
    for(let x=0;x<=10;x++){ctx.beginPath();ctx.moveTo(x*cw/10,0);ctx.lineTo(x*cw/10,ch);ctx.stroke();}
    for(let y=0;y<=8;y++){ctx.beginPath();ctx.moveTo(0,y*ch/8);ctx.lineTo(cw,y*ch/8);ctx.stroke();}
    if (!data) return;
    const sites = data.sites;
    const state=view && view.state;
    const resources=state && Array.isArray(state.resources)?state.resources:[];
    const resourceRows=resources.map(r=>Array.isArray(r)?{pos:r[0],cell:r[1]}:r).filter(r=>r&&r.pos);
    const points = [sites.a,sites.b,...resourceRows.map(r=>r.pos)];
    if(view) for(const e of view.entities) if(e.pos) points.push(e.pos);
    const bounds=state&&Array.isArray(state.bounds)?state.bounds:null;
    const xs=points.map(p=>Number(p.x)||0), ys=points.map(p=>Number(p.y)||0), minX=bounds?Number(bounds[0]):Math.min(...xs), maxX=bounds?Number(bounds[1]):Math.max(...xs), minY=bounds?Number(bounds[2]):Math.min(...ys), maxY=bounds?Number(bounds[3]):Math.max(...ys);
    const xy=p=>[18+(Number(p.x-minX)/(maxX-minX||1))*(cw-36),18+(Number(p.y-minY)/(maxY-minY||1))*(ch-36)];
    for(const [key,p] of Object.entries(sites)){const [x,y]=xy(p);ctx.fillStyle=colors.water;ctx.beginPath();ctx.arc(x,y,7,0,Math.PI*2);ctx.fill();ctx.fillStyle="#dce9e5";ctx.font="11px sans-serif";ctx.fillText(key.toUpperCase(),x+9,y+4);}
    for(const resource of resourceRows){const desc=JSON.stringify(resource.cell||resource).toLowerCase();if(!/water|food|wall|burrow/.test(desc))continue;const [x,y]=xy(resource.pos);ctx.fillStyle=desc.includes("water")?colors.water:desc.includes("food")?"#bcd991":desc.includes("wall")?"#777d79":"#a98b69";ctx.fillRect(x-2.5,y-2.5,5,5);}
    if(view) view.entities.forEach((e,i)=>{if(!e.pos)return;const [x,y]=xy(e.pos);ctx.fillStyle=e.actor||e.is_actor?colors.actor:colors.other;ctx.beginPath();ctx.arc(x,y,e.actor||e.is_actor?6:3.5,0,Math.PI*2);ctx.fill();});
  }
  function makeCard(name) { const c=document.createElement("section");c.className="arm";c.id="arm-"+name;const h=document.createElement("h2");h.textContent=name.replaceAll("_"," ");const cv=document.createElement("canvas");const label=document.createElement("p");label.className="muted";label.textContent="Su kaynakları A/B · kaydedilmiş varlıklar";const title=document.createElement("h3");title.textContent="Kare ayrıntıları";const facts=document.createElement("div");facts.className="facts";c.append(h,cv,label,title,facts);return c; }
  $("bundle-file").addEventListener("change", async e => {
    const file=e.target.files && e.target.files[0]; if(!file)return;
    try { if(file.size>MAX_FILE)throw Error("Dosya 20 MiB sınırını aşıyor."); const source=await file.text(); const parsed=validate(JSON.parse(source)); data=parsed;
      text($("seed"),parsed.seed);text($("source"),parsed.source);text($("fork"),parsed.fork_tick);text($("branch-note"),`Kaynak değişiklikleri yalnızca karelerde kaydedilmişse gösterilir; görüntüleyici olay çıkarımı yapmaz. Müdahale: ${parsed.intervention ?? "pakette belirtilmemiş"}`);
      const root=$("arms");root.replaceChildren(...ARMS.map(makeCard));$("viewer").hidden=false;$("error").hidden=true;$("status").textContent=`${file.name} yüklendi (${(file.size/1024/1024).toFixed(2)} MiB).`;
      const metricRoot=$("metrics");metricRoot.replaceChildren(...ARMS.map(name=>{const arm=parsed.arms.find(a=>a.name===name),box=document.createElement("section");box.className="metric-card";const h=document.createElement("h3");h.textContent=name.replaceAll("_"," ");box.append(h);const entries=Object.entries(arm.metrics||{});if(!entries.length)line(box,"Metrics","not provided");else entries.forEach(([k,v])=>line(box,k,v));return box;}));
      const tickLists=parsed.arms.map(a=>a.frames.map(f=>Number(f.tick)).filter(t=>Number.isFinite(t)).sort((a,b)=>a-b));
      const commonMin=Math.max(...tickLists.map(xs=>xs[0]??Infinity)),commonMax=Math.min(...tickLists.map(xs=>xs[xs.length-1]??-Infinity));
      if(!Number.isFinite(commonMin)||commonMin>commonMax)throw Error("Dört kolun ortak kayıtlı adım aralığı yok.");
      const slider=$("tick");slider.min=String(commonMin);slider.max=String(commonMax);slider.value=String(commonMin);slider.oninput=()=>render(Number(slider.value));render(Number(slider.value));
    } catch(err){$("error").hidden=false;$("error").textContent=err.message;}
  });
})(typeof globalThis !== "undefined" ? globalThis : this);

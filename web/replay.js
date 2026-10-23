"use strict";
const $ = (id) => document.getElementById(id);
const canvas = $("world"), ctx = canvas.getContext("2d");
const colors = {Herbivore:"#bddc94",Carnivore:"#eca080",Omnivore:"#b7a0db"};
const names = {Herbivore:"Otçul",Carnivore:"Etçil",Omnivore:"Hepçil"};
const labels = {SeekWater:"Su arıyor",SearchWater:"Su arıyor",RecallWater:"Hatırladığı suya gidiyor",SeekFood:"Besleniyor",SearchFood:"Yiyecek arıyor",LearnedHunt:"Deneyimine göre av seçti",HuntPrey:"Avını izliyor",EscapeThreat:"Tehlikeden kaçıyor",SeekShelter:"Sığınağa kaçıyor",Rest:"Dinleniyor",Mate:"Eşleşiyor",Explore:"Çevresini araştırıyor",RecoverStamina:"Dinleniyor",LearnedWater:"Deneyimine göre su seçti"};
let shards=[], records=[], current=null, selected=null, playing=false, timer=null, cursor=0, meta=null, navigation=0, playbackRun=0;
let lifeRows=[], lifeKey="", lifeShardIndex=-1, lifeBefore=0, lifeGeneration=0, lifeOlderAvailable=false, lifeAnchor=null, lifeLoadBusy=false, lifeBudgetLimited=false;
function api(path) { return fetch(path,{cache:"no-store"}).then(async r=>{const d=await r.json();if(!r.ok)throw Error(d.error||"Arşiv okunamadı");return d;}); }
function esc(value) { return String(value ?? "—"); }
function setError(error) { $("error").textContent=error?.message||""; $("error").hidden=!error; }
async function openShard(id, continuePlayback=false) {
  if(!continuePlayback)stop(); const nav=++navigation; records=[]; if(!continuePlayback)current=null; cursor=0; $("timeline").disabled=true; $("play").disabled=!continuePlayback; $("older").disabled=true;
  $("connection").textContent=`${id}. arşiv parçası açılıyor…`;
  meta=await api(`/api/shard-meta?shard=${id}`);
  if(nav!==navigation)return;
  if(!meta.records) throw Error("Bu arşiv parçasında kayıt yok.");
  records=await api(`/api/timeline?shard=${id}&after=0&limit=500`);
  if(nav!==navigation)return;
  cursor=records.length?records[records.length-1].id:0;
  $("older").disabled=records.length>=meta.records;
  $("timeline").min="0"; $("timeline").max=String(Math.max(0,records.length-1)); $("timeline").value=String(records.length-1); $("timeline").disabled=!records.length;
  $("play").disabled=!records.length; $("beginning").disabled=!records.length;
  $("connection").textContent=`${records.length} kayıt yüklendi · ${meta.records} toplam`;
  if(records.length && !continuePlayback) await show(records.length-1);
  setError(null);
}
async function show(index) {
  if(index<0||index>=records.length)return;
  const nav=++navigation, r=records[index], shard=Number($("shard").value), prior=selected;
  const loaded=await api(`/api/record?shard=${shard}&id=${r.id}`);
  if(nav!==navigation||shard!==Number($("shard").value))return;
  current=loaded; selected=(loaded.state.entities||[]).some(e=>e.id===prior)?prior:null;
  $("timeline").value=String(index); $("tick").textContent=Number(current.tick).toLocaleString("tr");
  $("record-title").textContent=`Adım ${Number(current.tick).toLocaleString("tr")}`;
  $("kind").textContent=current.kind==="tick"?"TICK":"CHECKPOINT";
  const alive=(current.state.entities||[]).filter(e=>e.life?.health>0);
  $("alive").textContent=String(current.metrics?.alive??alive.length);
  $("generation").textContent=String(current.metrics?.max_living_generation??Math.max(0,...alive.map(e=>e.generation||0)));
  $("timeline-label").textContent=`${index+1}/${records.length} · ${current.at_ms?new Date(Number(current.at_ms)).toLocaleString("tr"):"zaman bilgisi yok"}`;
  $("map-empty").hidden=alive.length>0;
  renderRoster(); renderSummary(); draw();
}
function renderRoster() {
  const list=$("roster");list.replaceChildren();
  for(const e of current?.state?.entities||[]) {
    const b=document.createElement("button");b.textContent=`${e.species==="Carnivore"?"▲":e.species==="Omnivore"?"■":"●"} #${e.id}`;
    b.style.color=colors[e.species]||"#fff";b.className=e.id===selected?"selected":"";b.onclick=()=>{selected=e.id;renderRoster();renderSummary();draw();};list.append(b);
  }
}
function renderSummary() {
  const state=current?.state, metrics=current?.metrics||{}, entities=state?.entities||[];
  const e=entities.find(x=>x.id===selected);
  $("individual-title").textContent=e?`${names[e.species]||e.species} #${e.id}`:"Dünya özeti";
  $("individual-sub").textContent=e?`${e.generation??"?"}. nesil · ${e.life?.age??"?"} adım yaşam`:`Kayıt türü: ${current?.kind||"—"} · ${entities.length} kayıtlı birey`;
  $("vitals").replaceChildren();
  for(const [key,label,color] of [["health","Sağlık","#d9a594"],["energy","Enerji","#d8d295"],["water","Su","#7eb8ca"],["stamina","Dayanıklılık","#a0bf93"]]){
    if(!e)break;const line=document.createElement("div"),name=document.createElement("span"),value=document.createElement("span"),track=document.createElement("div"),fill=document.createElement("i");
    line.className="vital-line";name.textContent=label;value.textContent=`${e.life?.[key]??"?"} / ${e.life?.[`max_${key}`]??"?"}`;track.className="track";fill.style.width=`${Math.min(100,100*(e.life?.[key]||0)/(e.life?.[`max_${key}`]||1))}%`;fill.style.background=color;track.append(fill);line.append(name,value);$("vitals").append(line,track);
  }
  const details=$("details");details.replaceChildren();
  if(e){for(const [k,v] of [["Konum",`${e.pos?.x}, ${e.pos?.y}`],["Tür",names[e.species]||e.species],["Ebeveynler",e.parents?.length?e.parents.map(x=>`#${x}`).join(" & "):"Kurucu"],["Bellek",`${e.memory?.sites?.length||0} kaynak · ${e.memory?.dangers?.length||0} tehlike`]]){const dt=document.createElement("dt"),dd=document.createElement("dd");dt.textContent=k;dd.textContent=esc(v);details.append(dt,dd);}}
  CuriousExperience.render($("experience"),e,(current?.decisions||[]).find(d=>d.id===selected));
  loadLife(e).catch(setError);
  const decisionBox=$("decisions");decisionBox.replaceChildren();
  const traceList=(current?.decisions||[]).filter(d=>selected===null||d.id===selected);
  const counts={};for(const d of traceList){const reason=d.decision?.reason||"Bilinmiyor";counts[labels[reason]||reason]=(counts[labels[reason]||reason]||0)+1;}
  for(const [name,count] of Object.entries(counts).sort((a,b)=>b[1]-a[1]).slice(0,8)){const p=document.createElement("p");p.textContent=`${name}: ${count}`;decisionBox.append(p);}
  if(!Object.keys(counts).length){const p=document.createElement("p");p.textContent="Bu kayıtta karar izi yok.";decisionBox.append(p);}
  const species=$("species");species.replaceChildren();
  const countsBy=metrics.by_species||{};for(const [kind,count] of Object.entries(countsBy)){const p=document.createElement("p");p.textContent=`${names[kind]||kind}: ${count}`;species.append(p);}
  if(!Object.keys(countsBy).length){const p=document.createElement("p");p.textContent=`Yaşayan: ${metrics.alive??entities.filter(x=>x.life?.health>0).length} · Doğum: ${metrics.births??0} · Ölüm: ${metrics.deaths??0}`;species.append(p);}
}
function addText(parent, tag, value, className="") { const node=document.createElement(tag);node.textContent=String(value??"—");if(className)node.className=className;parent.append(node);return node; }
function renderLife() {
  const box=$("documentary");box.replaceChildren();
  if(selected===null){addText(box,"p","Bir birey seçerek kayıtlı yaşam izini aç.");$("older-life").hidden=true;return;}
  if(lifeBudgetLimited)addText(box,"p","Bu sayfa okuma bütçesi nedeniyle kısmi. Daha eski kayıtları aynı arşiv parçasında sürdürmek için yeniden deneyin.","documentary-note");
  if(!lifeRows.length){addText(box,"p","Seçili birey için bu arşiv aralığında karar kaydı bulunamadı.");$("older-life").hidden=!lifeOlderAvailable;return;}
  const view=lifeRows.map(r=>({...r,summary:CuriousDocumentary.displayData(r)}));
  const comparison=CuriousDocumentary.encounterComparison(view,lifeAnchor);
  const rule=document.createElement("p");rule.className="documentary-note";
  rule.textContent=`Benzer karşılaşma kuralı: saklanan HuntContext içindeki enerji, dayanıklılık, mesafe ve av sağlığı sınıflarının dördü de aynı olmalı. Yalnız kaydedilmiş attempt bağlamı kullanılır; eksik sınıf eşleşmez ve algıdan tahmin edilmez. Benzerlik nedensel üstünlük göstermez. Karşılaştırma şu an taranmış ${view.length} kaydın kapsamındadır.`;box.append(rule);
  if(comparison.count){
    const compare=document.createElement("div");compare.className="life-compare";
    addText(compare,"strong",`Benzer av karşılaşması · ${CuriousDocumentary.contextLabel(comparison.context)}`);
    for(const [label,item] of [["İlk taranan",comparison.first],["Son taranan",comparison.last]])if(item){const s=item.summary;addText(compare,"p",`${label}: #${s.id} · tick ${s.tick} · karar öncesi yaş ${s.beforeAge??"bilinmiyor"} · ${s.reason}`);}
    addText(compare,"p",`${comparison.count} benzer karşılaşma grubu; artan attempt tick'leri tek bölüm sayılır. Bu sayı başarı veya öğrenme etkisi değildir.`);box.append(compare);
  } else addText(box,"p",lifeAnchor?"Bu yüklü kayıtlarda eşleşme ölçütünü karşılayan av karşılaşması yok.":"Seçili tick için bireysel karar izi yok; karşılaştırma başlatılamadı.");
  for(let index=view.length-1;index>=0;index--){
    const row=view[index],s=row.summary,previous=index>0?view[index-1]:null,card=document.createElement("article");card.className="life-entry";
    addText(card,"strong",`${s.identity} · ${s.reason}`);
    addText(card,"p",`Gördüğü: ${s.perception}`);
    addText(card,"p",`Hatırladığı: ${s.memory}`);
    addText(card,"p",`Hedef: ${s.target===null?"kayıtlı hedef yok":(typeof s.target==="number"?`#${s.target}`:s.target)}. ${s.cost}`);
    addText(card,"p",`Öğrendiği: ${s.learned} ${s.outcome}`);
    addText(card,"p",`Su Q tablosu: ${CuriousDocumentary.waterDelta(previous,row)}`);
    addText(card,"p",s.risk);
    if(row.trace?.perception?.entities?.length){const seen=row.trace.perception.entities.slice(0,12).map(x=>`#${x.id} ${x.species}, ${x.steps?.length??"?"} adım${x.health==null?", sağlık bilinmiyor":`, sağlık ${x.health}`}`).join("; ");addText(card,"p",`Görülen canlılar: ${seen}${row.trace.perception.entities.length>12?"; …":""}`);}
    box.append(card);
  }
  $("older-life").hidden=!lifeOlderAvailable;$("older-life").disabled=lifeLoadBusy;
}
async function loadLife(entity) {
  if(!entity||selected===null||!current){lifeRows=[];lifeKey="";renderLife();return;}
  const shard=Number($("shard").value),key=`${selected}:${shard}:${current.id}`;
  if(key===lifeKey)return;
  const generation=++lifeGeneration;lifeLoadBusy=false;lifeKey=key;lifeRows=[];lifeAnchor=null;lifeShardIndex=shards.findIndex(s=>s.shard===shard);lifeBefore=Number(current.id)+1;lifeOlderAvailable=true;renderLife();
  let page;
  try{page=await api(`/api/individual-history?shard=${shard}&id=${selected}&before=${lifeBefore}&limit=50`);}catch(e){if(generation===lifeGeneration)lifeKey="";throw e;}
  if(generation!==lifeGeneration||selected!==entity.id||Number($("shard").value)!==shard)return;
  lifeRows=page.items;lifeAnchor=page.items.find(r=>r.id===current.id&&r.shard===shard)||null;
  lifeBudgetLimited=page.budget_limited;
  lifeBefore=page.next_before??(page.items.length?page.items[0].id:lifeBefore);
  lifeOlderAvailable=page.has_more||lifeShardIndex>0;renderLife();
}
$("older-life").onclick=async()=>{
  if(!lifeOlderAvailable||selected===null||lifeLoadBusy)return;
  const generation=lifeGeneration, id=selected;
  lifeLoadBusy=true;$("older-life").disabled=true;
  try{
    let response={items:[],has_more:false,next_before:null,budget_limited:false},candidateIndex=lifeShardIndex,candidateBefore=lifeBefore,attempts=0;
    while(!response.items.length&&candidateIndex>=0&&attempts<3){
      const shard=shards[candidateIndex]?.shard;if(shard===undefined)break;
      if(candidateBefore<=1){candidateIndex--;candidateBefore=Number.MAX_SAFE_INTEGER;attempts++;continue;}
      const before=candidateBefore;
      const limit=Math.min(50,500-lifeRows.length);
      if(limit<=0)break;
      response=await api(`/api/individual-history?shard=${shard}&id=${id}&before=${before}&limit=${limit}`);
      if(response.budget_limited){candidateBefore=response.next_before??before;break;}
      if(!response.items.length){candidateIndex--;candidateBefore=Number.MAX_SAFE_INTEGER;attempts++;}
    }
    if(generation!==lifeGeneration||id!==selected)return;
    lifeShardIndex=candidateIndex;
    lifeBudgetLimited=response.budget_limited;
    if(response.items.length){lifeRows=[...response.items,...lifeRows];lifeBefore=response.next_before??response.items[0].id;}
    else if(response.budget_limited){lifeBefore=response.next_before??candidateBefore;}
    else {lifeBefore=Number.MAX_SAFE_INTEGER;if(candidateIndex<0)lifeOlderAvailable=false;}
    lifeOlderAvailable=lifeRows.length<500&&(response.has_more||response.budget_limited||(lifeShardIndex>=0&&(lifeBefore>1||lifeShardIndex>0)));
    renderLife();
  }catch(e){setError(e);}finally{if(generation===lifeGeneration){lifeLoadBusy=false;$("older-life").disabled=!lifeOlderAvailable;}}
};
function draw() {
  if(!current?.state)return;const rect=canvas.getBoundingClientRect(),dpr=devicePixelRatio||1;canvas.width=Math.round(rect.width*dpr);canvas.height=Math.round(rect.height*dpr);ctx.setTransform(dpr,0,0,dpr,0,0);const w=rect.width,h=rect.height;ctx.fillStyle="#14251a";ctx.fillRect(0,0,w,h);
  const b=current.state.bounds||[0,1,0,1],[minX,maxX,minY,maxY]=b;const cell=Math.min((w-30)/(maxX-minX+1),(h-30)/(maxY-minY+1)),ox=(w-cell*(maxX-minX+1))/2,oy=(h-cell*(maxY-minY+1))/2;
  ctx.strokeStyle="#294030";ctx.lineWidth=.5;for(let x=minX;x<=maxX+1;x++){ctx.beginPath();ctx.moveTo(ox+(x-minX)*cell,oy);ctx.lineTo(ox+(x-minX)*cell,oy+(maxY-minY+1)*cell);ctx.stroke();}for(let y=minY;y<=maxY+1;y++){ctx.beginPath();ctx.moveTo(ox,oy+(y-minY)*cell);ctx.lineTo(ox+(maxX-minX+1)*cell,oy+(y-minY)*cell);ctx.stroke();}
  const point=p=>[ox+(p.x-minX+.5)*cell,oy+(p.y-minY+.5)*cell],inside=p=>p.x>=minX&&p.x<=maxX&&p.y>=minY&&p.y<=maxY;
  for(const [p,c] of current.state.resources||[]){if(!inside(p))continue;const [x,y]=point(p),kind=typeof c==="string"?c:Object.keys(c)[0];ctx.fillStyle=kind==="Water"?"#3a6d78":kind==="Wall"?"#63695c":kind==="Burrow"?"#91775a":"#396044";ctx.fillRect(x-cell*.43,y-cell*.43,cell*.86,cell*.86);if(cell>25&&c[kind]?.amount!=null){ctx.fillStyle=kind==="Water"?"#b1d9e2":"#a2c982";ctx.font="10px system-ui";ctx.textAlign="center";ctx.fillText(String(c[kind].amount),x,y+4);}}
  for(const e of current.state.entities||[]){if(!inside(e.pos))continue;const [x,y]=point(e.pos),r=Math.max(3,Math.min(13,cell*.24));ctx.save();ctx.translate(x,y);ctx.globalAlpha=e.life?.health>0?1:.3;if(e.id===selected){ctx.strokeStyle="#edf0cc";ctx.lineWidth=2;ctx.beginPath();ctx.arc(0,0,r+6,0,Math.PI*2);ctx.stroke();}ctx.fillStyle=colors[e.species]||"#fff";ctx.beginPath();if(e.species==="Carnivore"){ctx.moveTo(0,-r);ctx.lineTo(r,r);ctx.lineTo(-r,r);ctx.closePath();}else if(e.species==="Omnivore")ctx.rect(-r,-r,r*2,r*2);else ctx.arc(0,0,r,0,Math.PI*2);ctx.fill();if(cell>27){ctx.fillStyle="#132217";ctx.font="bold 10px system-ui";ctx.textAlign="center";ctx.fillText(e.id,0,4);}ctx.restore();}
}
function stop(){playing=false;playbackRun++;clearTimeout(timer);$("play").textContent="▶ Oynat";}
async function advance(run=playbackRun){if(!playing||run!==playbackRun)return;let next=Number($("timeline").value)+1;if(next>=records.length){if(records.length<meta.records){try{const page=await api(`/api/timeline?shard=${$("shard").value}&after=${cursor}&limit=200`);if(!playing||run!==playbackRun)return;if(!page.length){stop();return;}records.push(...page);cursor=records[records.length-1].id;$("timeline").max=String(records.length-1);$("older").disabled=records.length>=meta.records;next=records.length-page.length;}catch(e){if(run===playbackRun){stop();setError(e);}return;}}else{const at=shards.findIndex(s=>s.shard===Number($("shard").value));if(at<0||at+1>=shards.length){stop();return;}try{const nextShard=shards[at+1].shard;$("shard").value=String(nextShard);await openShard(nextShard,true);if(!playing||run!==playbackRun||!records.length)return;$("play").textContent="Ⅱ Duraklat";await show(0);if(!playing||run!==playbackRun)return;timer=setTimeout(()=>advance(run),1000/Number($("rate").value));return;}catch(e){if(run===playbackRun){stop();setError(e);}return;}}}
  try{await show(next);if(!playing||run!==playbackRun)return;timer=setTimeout(()=>advance(run),1000/Number($("rate").value));}catch(e){if(run===playbackRun){stop();setError(e);}}}
$("play").onclick=()=>{if(playing){stop();return;}playing=true;const run=++playbackRun;$("play").textContent="Ⅱ Duraklat";advance(run);};
$("rate").onchange=()=>{};
$("timeline").oninput=()=>{stop();show(Number($("timeline").value)).catch(setError);};
$("shard").onchange=()=>openShard(Number($("shard").value)).catch(setError);
$("older").onclick=async()=>{const page=await api(`/api/timeline?shard=${$("shard").value}&after=${cursor}&limit=200`);if(!page.length){$("older").disabled=true;return;}records.push(...page);cursor=records[records.length-1].id;$("timeline").max=String(records.length-1);$("older").disabled=page.length<200;setError(null);};
$("beginning").onclick=async()=>{stop();if(!shards.length)return;$("shard").value=String(shards[0].shard);await openShard(shards[0].shard);await show(0);};
new ResizeObserver(draw).observe(canvas);
(async()=>{try{shards=await api("/api/shards");if(!shards.length)throw Error("Bu dizinde Curious arşivi bulunamadı.");for(const s of shards){const o=document.createElement("option");o.value=s.shard;o.textContent=`${s.shard} · ${s.compressed?"sıkıştırılmış":"aktif SQLite"} · ${(s.bytes/1048576).toFixed(1)} MiB`;$("shard").append(o);}$("shard").value=String(shards[shards.length-1].shard);await openShard(Number($("shard").value));}catch(e){$("connection").textContent="Arşiv açılamadı";setError(e);}})();

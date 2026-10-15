"use strict";
const $ = (id) => document.getElementById(id),
  canvas = $("world"),
  ctx = canvas.getContext("2d");
const names = { Herbivore: "Otçul", Carnivore: "Etçil", Omnivore: "Hepçil" };
const colors = {
  Herbivore: "#bddc94",
  Carnivore: "#eca080",
  Omnivore: "#b7a0db",
};
const reasons = {
  SeekShelter: "Gördüğü sığınağa kaçıyor",
  EscapeThreat: "Tehlikeden uzaklaşıyor",
  CorneredDefense: "Sıkıştı; kendini savunuyor",
  RecoverStamina: "Yoruldu; dinleniyor",
  HuntLowEnergy: "Av için enerjisi yetmiyor",
  HuntTooCostly: "Bu avın bedeli çok yüksek",
  Defend: "Kendini savunuyor",
  SeekWater: "Suya yöneliyor",
  SearchWater: "Su arıyor",
  RecallWater: "Hatırladığı suya dönüyor",
  RecallFood: "Hatırladığı yiyeceğe dönüyor",
  RecallMate: "Daha önce gördüğü eşin son konumunu arıyor",
  LearnedWater: "Deneyimine göre su seçti",
  SeekFood: "Beslenmeye yöneliyor",
  HuntPrey: "Avını takip ediyor",
  SearchFood: "Yiyecek arıyor",
  ApproachMate: "Eşine yaklaşıyor",
  Mate: "Üreme fırsatını değerlendiriyor",
  NoEligibleMate: "Uygun bir eş bulamadı",
  Explore: "Çevresini araştırıyor",
  Rest: "Dinleniyor",
  ExternalCommand: "Verilen komutu uyguluyor",
  PlayerWaiting: "Komut bekliyor",
  CustomPolicy: "Kendine özgü bir karar verdi",
};
let token = "",
  state = null,
  status = null,
  decisions = [],
  selected = null,
  journal = [],
  busy = false,
  geometry = null,
  online = false;
function message(text) {
  $("error").textContent = text;
  $("error").hidden = !text;
}
async function api(path, body) {
  const headers = {};
  if (token) headers.Authorization = "Bearer " + token;
  if (body) headers["Content-Type"] = "application/json";
  const response = await fetch(path, {
    method: body ? "POST" : "GET",
    headers,
    body: body ? JSON.stringify(body) : undefined,
    signal: AbortSignal.timeout(15000),
  });
  const data = await response.json();
  if (!response.ok) {
    if (response.status === 401) $("auth").hidden = false;
    throw new Error(data.error || "Bağlantı kurulamadı");
  }
  return data;
}
async function control(body) {
  if (busy) return;
  busy = true;
  renderControls();
  try {
    await api("/control", body);
    message("");
    await poll();
  } catch (error) {
    message(
      error.message +
        " · İstek tekrar gönderilmedi. Durumu yenileyerek kontrol et.",
    );
  } finally {
    busy = false;
    renderControls();
  }
}
function renderControls() {
  for (const id of ["toggle", "step", "steps", "speed", "full"])
    $(id).disabled = busy || !online;
  const paused = status?.paused;
  $("toggle").textContent = paused ? "▶ Başlat" : "Ⅱ Duraklat";
  $("step").disabled = busy || !online || !paused;
  $("steps").disabled = busy || !online || !paused;
  if (status) {
    const value = String(status.ticks_per_second);
    if (![...$("speed").options].some((o) => o.value === value))
      $("speed").add(new Option(value + " adım/sn", value));
    $("speed").value = value;
  }
}
function labelEvent(event) {
  const [kind, e] = Object.entries(event)[0];
  return (
    {
      EntityMoved: () => `#${e.id} yer değiştirdi`,
      EntityAte: () => `#${e.id}, ${e.amount} enerji beslendi`,
      EntityDrank: () => `#${e.id}, ${e.amount} su içti`,
      EntityAttacked: () =>
        `#${e.attacker}, #${e.target} bireyine ${e.damage} hasar verdi`,
      EntityMated: () => `#${e.parent1} ve #${e.parent2} → yavru #${e.child}`,
      EntityBorn: () => `#${e.id} doğdu`,
      EntityDied: () =>
        `#${e.id} yaşamını yitirdi (${{ OldAge: "yaşlılık", Combat: "saldırı", Starvation: "açlık", Dehydration: "susuzluk" }[e.cause] || "neden belirsiz"})`,
      ActionRejected: () =>
        `Eylem gerçekleşmedi: ${{ Occupied: "yol dolu", NoPath: "yol yok", Fatigued: "yorgunluk", InsufficientEnergy: "enerji yetersiz", OutOfRange: "hedef uzakta", MissingResource: "kaynak kalmadı", Cooldown: "üreme için dinlenme", Inactive: "hedef etkin değil" }[e.reason] || e.reason}`,
      CorpseConsumed: () =>
        `#${e.eater}, #${e.id} bedenindeki son besini tüketti`,
    }[kind]?.() || kind
  );
}
function renderJournal() {
  const list = $("journal");
  list.replaceChildren();
  const records = journal
    .filter((d) => selected === null || d.id === selected)
    .slice(0, 80);
  if (!records.length) {
    const li = document.createElement("li");
    li.textContent = "Henüz kaydedilmiş bir karar yok.";
    list.append(li);
  }
  for (const d of records) {
    const li = document.createElement("li"),
      time = document.createElement("time"),
      entry = document.createElement("div"),
      text = document.createElement("strong"),
      out = document.createElement("p");
    time.textContent = "Adım " + d.tick;
    text.textContent = `#${d.id} · ${reasons[d.decision.reason] || d.decision.reason}`;
    out.textContent =
      d.outcome?.events.map(labelEvent).join(" · ") ||
      "Ek bir olay kaydedilmedi.";
    entry.append(text, out);
    li.append(time, entry);
    list.append(li);
  }
}
function renderIndividual() {
  const e = state?.entities.find((e) => e.id === selected);
  $("individual-body").hidden = !e;
  if (!e) {
    $("individual-title").textContent =
      selected === null
        ? "Bir birey seç"
        : "#" + selected + " artık haritada değil";
    $("individual-sub").textContent =
      selected === null
        ? "Haritadaki bir canlıya veya altındaki numaraya dokun."
        : "Kayıtlı kararları alan günlüğünden inceleyebilirsin.";
    return;
  }
  $("individual-title").textContent = names[e.species] + " #" + e.id;
  $("individual-sub").textContent =
    `${e.generation}. nesil · ${e.life.age} adımlık yaşam · ${e.life.age >= e.life.maturity_age ? "yetişkin" : "yavru"}`;
  $("vitals").replaceChildren();
  for (const [key, name, color] of [
    ["health", "Sağlık", "#d9a594"],
    ["energy", "Enerji", "#d8d295"],
    ["water", "Su", "#7eb8ca"],
    ["stamina", "Dayanıklılık", "#a0bf93"],
  ]) {
    const div = document.createElement("div");
    div.className = "vital";
    const line = document.createElement("div"),
      title = document.createElement("span"),
      value = document.createElement("span"),
      track = document.createElement("div"),
      fill = document.createElement("i");
    title.textContent = name;
    value.textContent = e.life[key] + " / " + e.life["max_" + key];
    line.append(title, value);
    track.className = "track";
    fill.style.width = (100 * e.life[key]) / e.life["max_" + key] + "%";
    fill.style.background = color;
    track.append(fill);
    div.append(line, track);
    $("vitals").append(div);
  }
  $("details").replaceChildren();
  for (const [key, value] of [
    ["Konum", `${e.pos.x}, ${e.pos.y}`],
    [
      "Ebeveynler",
      e.parents.length
        ? e.parents.map((id) => "#" + id).join(" & ")
        : "Kurucu birey",
    ],
    [
      "Hareket",
      `${e.life.speed} adım / ${e.life.move_energy_cost || 1} enerji`,
    ],
    ["Kalıtsal hareket", e.genome?.mobility ?? 0],
    ["Bir öğünde en çok", `${e.life.max_meal ?? 10} besin`],
    [
      "Üreme için enerji",
      `${Math.floor((e.life.max_energy * (e.life.reproduction_reserve_per_mille ?? 500)) / 1000)} / ${e.life.max_energy}`,
    ],
    [
      "Üreme bekleme süresi",
      `${e.life.reproduction_cooldown} adım (aralık ${e.life.reproduction_interval ?? 25})`,
    ],
  ]) {
    const dt = document.createElement("dt"),
      dd = document.createElement("dd");
    dt.textContent = key;
    dd.textContent = value;
    $("details").append(dt, dd);
  }
  const d = decisions.find((d) => d.id === selected);
  $("reason").textContent = d
    ? reasons[d.decision.reason] || d.decision.reason
    : "Yeni bir karar bekleniyor";
  $("outcome").textContent = d
    ? `Gördükleri: ${d.seen.entities.length} birey, ${d.seen.waters} su, ${d.seen.foods} besin noktası.`
    : "";
  $("knowledge").textContent =
    `${e.memory?.sites.length || 0} kaynak, ${e.memory?.dangers.length || 0} tehlike ve ${e.memory?.companions?.length || 0} türdeş konumu hatırlıyor. Su seçimini öğrenme ${e.learner?.enabled ? "açık" : "kapalı"}.`;
  $("values").replaceChildren();
  for (const v of e.learner?.values || []) {
    const s = document.createElement("small");
    s.textContent = `Su (${v.pos.x}, ${v.pos.y}) · ${v.visits} deneyim · değer ${v.value}`;
    $("values").append(s);
  }
}
function select(id) {
  selected = id;
  renderIndividual();
  renderRoster();
  renderJournal();
  draw();
}
function renderRoster() {
  $("roster").replaceChildren();
  for (const e of state.entities) {
    const b = document.createElement("button");
    b.textContent =
      (e.species === "Carnivore" ? "▲" : e.species === "Omnivore" ? "■" : "●") +
      " #" +
      e.id;
    b.style.color = colors[e.species];
    b.className = e.id === selected ? "selected" : "";
    b.setAttribute("aria-pressed", String(e.id === selected));
    b.title = names[e.species] + ", " + e.life.age + " adım";
    b.onclick = () => select(e.id);
    $("roster").append(b);
  }
}
function draw() {
  if (!state) return;
  const rect = canvas.getBoundingClientRect(),
    dpr = window.devicePixelRatio || 1;
  canvas.width = Math.round(rect.width * dpr);
  canvas.height = Math.round(rect.height * dpr);
  ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
  const width = rect.width,
    height = rect.height;
  ctx.fillStyle = "#14251a";
  ctx.fillRect(0, 0, width, height);
  let [minX, maxX, minY, maxY] = state.bounds;
  const focal = state.entities.find((e) => e.id === selected);
  if ($("follow").checked && focal) {
    const radius = 5;
    minX = Math.max(minX, focal.pos.x - radius);
    maxX = Math.min(maxX, focal.pos.x + radius);
    minY = Math.max(minY, focal.pos.y - radius);
    maxY = Math.min(maxY, focal.pos.y + radius);
  }
  const cell = Math.min(
      (width - 30) / (maxX - minX + 1),
      (height - 30) / (maxY - minY + 1),
    ),
    ox = (width - cell * (maxX - minX + 1)) / 2,
    oy = (height - cell * (maxY - minY + 1)) / 2;
  geometry = { cell, ox, oy, minX, minY, maxX, maxY };
  ctx.strokeStyle = "#294030";
  ctx.lineWidth = 0.5;
  for (let x = minX; x <= maxX + 1; x++) {
    ctx.beginPath();
    ctx.moveTo(ox + (x - minX) * cell, oy);
    ctx.lineTo(ox + (x - minX) * cell, oy + (maxY - minY + 1) * cell);
    ctx.stroke();
  }
  for (let y = minY; y <= maxY + 1; y++) {
    ctx.beginPath();
    ctx.moveTo(ox, oy + (y - minY) * cell);
    ctx.lineTo(ox + (maxX - minX + 1) * cell, oy + (y - minY) * cell);
    ctx.stroke();
  }
  const point = (p) => [
      ox + (p.x - minX + 0.5) * cell,
      oy + (p.y - minY + 0.5) * cell,
    ],
    inside = (p) => p.x >= minX && p.x <= maxX && p.y >= minY && p.y <= maxY;
  for (const site of state.environment?.sites || []) {
    if (
      !inside(site.pos) ||
      state.resources.some(([p]) => p.x === site.pos.x && p.y === site.pos.y)
    )
      continue;
    const [x, y] = point(site.pos);
    ctx.strokeStyle = site.kind === "Water" ? "#477d8d" : "#466d4a";
    ctx.setLineDash([3, 3]);
    ctx.strokeRect(x - cell * 0.43, y - cell * 0.43, cell * 0.86, cell * 0.86);
    ctx.setLineDash([]);
    if (cell > 25) {
      ctx.fillStyle = site.kind === "Water" ? "#7eb8ca" : "#8da878";
      ctx.font = "10px system-ui";
      ctx.textAlign = "center";
      ctx.fillText("0", x, y + 4);
    }
  }
  for (const [p, c] of state.resources) {
    if (!inside(p)) continue;
    const [x, y] = point(p),
      kind = typeof c === "string" ? c : Object.keys(c)[0];
    ctx.fillStyle =
      kind === "Water"
        ? "#3a6d78"
        : kind === "Wall"
          ? "#63695c"
          : kind === "Burrow"
            ? "#91775a"
            : "#396044";
    ctx.globalAlpha = kind === "Wall" ? 1 : 0.8;
    ctx.fillRect(x - cell * 0.43, y - cell * 0.43, cell * 0.86, cell * 0.86);
    ctx.globalAlpha = 1;
    if (cell > 25 && kind !== "Wall" && kind !== "Burrow") {
      ctx.fillStyle = kind === "Water" ? "#b1d9e2" : "#a2c982";
      ctx.font = `${Math.min(12, cell * 0.25)}px system-ui`;
      ctx.textAlign = "center";
      ctx.fillText(String(c[kind].amount), x, y + 4);
    }
  }
  for (const e of state.entities) {
    if (!inside(e.pos)) continue;
    const [x, y] = point(e.pos),
      r = Math.max(3, Math.min(13, cell * 0.24));
    ctx.save();
    ctx.translate(x, y);
    ctx.globalAlpha = e.life.health > 0 ? 1 : 0.3;
    if (e.id === selected) {
      ctx.strokeStyle = "#edf0cc";
      ctx.lineWidth = 2;
      ctx.beginPath();
      ctx.arc(0, 0, r + 6, 0, Math.PI * 2);
      ctx.stroke();
    }
    ctx.fillStyle = colors[e.species];
    ctx.beginPath();
    if (e.species === "Carnivore") {
      ctx.moveTo(0, -r);
      ctx.lineTo(r, r);
      ctx.lineTo(-r, r);
      ctx.closePath();
    } else if (e.species === "Omnivore") {
      ctx.rect(-r, -r, r * 2, r * 2);
    } else {
      ctx.arc(0, 0, r, 0, Math.PI * 2);
    }
    ctx.fill();
    if (cell > 27) {
      ctx.fillStyle = "#132217";
      ctx.font = "bold 10px system-ui";
      ctx.textAlign = "center";
      ctx.fillText(e.id, 0, 4);
    }
    ctx.restore();
  }
}
async function poll() {
  try {
    const snapshot = await api("/observation"),
      s = snapshot.status,
      w = snapshot.state,
      d = snapshot.decisions;
    if (!online) message("");
    status = s;
    state = w;
    decisions = d;
    online = true;
    $("connection").textContent = s.paused
      ? "● Dünya duraklatıldı"
      : "● Yaşam sürüyor";
    $("world-label").textContent = s.paused ? "DURAKLATILDI" : "CANLI";
    $("tick").textContent = s.tick.toLocaleString("tr");
    $("alive").textContent = s.alive;
    $("generation").textContent = Math.max(
      0,
      ...w.entities.filter((e) => e.life.health > 0).map((e) => e.generation),
    );
    $("map-empty").hidden = s.alive > 0;
    if (s.storage_error) message("Kayıt hatası: " + s.storage_error);
    for (const trace of d) {
      if (
        !journal.some((old) => old.tick === trace.tick && old.id === trace.id)
      )
        journal.unshift(trace);
    }
    journal.sort((a, b) => b.tick - a.tick || a.id - b.id);
    journal = journal.slice(0, 500);
    renderRoster();
    renderIndividual();
    renderJournal();
    draw();
  } catch (error) {
    online = false;
    $("connection").textContent = "● Bağlantı bekleniyor";
    message(error.message);
  }
  renderControls();
}
$("toggle").onclick = () =>
  control({ action: status?.paused ? "resume" : "pause" });
$("step").onclick = () => control({ action: "step", ticks: 1 });
$("steps").onclick = () => control({ action: "step", ticks: 10 });
$("speed").onchange = () =>
  control({ action: "speed", ticks_per_second: Number($("speed").value) });
$("follow").onchange = draw;
$("full").onclick = () =>
  selected !== null &&
  control({ action: "observation", full_all: false, full_for: [selected] });
$("auth-open").onclick = () => ($("auth").hidden = !$("auth").hidden);
$("auth").onsubmit = (event) => {
  event.preventDefault();
  token = $("token").value;
  $("token").value = "";
  $("auth").hidden = true;
  message("");
  poll();
};
canvas.onclick = (event) => {
  if (!geometry || !state) return;
  const rect = canvas.getBoundingClientRect(),
    g = geometry,
    x = Math.floor((event.clientX - rect.left - g.ox) / g.cell) + g.minX,
    y = Math.floor((event.clientY - rect.top - g.oy) / g.cell) + g.minY;
  const e = state.entities.find((e) => e.pos.x === x && e.pos.y === y);
  if (e) select(e.id);
};
$("history").onclick = async () => {
  if (selected === null) {
    message("Önce bir birey seç.");
    return;
  }
  const id = selected;
  $("history").disabled = true;
  try {
    const shards = await api("/databases");
    let records = [];
    for (const shard of [...shards].reverse()) {
      const result = await api("/query", {
        shard: shard.shard,
        sql: `SELECT d.trace_json FROM decision_traces d JOIN records r ON r.id=d.record_id WHERE r.kind='tick' AND d.entity_id=${Number(id)} ORDER BY d.record_id DESC LIMIT 50`,
      });
      for (const row of result.rows) records.push(JSON.parse(row[0]));
      if (records.length >= 50) break;
    }
    if (selected === id) {
      journal = [...records, ...journal.filter((d) => d.id !== id)]
        .sort((a, b) => b.tick - a.tick)
        .slice(0, 500);
      renderJournal();
      $("journal-note").textContent =
        "Kalıcı kayıtlardan #" + id + " bireyinin son kararları.";
    }
    message("");
  } catch (error) {
    message("Geçmiş getirilemedi: " + error.message);
  } finally {
    $("history").disabled = false;
  }
};
new ResizeObserver(draw).observe(canvas);
(async function loop() {
  await poll();
  setTimeout(loop, 1000);
})();

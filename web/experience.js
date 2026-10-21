/* Shared read-only presentation of recorded individual learning. No network or state writes. */
(function (root, factory) {
  const api = factory();
  if (typeof module === "object" && module.exports) module.exports = api;
  else root.CuriousExperience = api;
})(typeof globalThis !== "undefined" ? globalThis : this, function () {
  "use strict";
  const number = value => typeof value === "number" && Number.isFinite(value) ? String(value) : "veri yok";
  const position = value => value && Number.isFinite(value.x) && Number.isFinite(value.y) ? `(${value.x}, ${value.y})` : "veri yok";
  const enabled = value => value === true ? "açık" : value === false ? "kapalı" : "veri yok";
  const waterEnds = {Reached:"Su içti",Failed:"Kaynak bulunamadı",TimedOut:"Süre sınırı doldu",Interrupted:"Kesildi: öncelik veya görünen hedef değişti",Disabled:"Kesildi: öğrenme kapatıldı",Died:"Kesildi: yaşam sona erdi"};
  function water(learner, reason) {
    const lines = [`Su seçimini öğrenme: ${enabled(learner?.enabled)}.`];
    if (!learner || typeof learner !== "object") { lines.push("Bu kayıtta su deneyimi verisi yok."); return lines; }
    lines.push(`Su değerlerini güncelleme: ${enabled(learner.update_enabled)}${learner.update_enabled === false ? " (öğrenilmiş tercihler kullanılır, tablo sabit tutulur)" : ""}.`);
    if (reason === "LearnedWater") lines.push("Bu karar su öğrenicisinden geldi. Kayıt, keşif ile en yüksek değeri seçme nedenini ayırmıyor.");
    const attempt = learner.episode;
    if (attempt) lines.push(`Devam eden su denemesi: ${position(attempt.target)} · ${number(attempt.ticks)}/20 adım · ${number(attempt.movement_energy)} hareket enerjisi. Henüz sonuçlanmadı.`);
    const result = learner.last_episode;
    if (result) {
      lines.push(`Son su denemesi: ${waterEnds[result.end] || "Sonuç türü bilinmiyor"} · hedef ${position(result.attempt?.target)} · ${number(result.attempt?.ticks)} adım · ${number(result.water)} su · ${number(result.attempt?.movement_energy)} hareket enerjisi.`);
      lines.push(`Dış saldırı hasarı: ${number(result.attempt?.incoming_damage)} (su puanından düşülmez). ${result.reward == null ? "Puan güncellemesi yok; bu kayıt başarısızlık puanı değildir." : `Deneme puanı: ${number(result.reward)}; bu bir yaşam başarısı ölçüsü değildir.`}`);
    } else if (Object.hasOwn(learner,"last_episode")) lines.push("Sonuçlanmış son su denemesi yok.");
    else lines.push("Son su denemesi verisi yok (eski veya eksik kayıt).");
    const values = Array.isArray(learner.values) ? learner.values : null;
    if (!values) lines.push("Su tercih tablosu verisi yok.");
    else if (!values.length) lines.push("Su tercih tablosu boş; kayıtlı puanlanmış deneyim yok.");
    else {
      lines.push("Su tercih tablosu — ziyaret, tamamlanıp puanlanan deneme sayısıdır; başarı sayısı değildir. En çok 32 konum saklanır; yaşam boyu toplam gösterilmez.");
      for (const v of values) lines.push(`Su ${position(v?.pos)} · ${number(v?.visits)} puanlanmış deneme · tercih değeri ${number(v?.value)}.`);
    }
    return lines;
  }
  const huntEnds = {Fed:"Kendi avından besin aldı",TargetLost:"Hedef kayboldu",TimedOut:"Süre sınırı doldu",Interrupted:"Kesildi: başka karar öncelik aldı",Died:"Kesildi: yaşam sona erdi",ForeignKill:"Avı başka birey öldürdü"};
  const bands = ["düşük", "orta", "yüksek"];
  function context(c) {
    if (!c) return "bağlam verisi yok";
    const health = ["bilinmiyor", "tek vuruşluk", "iki vuruşluk", "ikiden fazla vuruşluk"];
    const distance = ["0–1", "2–3", "4+"];
    return `enerji ${bands[c.energy] || "veri yok"}, dayanıklılık ${bands[c.stamina] || "veri yok"}, mesafe ${distance[c.distance] || "veri yok"} adım, görünür av sağlığı ${health[c.health] || "veri yok"}`;
  }
  function huntAttempt(a) {
    return `hedef #${number(a?.target)} · ${number(a?.ticks)}/20 adım · ${number(a?.movement_energy)} hareket + ${number(a?.attack_energy)} saldırı enerjisi · ${number(a?.food)} edinilen besin · ${number(a?.incoming_damage)} dış saldırı hasarı`;
  }
  function hunt(learner, reason) {
    const lines = [`Av seçimini öğrenme: ${enabled(learner?.enabled)}.`];
    if (!learner || typeof learner !== "object") return [...lines,"Bu kayıtta av deneyimi verisi yok."];
    lines.push(`Av değerlerini güncelleme: ${enabled(learner.update_enabled)}${learner.update_enabled === false ? " (öğrenilmiş tercihler kullanılır, tablo sabit tutulur)" : ""}.`);
    if (reason === "LearnedHunt") lines.push("Bu karar av öğrenicisinden geldi. Kayıt, keşif ile en yüksek değeri seçme nedenini ayırmıyor.");
    if (learner.episode) {
      lines.push(`Devam eden av denemesi: ${huntAttempt(learner.episode)}. Henüz sonuçlanmadı.`);
      lines.push(`Başlangıç bağlamı: ${context(learner.episode.context)}.`);
    }
    const result=learner.last_episode;
    if (result) {
      lines.push(`Son av denemesi: ${huntEnds[result.end] || "Sonuç türü bilinmiyor"} · ${huntAttempt(result.attempt)}.`);
      lines.push(`Nedensellik belirsizliği: ${result.attempt?.ambiguous === true ? "var; sonuçtan başarı çıkarılamaz" : result.attempt?.ambiguous === false ? "işaretlenmemiş" : "veri yok"}. ${result.reward == null ? "Deneme puanı yok; eksik puan başarısızlık değildir." : `Deneme puanı: ${number(result.reward)}; yaşam başarısı değildir.`} Tablo güncellendi: ${result.updated === true ? "evet" : result.updated === false ? "hayır" : "veri yok"}.`);
    } else lines.push(Object.hasOwn(learner,"last_episode") ? "Sonuçlanmış son av denemesi yok." : "Son av denemesi verisi yok.");
    const values=Array.isArray(learner.values) ? learner.values : null;
    if (!values) lines.push("Av tercih tablosu verisi yok.");
    else if (!values.length) lines.push("Av tercih tablosu boş; kayıtlı puanlanmış deneyim yok.");
    else {
      lines.push("Av tercihleri birey kimliğine değil görünür durum sınıflarına aittir. Deneme sayısı başarı sayısı değildir.");
      for (const v of values) lines.push(`${context(v?.context)} · ${number(v?.visits)} puanlanmış deneme · tercih değeri ${number(v?.value)}.`);
    }
    return lines;
  }
  function describe(individual, trace) {
    if (!individual) return ["Deneyimi görmek için bir birey seç."];
    return ["Yaş ve nesil, deneyim veya öğrenme başarısı değildir. Aşağıdakiler yalnız seçili kaydın verileridir.",
      ...water(individual.learner, trace?.decision?.reason),
      ...hunt(individual.hunt_learner, trace?.decision?.reason)];
  }
  function render(container, individual, trace) {
    container.replaceChildren();
    for (const line of describe(individual,trace)) {
      const p = container.ownerDocument.createElement("p");
      p.textContent = line;
      container.append(p);
    }
  }
  return {describe, water, hunt, context, render};
});

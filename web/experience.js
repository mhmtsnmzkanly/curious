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
  const huntEnds = {Fed:"Kendi avından besin aldı",TargetLost:"Hedef kayboldu",TimedOut:"Süre sınırı doldu",Interrupted:"Karar veya hedef değişti; varsa puan ayrıca gösterilir",Died:"Kesildi: yaşam sona erdi",ForeignKill:"Avı başka birey öldürdü"};
  const bands = ["düşük", "orta", "yüksek"];
  function context(c) {
    if (!c) return "bağlam verisi yok";
    const health = ["bilinmiyor", "tek vuruşluk", "iki vuruşluk", "ikiden fazla vuruşluk"];
    const distance = ["0–1", "2–3", "4+"];
    return `enerji ${bands[c.energy] || "veri yok"}, dayanıklılık ${bands[c.stamina] || "veri yok"}, mesafe ${distance[c.distance] || "veri yok"} adım, görünür av sağlığı ${health[c.health] || "veri yok"}`;
  }
  function huntAttempt(a) {
    return `hedef #${number(a?.target)} · ${number(a?.ticks)}/20 adım · ${number(a?.movement_energy)} hareket + ${number(a?.attack_energy)} saldırı enerjisi · ${number(a?.food)} edinilen besin · ${number(a?.incoming_damage)} alınan toplam hasar (kaynak ayrımı bu kayıtta yok)`;
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
  function refuge(memory, reason) {
    const lines = [`Sığınak seçimini öğrenme: ${enabled(memory?.refuge_outcomes_enabled)}.`];
    if (!memory || typeof memory !== "object") return [...lines, "Bu kayıtta sığınak deneyimi verisi yok."];
    lines.push(`Sığınak genellemesi: ${enabled(memory.refuge_generalization_enabled)}${memory.refuge_generalization_update_enabled === false ? " (tablo güncelleme kapalı)" : ""}.`);
    if (reason && typeof reason === "string" && reason.includes("Refuge")) {
      lines.push("Bu karar sığınak deneyimi veya kaçış gerekçesinden geldi.");
    }
    const episode = memory.refuge_episode;
    if (episode) {
      lines.push(`Devam eden sığınak denemesi: hedef ${position(episode.target)} · ${number(episode.elapsed_ticks)} adım · ${number(episode.movement_energy)} hareket enerjisi · ${number(episode.threat_damage)} tehdit hasarı. Henüz sonuçlanmadı.`);
    } else if (Object.hasOwn(memory, "refuge_episode")) {
      lines.push("Devam eden sığınak denemesi yok.");
    } else {
      lines.push("Devam eden sığınak denemesi verisi yok.");
    }
    const outcomes = Array.isArray(memory.refuge_outcomes) ? memory.refuge_outcomes : null;
    if (!outcomes) {
      lines.push("Sığınak sonuç tablosu verisi yok.");
    } else if (!outcomes.length) {
      lines.push("Sığınak sonuç tablosu boş; kayıtlı deneyim yok.");
    } else {
      lines.push(`Kayıtlı exact sığınak sonuçları: ${outcomes.length} kayıt.`);
      for (const row of outcomes) {
        lines.push(`Sığınak hedefi ${position(row?.key?.endpoint)} · ${number(row?.completed)} tamamlanma · ${number(row?.safe_arrivals)} güvenli varış · ${number(row?.threat_damage)} hasar · ${number(row?.movement_energy)} hareket enerjisi.`);
      }
    }
    const genOutcomes = Array.isArray(memory.refuge_generalized_outcomes) ? memory.refuge_generalized_outcomes : null;
    if (genOutcomes && genOutcomes.length) {
      lines.push(`Kayıtlı genelleştirilmiş sığınak sonuçları: ${genOutcomes.length} kayıt.`);
      for (const row of genOutcomes) {
        lines.push(`Genelleştirilmiş rota: ${number(row?.completed)} tamamlanma · ${number(row?.safe_arrivals)} güvenli varış · ${number(row?.threat_damage)} hasar · ${number(row?.movement_energy)} hareket enerjisi.`);
      }
    }
    return lines;
  }
  function environment(state, individual) {
    const env = state?.environment || (state && (state.season_cycle || state.day_night || state.signals || state.burrows) ? state : null);
    const tick = typeof state?.tick === "number" && Number.isFinite(state.tick) ? state.tick : 0;
    const entity = typeof individual === "number" && Array.isArray(state?.entities)
      ? state.entities.find(x => x.id === individual)
      : individual;
    let season = "Sabit İklim";
    if (env?.season_cycle && typeof env.season_cycle.period === "number" && env.season_cycle.period > 0) {
      const period = env.season_cycle.period;
      const phase = ((tick % period) + period) % period;
      const quarter = Math.floor((phase * 4) / period);
      const seasons = ["İlkbahar (Spring)", "Yaz (Summer)", "Sonbahar (Autumn)", "Kış (Winter)"];
      season = seasons[quarter] || "Sabit İklim";
    }
    let dayPhase = "Sabit Işık";
    if (env?.day_night && typeof env.day_night.period === "number" && env.day_night.period > 0) {
      const period = env.day_night.period;
      const phase = ((tick % period) + period) % period;
      const dayTicks = env.day_night.day_ticks || 0;
      if (phase === 0) dayPhase = "Şafak (Dawn)";
      else if (phase < dayTicks) dayPhase = "Gündüz (Day)";
      else if (phase === dayTicks) dayPhase = "Alacakaranlık (Dusk)";
      else dayPhase = "Gece (Night)";
    }
    const signalCount = Array.isArray(env?.signals) ? env.signals.length : 0;
    const burrowCount = Array.isArray(env?.burrows) ? env.burrows.length : 0;
    const knownSectorsCount = Array.isArray(entity?.memory?.known_sectors) ? entity.memory.known_sectors.length : 0;
    const contextEntriesCount = Array.isArray(entity?.learner?.context_entries) ? entity.learner.context_entries.length : 0;
    let corpseStage = null;
    if (entity?.phase && typeof entity.phase === "object" && entity.phase.Corpse) {
      const rem = entity.phase.Corpse.remaining ?? 0;
      corpseStage = rem >= 4 ? "Taze (Fresh)" : rem >= 2 ? "Yaşlanan (Aging)" : "Çürük (Putrid)";
    }
    const summary = `${season} · ${dayPhase} · Sinyal: ${signalCount} · İn: ${burrowCount}`;
    const detail = `Merak sektörü: ${knownSectorsCount}/16 · Su bağlamı: ${contextEntriesCount}/4${corpseStage ? ` · Ceset evresi: ${corpseStage}` : ""}`;
    return {
      season,
      dayPhase,
      signalCount,
      burrowCount,
      knownSectorsCount,
      contextEntriesCount,
      corpseStage,
      summary,
      detail,
      lines: [summary, detail],
    };
  }
  function describe(individual, trace) {
    if (!individual) return ["Deneyimi görmek için bir birey seç."];
    return ["Yaş ve nesil, deneyim veya öğrenme başarısı değildir. Aşağıdakiler yalnız seçili kaydın verileridir.",
      ...water(individual.learner, trace?.decision?.reason),
      ...hunt(individual.hunt_learner, trace?.decision?.reason),
      ...refuge(individual.memory, trace?.decision?.reason)];
  }
  function render(container, individual, trace, stateArg) {
    container.replaceChildren();
    const isStateObj = trace && typeof trace === "object" && !Object.hasOwn(trace, "decision") && (Object.hasOwn(trace, "environment") || Object.hasOwn(trace, "tick") || Object.hasOwn(trace, "entities"));
    const traceObj = isStateObj ? null : trace;
    const stateObj = stateArg || (isStateObj ? trace : null);
    for (const line of describe(individual, traceObj)) {
      const p = container.ownerDocument.createElement("p");
      p.textContent = line;
      container.append(p);
    }
    const data = individual;
    const refugeData = data?.refuge;
    const mem = data?.memory;
    const hasRefuge =
      Boolean(refugeData && (refugeData.outcomes?.length || refugeData.rows?.length || refugeData.episode || refugeData.refuge_outcomes_enabled || refugeData.enabled)) ||
      Boolean(mem && (mem.refuge_outcomes?.length || mem.refuge_episode || mem.refuge_outcomes_enabled));

    if (hasRefuge) {
      const title = container.ownerDocument.createElement("p");
      title.textContent = "Sığınak deneyimi";
      container.append(title);

      if (refugeData?.headline) {
        const headline = container.ownerDocument.createElement("p");
        headline.textContent = String(refugeData.headline);
        container.append(headline);
      }
      if (refugeData?.Rule || refugeData?.rule) {
        const rule = container.ownerDocument.createElement("p");
        rule.textContent = String(refugeData.Rule || refugeData.rule);
        container.append(rule);
      }
      if (Array.isArray(refugeData?.rows)) {
        for (const row of refugeData.rows) {
          const rowP = container.ownerDocument.createElement("p");
          rowP.textContent = String(typeof row === "string" ? row : (row?.text || JSON.stringify(row)));
          container.append(rowP);
        }
      }
    }
    if (stateObj?.environment || individual) {
      const envInfo = environment(stateObj, individual);
      const envTitle = container.ownerDocument.createElement("p");
      envTitle.textContent = "Çevre ve V19 Telemetri";
      container.append(envTitle);
      const envSummary = container.ownerDocument.createElement("p");
      envSummary.textContent = envInfo.summary;
      container.append(envSummary);
      const envDetail = container.ownerDocument.createElement("p");
      envDetail.textContent = envInfo.detail;
      container.append(envDetail);
    }
  }
  return {describe, water, hunt, refuge, environment, context, render};
});

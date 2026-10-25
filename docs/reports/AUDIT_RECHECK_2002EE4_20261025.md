# `2002ee4` F1–F8 düzeltmelerinin yeniden denetimi (25 Ekim 2026)

Birincil kategori K8 Motor ve işletim; K1–K6 ile kesişir.

## Kapsam ve yöntem

Bu salt-okunur kaynak incelemesi `2002ee4` HEAD'ini, `19da80a..HEAD` farkını, önceki denetimi ([AUDIT_CA5971B_20261025](AUDIT_CA5971B_20261025.md)) ve düzeltme raporunu ([AUDIT_CA5971B_REMEDIATION_20261025](AUDIT_CA5971B_REMEDIATION_20261025.md)) karşılaştırır. Canlı dünya veya yeni deney çalıştırılmadı; test sonucu aşağıda belirtilmiştir. Aşağıdaki durumlar bu incelemenin kaynak ve mevcut test kanıtına göredir; ekolojik yarar kabulü değildir.

## Bulgular

### F1 — Kısmi

Çağrı yolları bağlanmış: su sonucu bağlam kaydı ([water_outcomes.rs](../../src/systems/water_outcomes.rs#L57)), bağlam tahmini ([learning.rs](../../src/entity/learning.rs#L278)), merak yönelimi ([intent_collection.rs](../../src/systems/intent_collection.rs#L48)) ve ilk-aralık hasar kaydı ([refuge_outcomes.rs](../../src/systems/refuge_outcomes.rs#L205)) normal sistemlerde mevcut. Ancak bağlam tablosuna kaydedilen `reward`, `water*10 - movement_energy*2 - ticks` biçimindeki bölüm ödülüdür; seçim skorunda mevcut foraging baseline'ına eklenir ([learning.rs](../../src/entity/learning.rs#L268-L283)). Bu iki niceliğin aynı ölçekte/karşılaştırılabilir olduğuna dair kaynakta dönüşüm yok. Ayrıca tamamlanan sonuç önceki `last_episode` ile eşitse kayıt atlanıyor; aynı koşulda tekrar eden özdeş örnekler öğrenme sayacına girmez ([water_outcomes.rs](../../src/systems/water_outcomes.rs#L57-L60)). Entegrasyon kısmen çalışır; bağlam puanlaması ve tekrar örneği işleme kanıtı eksik.

### F2 — Kısmi

Politika-aware saldırı çözümleme, ebeveyn bakımı ve alarm çağrıları `World::tick` yolunda ([world/mod.rs](../../src/world/mod.rs#L179), [world/mod.rs](../../src/world/mod.rs#L229-L253)). Sığınak pusu bonusu ise yalnız saldıranın mevcut harita hücresi `Cell::Burrow` ve stamina'sı tamken işler ([interaction.rs](../../src/systems/interaction.rs#L410-L416)); sıradan carnivore'lar burrow kullanamadığından bu koşul olağan kullanımda erişilemez. Mevcut doğrulama yapay burrow/saldıran durumuna dayanıyor. Care ve alarm bağlantısı uygulanmış; pusu davranışı normal World davranışı olarak doğrulanmamış.

### F3 — Düzeltildi; doğrulama notu

Yeni kayıt V19 yazar ve okuyucu yalnız `1..=19` kabul eder ([checkpoint.rs](../../src/simulation/checkpoint.rs#L75-L77), [checkpoint.rs](../../src/simulation/checkpoint.rs#L113-L116)). İncelenen önceki V18 kaynak okuyucusu `1..=18` sınırı kullandığından V19'u sürüm kapısında reddeder. Yeni alanlar V19 roundtrip testinde korunuyor ([persistence.rs](../../tests/persistence.rs#L111-L152)); V19'dan düşük sürümlerde yeni alanlar kapatılıp temizleniyor ([checkpoint.rs](../../src/simulation/checkpoint.rs#L220-L236)). Migration testi gerçek V18 JSON/binary yerine V19 State'in sürüm numarasını 18'e indiriyor ([persistence.rs](../../tests/persistence.rs#L261-L285)); eski binary yeniden çalıştırılmadı. Kod sürüm kapısı eski okuyucunun reddini destekliyor, ancak bu çalışma zamanı uyumluluk koşusu değildir.

### F4 — Düzeltilmiş

Mevsimsel termal maliyet World yaşam döngüsüne eklenmiş; rapordaki reproda sığınaksız ve inli bedenlerin karşılaştırması verilmiş ([düzeltme raporu](AUDIT_CA5971B_REMEDIATION_20261025.md)). Bu, mekanizma entegrasyon kanıtıdır; uzun dönem ekolojik sonuç değildir.

### F5 — Kısmi

Harita hücresi Empty ise kazı fiziksel Burrow açıyor, TTL sona erince güncelleme burrow hücresini kapatıyor ([environment.rs](../../src/environment.rs#L296-L305), [environment.rs](../../src/environment.rs#L431-L439)). Buna karşın geçersiz/sınır dışı konum, dolu hücre veya `ttl=0` için metadata eklenebilir ve fonksiyon başarı döndürür; haritada fiziksel in oluşmayabilir. `from_map` burrow hücrelerini metadata listesine almıyor (`environment.rs:287-294`). World üzerinde açık `dig_burrow` API'si var ([world/mod.rs](../../src/world/mod.rs#L90-L94)), fakat otomatik aktör kazma eylemi yok. Metadata ile fiziksel habitat senkronizasyonu tüm girdiler için güvence altında değil.

### F6 — Kısmi

Ceset evre eşikleri 5 tick ömrüyle uyumlu ve tüketimde tür/evre toksisitesi uygulanıyor ([phase.rs](../../src/entity/phase.rs#L60-L68), [interaction.rs](../../src/systems/interaction.rs#L67-L74)). Enerji çürümesi ise yalnız evre güncellemesinden sonra `remaining == 1` iken bir birim azalıyor ([life_cycle.rs](../../src/systems/life_cycle.rs#L18-L23)); rapordaki “her tick” ifadesini karşılamıyor. Ayrıca toksisite `amount > 0` koşuluna bağlı değil; geçerli ceset yeme çözümlemesinde sıfır enerji alınsa bile hasar uygulanabilir ([interaction.rs](../../src/systems/interaction.rs#L67-L74)). Evre ve toksisite bağlantıları düzeldi, fakat çürüme akışı ve sıfır tüketim sınırı açık.

### F7 — Düzeltilmiş

Gözlem değerleri doğrulamada `-1000..=1000` ile sınırlandırılmış, giriş kaydı clamp ediyor ve tahmin toplamı `i64` kullanıyor ([learning.rs](../../src/entity/learning.rs#L515-L585)). Önceki overflow yolu kapatılmış; bu kaynak incelemesi yeni bir sayısal hata bulmadı. Bu, mekanizma güvenliği kanıtıdır; fayda kanıtı değildir.

### F8 — Kısmi

Testler farklı seedlerle bağımsız `World::with_seed` yörüngeleri kurup deterministik tekrar ve farklı state/RNG kontrolü yapıyor ([k8_throughput_audit.rs](../../tests/k8_throughput_audit.rs#L290-L349), [k8_throughput_audit.rs](../../tests/k8_throughput_audit.rs#L494-L507)). Bu World koşuları ilgili deney bayraklarını açmıyor; karşılaştırılan mekanizma sonuçları seed ile değiştirilen sentetik harness'lerden geliyor. Dolayısıyla F8'in önceki seed/fixture karışıklığı raporda ayrıştırılmış, ancak feature-on çoklu seed karşılaştırması olmadığı için nedensel etkinlik kanıtı eksik; ekolojik üstünlük de gösterilmiyor. Remediation raporunun ekolojik üstünlük ve uzun vadeli kalıcılığı kanıtlanmamış sayması yerinde ([düzeltme raporu](AUDIT_CA5971B_REMEDIATION_20261025.md)).

## Sonuç

F3 ve F7 düzeltildi; F1, F2, F5, F6 ve F8 kısmi kaldı; F4 düzeltildi. Root tarafından `cargo test --locked --all-targets --jobs 1` çalıştırıldı: 392 geçti, 0 başarısız oldu (özet grupları: 68). Günlük `data/operations/recheck-2002ee4-20261025/rust-tests.log` olarak korundu. Bu test sonucu statik incelemede işaretlenen F1/F2/F5/F6/F8 boşlukları için ayrı bir nedensel repro değildir. Canlı servislerin systemd durumu salt-okunur kontrol edildi; ikisi de aktifti. Canlı davranış değerlendirilmedi ve servisler değiştirilmedi. Bulgular kaynak ve mevcut test tanımlarına dayanır.

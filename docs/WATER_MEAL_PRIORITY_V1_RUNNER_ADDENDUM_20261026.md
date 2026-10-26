# Yem–su önceliği V1 çalıştırıcı düzeltme eki

**Kimlik:** `water-meal-priority-v1-runner-copy2-fix-20261026`  
**Durum:** Çalıştırıcı hatası tanımlandı ve düzeltildi; bu ek belge World çalıştırmaz, sonuç bildirmez veya protokolü değiştirmez.

## Önceki kayıt

İlk kayıtlı çalıştırıcı denemesi, `6f033f6` kaynak revizyonunda `shutil.copy2` çağrısında `NameError: name 'shutil' is not defined` hatasıyla durdu. Hata registration dosyasını yeni çıktı dizinine kopyalarken, ilk World süreci başlatılmadan meydana geldi. Bu denemede **0 World ve 0 World tick'i** tamamlandı; biyolojik ölçüm veya deney sonucu yoktur. İlk denemenin kayıt, manifest ve stderr dosyaları başarısız/eksik kanıt olarak korunur.

## Düzeltme ve sınır

`scripts/run_water_meal_priority_lab.py` standart `shutil` modülünü içe aktarır. `scripts/test_water_meal_priority_lab.py` içindeki yeni sınır testi runner'ı geçici dizinlerde çalıştırır; kaynak kimliği ve seed taramasını mock'lar, `subprocess.Popen` çağrısını World başlatmayan bir sentinel hata ile durdurur, ardından registration kopyasının ve `status=incomplete` manifestinin oluştuğunu doğrular. Bu test, eksik `shutil` importunun neden olduğu hatayı saptar ve World süreci başlatmaz.

Bu düzeltme için formal V1 protokolü, eşikler, kaynaklar, tohumlar, deney kolları, süre veya analiz değiştirilmemiştir. Sonraki bir kayıt, düzeltilmiş kaynak ağacının yeni revision ve içerik kimliğine bağlanmalı ve kayıt incelemesinden geçmelidir. Önceki registration ve sıfır-tick başarısız deneme yeniden yazılamaz veya başarıya yükseltilemez. Bu ek belge yeni bir çalıştırma için yetki veya kayıt yerine geçmez.

Herhangi bir sonraki V1 çalışması aynı tasarımı kullanır: `62501..62506` tohumları, her birinin dört kayıtlı kolu, üç fixture ve her World için 300 tick (72 World / 21.600 planlanmış tick). Bu sayılar protokolden aktarılmıştır; bu düzeltme kapsamında hiçbir World çalıştırılmamıştır. Önceki başarısız deneme ve teşhisi, [dondurulmuş V1 protokolünden](WATER_MEAL_PRIORITY_V1_PROTOCOL_20261026.md) ayrı tutulur.

## Kaynak kimliği kapsamı

Runner `SOURCE_PATHS` aşağıdaki sabit girdileri kaynak kimliğine bağlar; buna ek olarak `src/` altındaki bütün Git-izli Rust dosyalarını da hash'ler:

```text
Cargo.toml
Cargo.lock
examples/water_meal_priority_lab.rs
scripts/run_water_meal_priority_lab.py
scripts/test_water_meal_priority_lab.py
scripts/run_ecology_traits_lab.py
scripts/evidence_archive.py
docs/EVIDENCE_ARCHIVES.md
docs/WATER_MEAL_PRIORITY_V1_PROTOCOL_20261026.md
docs/WATER_MEAL_PRIORITY_V1_REGISTRATION_TEMPLATE.json
docs/README.md
docs/WATER_MEAL_PRIORITY_V1_RUNNER_ADDENDUM_20261026.md
scenarios/meadow.json
scenarios/ecology-traits-lab.json
```

Herhangi bir yeni registration, bu dosyalar ve izli `src/` girdileri için manifestte kaydedilen yeni içerik hash'lerini kullanmalıdır. Bu ek belgeyi hash kapsamına eklemek de önceki kayıt kimliğini geçersiz kılar.

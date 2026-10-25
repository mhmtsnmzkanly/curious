# Su uyum labı V2 — geçerli deney, ek öğrenme faydası gösterilmedi

Birincil kategori K3; K1/K7/K8 ile kesişir. Kullanıcı V1'in gıda erişim kilitlenmesinden sonra yeni fixture'ı alt ajanlarla kurmayı istedi. Kaynak `dd7c65d`; [önceden sabitlenen V2 protokolü](../WATER_ADAPTATION_V2_PROTOCOL_20261025.md). V1 kodu/protokolü/ham sonucu değişmedi. Yeni motor, ödül veya öğrenici yazılmadı; canlıya dağıtım yoktur.

## Tasarım ve doğrulama

V1'deki tek gıda noktası yerine su dışındaki89 hücreye normal, sonlu, yenilenebilir gıda kondu. Hareket/enerji/metabolizma kuralları aynı. Normal World testleri, sıfır enerjili canlı için hem altında hem bitişiğinde gıdayı hareket etmeden yemenin mümkün olduğunu doğruladı. Bu yapay gıda-zengin çalışma su kararını açlık erişiminden ayırır; ekoloji/kıtlık gerçekçiliğini ölçmez.

Ortak200tick öğrenme geçmişinden aynı beden/ordinary memory/harita/RNG ile dört200tick kol: trained_online, trained_frozen, naive_online ve rule_based. A(-2,0)'nın yenilenmesi fork'ta4/2tick'ten1/12tick'e, B(4,0)'nınki1/12tick'ten4/2tick'e değişir; A/B miktarları1/12 olarak bir kez ayarlanır. Ardından beden/konum/kaynak sıfırlaması yapılmaz. V19 context öğrenmesi açık, V17 residual seçimi kapalıdır. Frozen tablo sabittir fakat48tick tahmin TTL'si işler.

Kaynak/binary/protokol aynı SHA ile kullanıldı; temiz kaynak önce commit edildi. Seed taraması1399 metin dosyasında yeni62299 ve62201..62206 kayıtlarını bulmadı; bunun kapsamı dışına küresel tazelik iddiası taşınmaz. Bir kalibrasyon ve tek formal batch dışında tekrar/tuning yoktur. Toplam7000 lab World tick, ek2 fixture doğrulama tick'i yürüdü.

## Kalibrasyon ve formal geçerlilik

Kalibrasyon62299 geçti: prefixte9 taze iki-kaynak seçimi; online ve frozen suffixte ayrı ayrı4 seçim. Dört kol200tick suffix boyunca yaşadı; ortak başlangıç eşitliği ve frozen tabloların değişmediği doğrulandı. Kalibrasyon fiziksel fayda kanıtı sayılmaz.

Formal62201..62206 tek kez çalıştı. Bütün kollar ve seedler200tick suffix boyunca yaşadı; bütün frozen ve başlangıç eşitliği kontrolleri geçti. Seed62206'da prefixte yalnız4 iki-kaynak seçimi olduğundan >=5 destek kapısı geçilmedi. Bu seed gizlenmedi veya değiştirilmedi: betimsel tabloda tutuldu, yeterli geçmiş isteyen öğrenme kabulünden ayrı işaretlendi. Diğer beş seed bütün geçerlilik kontrollerini geçti.

## Ana sonuç: öğrenilmiş geçmiş davranışı değiştirmedi

Altı seedin her birinde trained_online, trained_frozen ve naive_online kollarının200 suffix tick'inin tamamında gerçek eylem/hedefleri, learner dışı State ve RNG birebir aynıydı. Yalnız sayısal metriklerin eşitliği değil, ham yörünge karşılaştırması da bunu doğruladı. Öğrenici tabloları/audit kayıtları değişebilir; gerçek davranışla aynı şey değildir. Eş beden/RNG/algı altında farklı hedef ve farklı etkin context tahmini gerektiren mekanizma şahidi hiçbir seedde bulunmadı.

Suffix boyunca online tablosunda gözlenen context örnekleri beş seedde yalnız0;62204'te0 ve67 idi. Bu küçük sinyalin bir hedef sıralaması değişikliği ürettiği gösterilmedi. Buradan tüm öğrenmenin işe yaramadığı veya motorun bozuk olduğu çıkarılmaz: bu fixture'da ek deneyimin karar/fayda etkisi gösterilemedi.

### Altı formal seedin betimsel toplamı

| Kol | Yaşayan birey-tick | Su kazanımı | Hareket enerjisi | Susuzluk /sıfır-su tick | Terminal enerji /sağlık /su |
|---|---:|---:|---:|---:|---:|
| trained_online |1200|1178|991|0 /0|369 /720 /289|
| trained_frozen |1200|1178|991|0 /0|369 /720 /289|
| naive_online |1200|1178|991|0 /0|369 /720 /289|
| rule_based |1200|1182|946|0 /0|384 /720 /293|

Bu toplamlar62206 düşük destekli seedini de içerir; yalnız betimsel toplamdır. Kurallı kol bu toplam vektörde daha çok su,45 daha az hareket enerjisi, daha yüksek terminal enerji/su ile hakimdir; bütün seedlerde tek tek üstün demek değildir. Bütün kollarda susuzluk ve sıfır-su0 olduğundan bu bileşenlerde taban etkisi vardır.

### Seed başına online politika–kurallı politika karşılaştırması

| Seed | Prefix /online suffix taze iki-kaynak seçimi | Online vs rule | Deneyim atfı |
|---|---:|---|---|
|62201|5 /10|Ödünleşim|Frozen/naive aynı; öğrenme şahidi yok|
|62202|6 /4|Online politika Pareto hakim|Frozen/naive de aynı kazancı verir|
|62203|6 /8|Ödünleşim|Öğrenme şahidi yok|
|62204|7 /7|Kurallı politika Pareto hakim|Öğrenme şahidi yok|
|62205|6 /5|Ödünleşim|Öğrenme şahidi yok|
|62206|4 /9|Online politika hakim, fakat prefix desteği yetersiz|Öğrenme kabulüne katılmaz|

Geçerli beş seedde1 politika kazanımı,1 kayıp ve3 ödünleşim var; online-frozen karşılaştırması hepsinde fiziksel olarak nötr. Öğrenici-enabled politikanın seçim/rastlantı akışı ile kurallı politikanın farkı, deneyim güncellemesinin faydası olarak sunulmaz. Yeni deneyim, eski deneyim ve updates-frozen kollar aynı kalırken rule farkı öğrenilmiş bilgiye bağlanamaz. Önceden tanımlanan öğrenme varlık-örneği kapısı geçilmedi.

## Replay, testler ve kanıt

Mevcut yerel `web/water-lab.html` dört kolu aynı201..400 zaman çizgisinde gösterir. Altı gerçek formal replay bundle'ı doğrulandı ve her kolun201/400 endpointleri projeksiyonda kontrol edildi. Replay State'in yalnız tick/bounds/resources/entities ve karar id/pos/decision alanlarını taşır; tam Environment/algı raw JSON'dadır. Fake olay, canlı yazma veya veri yükleyen ağ çağrısı yoktur. Gerçek tarayıcı/screenshot testi yapılmadı.

Yerel paket: `data/experiments/water-adaptation-v2-presentation-20261025/water-lab-v2-replay.zip`. ZIP'i çıkarıp water-lab.html açın; seed-62201.replay.json ve diğer kayıtları seçin. İlk kayıt önceden ilan edilen ilk formal seed'dir; en iyi sonuç seçilmedi. Dosya tüm altı seed'i,62206 uyarısını ve kullanım notunu içerir.

2 Rust fixture testi ve14 Python analiz testi geçti. Kalibrasyon ve formal stderr dosyaları boş; raw SHA'lar yeniden doğrulandı; source/binary/protokol SHA'ları iki fazda aynı. Mevcut replay Node kontrolü ve gerçek altı paket projeksiyonu geçti. Tüm Rust regresyon paketi bu görevde yeniden çalıştırılmadı; çekirdek motor değiştirilmedi.

Ham kanıt dizinleri `data/experiments/water-adaptation-v2-{preflight,calibration,formal,presentation}-20261025/`: seed taraması, build/test logları, manifestler, korunmuş probe, ham JSON, validity/Pareto özetleri, action-target ve trajectory karşılaştırmaları, replay paketleri. Yerel/ignore edilen raw veriler Git'e gönderilmez; Git'teki JSON özet bunların yerine geçmez. Başarılı V2, başarısız V1'i silmez.

## Karar

Fixture artık ölçüm yapabiliyor; bu çalışma öğrenme yararını kabul ettirmedi. Aynı kimlikle yeni seed, uzatma veya ödül/parametre tuning'i yapılmadı. Canlı7882/7883 servisleri ve V12 ikilisi değiştirilmedi. Sonraki inceleme, çoğunlukla sıfır olan artık sinyalinin neden mevcut algı/foraging skoruna ek bilgi katmadığını açıklamalı; yeni sweep veya olumlu sonuç üretmek için baseline'ı zayıflatmak bu çalışmanın devamı değildir.

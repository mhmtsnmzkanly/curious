# 2 — Av kararları

Av seçimi `entity/hunting.rs` altında bağımsız bir politika oldu. Görüşteki avın
sağlık bilgisi, mesafe ve saldırı maliyetiyle tahmini yaklaşma/öldürme bütçesi
hesaplanır. Yaralı, daha az bedel gerektiren av tercih edilir. Uzaktaki pahalı av
bırakılır; bitişik avda son karşılanabilir saldırı mümkündür. Dayanıklılık düşünce
dinlenir. Görüş dışı avın gerçek konumu okunmaz. Bunlar öğrenilmiş tercihler değil.

Hedef seçimi, maliyet nedeniyle vazgeçme, dayanıklılık ve köşede savunma testleri
geçti. Dört seed × 1.000 tick küçük karma çayırda tüm dünyalar 175–196 aralığında
tükendi. Başarılı avcı bütün avı tüketebiliyor; daha çok av öldürmek ekosistemin
daha iyi olduğu anlamına gelmez. Ham veri `data/experiments/roadmap-2-hunting`.

Aşama 3: aynı motorla nüfus, harita ve kaynak dağılımını karşılaştır; başarısız
başlangıçları sakla. Nüfus destekleyerek veya ölüm engelleyerek denge taklidi yapma.

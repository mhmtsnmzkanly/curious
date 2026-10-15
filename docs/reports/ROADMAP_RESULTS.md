# 11 başlığın uygulama ve ölçüm sonucu

Bu rapor bütün hedefin bittiğini söylemez. Kalıcı avcı–av birlikte yaşamı ile
gerçek gün/hafta dayanıklılığı açık; diğer başlıklarda ilk çalışan uygulamalar var.

| Başlık | Gerçek sonuç |
|---|---|
| 1. Yavru | Açken iyileşme gideri düzeltildi; etçil yavrular olgunlaşıp üreyebiliyor. Kalıcı soy yok. |
| 2. Av kararları | Yerel yaralanma/maliyet, karşılanabilir av ve dinlenme ayrı politikada. |
| 3. Karma dünya | Farklı yerleşimler ve aynı koşullarda sığınak karşılaştırması; kalıcı denge yok. |
| 4. Sözleşmeler | Her gerçek yol adımı tekrar denetleniyor; duvar/sığınak, diyet ve sonlu tüketim. |
| 5. Hafıza | Görülen kaynak/tehlike, sınırlı kayıt, eski bilgi ve görülen tükenmeyi unutma. |
| 6. Öğrenme | Su seçimi banditi; son 100 kontrollü denemede 970–1.000 su, öğrenmeyen 500. |
| 7. Kalıtım | Hareket alleli, gerçek enerji bedeli, ebeveyn aktarımı, %2 mutasyon ve dağılım ölçümü. |
| 8. Gözlem | Masaüstü/telefon arayüzü; birey, karar, beden, hafıza, deneyim, ebeveyn ve geçmiş. |
| 9. Dayanıklılık | 10.000 kayıtlı tick, 304. nesil; gerçek süreli ayrı koşu ve örnekleme sürüyor. |
| 10. Ölçek | 1.000 kurucuda motor p95 yaklaşık 71–84 ms → 19–29 ms; SQLite hariç. |
| 11. İşletim | Yedekli V3→V7→V8 laboratuvar göçü; ayrı monitör gözetimi ve gerçek gün raporları. |

Sığınaklı sekiz koşuda otçullar 4.000 tick sürdü. Bir etçil soy 20. nesle ulaşsa
da hepsi sonunda kesildi. Bu sınır, öğrenme/kalıtım eklenince de kendiliğinden
çözülmedi. Sayılar REFUGE.json ve ROADMAP_FINAL_ECOLOGY.json içinde.

Doğrulama: **40 Rust testi** (39 çekirdek/entegrasyon + 1 öğrenme örneği),
**11 Python testi**, HTTP auth/Origin/kalıcı müdahale/terrain/SIGKILL/arşiv ve
Playwright ile masaüstü/telefon, geçmiş, adım, hız, pause/resume. Arayüz sığınaklı
senaryoda ayrıca denendi. Gerçek 7881 dünyası da tarayıcıdan salt okunur incelendi.

## Çalışan servisler

- 7878: eski kurucular, pause, eski motor.
- 7879: eski sürekli çayır, eski motor, kesintisiz 0,1 tick/s.
- 7880: V8 laboratuvar, pause, tick 30 korunmuş; gözlem ekranı `/`.
- 7881: V7 deneyim/kalıtım çayırı, 0,1 tick/s; gerçek uzun gözlem için sürüm
  sabit bırakıldı. Buraya sığınak deneyi eklenmedi. Gözlem ekranı `/`.

Adresler sunucunun loopback arayüzündedir. Uzaktan yerel tarayıcı erişimi örneği:
`ssh -L 7881:127.0.0.1:7881 root@SUNUCU`, sonra `http://127.0.0.1:7881/`.

V7 gerçek süreli koşu 2026-10-15 11:47:50 UTC'de başladı. Örnekler
`data/soak-evolution`; 1/7/30 günlük yeterli zaman kapsamı oluştuğunda Notify
otomatik özet verir. Bu, ajanın arka planda yeni kod yazdığı anlamına gelmez;
servisler simülasyonu, ölçümü ve uyarıları sürdürür.

40 testin geçmesi, aylarca güvenilirlik veya açık uçlu evrim garantisi değildir.
Yeni davranışa geçiş için eski veri+ikili yedeği tutulur. Deney geçmişleri Git
dışında, `data/experiments` altında korunur; derleme temizliğiyle silinmez.

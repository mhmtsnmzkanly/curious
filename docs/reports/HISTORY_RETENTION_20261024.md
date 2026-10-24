# Kayıt saklama ve temizlik — 24 Ekim 2026

**Sonuç:** Kullanıcının onayladığı eski gözlem geçmişi temizlendi; son durumlar korundu. Önemli deney kayıtları kayıpsız sıkıştırıldı ve yeni ayrıntılı kayıtlar için süre/boyut politikası kuruldu. Doğrulama 24 Ekim 2026, 10:58 UTC'de tekrarlandı. Uygulama kaynağı: `904d4cc`.

## Eski dünyalar

Durdurulmuş `main`, `meadow`, `lab` ve `evolution` dünyalarının ayrıntılı tick geçmişleri kaldırıldı. Her dünyadan son durumun özgün metni, son metrikler, kaynak dosyaların hash envanteri ve tek kayıt içeren küçük bir SQLite veritabanı korundu. Son tickler sırasıyla 10, 77.974, 30 ve 74.075. Korunan veritabanları `quick_check` kontrolünü geçti; durum metinlerinin hashleri özgün son kayıtla eşleşti. Mevcut inceleme örnekleri ve kaynak/ikili sürümler korundu.

Eski ayrıntılı kayıtlardan net 929.698.848 byte kaldırıldı. Checkpoint yedekleri 292.787 byte; plan ve doğrulama kanıtı `data/retired-worlds/20261024T015208Z/plan.json` içindedir. Bu temizleme, eski dünyaların tam geçmişinin artık oynatılamayacağı anlamına gelir.

## Deneylerin korunması

8 MiB veya daha büyük 21 tamamlanmış deney JSONL dosyası Zstandard seviye 3 ile sıkıştırıldı:

| Ölçüm | Değer |
| --- | ---: |
| Özgün JSONL toplamı | 1.060.085.984 byte |
| Sıkıştırılmış toplam | 52.274.679 byte |
| Bu arşivlemeden kazanılan alan | 1.007.811.305 byte |
| Değişmeden kalan özgün metadata dosyası | 710 |
| Mevcut tarayıcıda görünürlüğü doğrulanan seed değeri | 2.607 |

Her kaynak kaldırılmadan önce sıkıştırılmış dosya tamamen geri açılarak özgün boyut ve SHA-256 ile karşılaştırıldı. Özgün manifest, protokol ve sonuç dosyaları değiştirilmedi. `.jsonl.archive.json` indeksleri özgün/sıkıştırılmış hashleri ve seed rollerini korur. Son denetimde 21 arşivin hashleri ve 710 metadata dosyasının değişmediği tekrar doğrulandı.

Gerçek kalibrasyon kaydı `calibration-53099.jsonl` ayrıca geri açıldı: özgün hash `29d23f7b7a140beac70519faf18eb6392466e7dab764824d72c04a835f1491bd` eşleşti. Dondurulmuş `466f9a0` doğrulayıcısı aynı 200 ticki okuyup kapıları yeniden doğruladı; yeni simülasyon tick'i çalıştırılmadı. Dosya ardından yeniden arşivlendi. Eski analiz araçlarını kullanmadan önce [geri açma komutu](../EVIDENCE_ARCHIVES.md) uygulanmalıdır.

## Gelecekteki kayıtlar

`curious-history-retention.timer` 15 dakikada bir çalışır. `data/coexistence` altındaki kapalı sıkıştırılmış SQLite arşivleri için 7 günlük süre ve 3 GiB boyut hedefi uygulanır; her çalışmada en fazla iki parça işlenir. Ayrıntı silinmeden önce ilk/son tam durum, olay türü toplamları, son metrikler ve en fazla 20 seçilmiş yaşam/hata olayı sıkıştırılmış kalıcı özete kaydedilir ve doğrulanır. Seçilen olaylar tam yaşam öyküsü değildir; eksik bırakılan örneklerin sayısı belirtilir.

Aktif ham SQLite, WAL/SHM ve en yeni parça korunur; bozuk veya desteklenmeyen biçimler silinmez. Bu nedenle **3 GiB bütün veri klasörünün kesin üst sınırı değildir**; korunan dosyalar ve kalıcı özetler ayrıca yer kaplar. Dondurulmuş motorun `archived_bytes` sayacı dış temizlikten sonra eski değeri tutabilir; bu çalışma motorun uzun süreli bütçe davranışını tamamen çözmez. Motor bu işlem için yeniden başlatılmadı.

Dondurulmuş saklama betiği `/root/curious-runtime/releases/history-retention-904d4cc/history_retention.py` konumunda; SHA-256: `2331196cec3c827f03546102265bd1b331b2c7d81bcf1cff8f5ecd6d3de8bee9`. İlk çalıştırma ve son periyodik çalıştırma başarılı; henüz süre/boyut koşuluna giren canlı arşiv yok. Canlı 7882 ve izleyici 7883 açık; dört eski servis kapalı kaldı.

## Testler ve alan

- Tam Python paketi yeniden çalıştırıldı: **177 geçti, 0 başarısız**. İlk denemede arşivlenmiş gerçek dosyaya bağlı bir test bulundu; sentetik ve bağımsız fixture ile düzeltildikten sonra paket tekrar geçti.
- Gerçek kapalı Zstandard arşivinin özel kopyasında 17 kayıt ve 15.876 olay doğrulandı. Özetin ilk/son durumları ve olay toplamları bağımsız SQLite sorgularıyla eşleşti; özgün canlı arşiv, aktif parça ve WAL değişmedi.
- Dört korunan SQLite checkpointi ve son durum hashleri yeniden doğrulandı.
- Systemd birim doğrulaması geçti; saklama servisinin son sonucu `success`, çıkış kodu 0.
- Gerçek arşiv geri açma ve dondurulmuş doğrulayıcı kontrolü geçti; seed indeksleri mevcut tarayıcıyla kontrol edildi.

Temizlenen eski ayrıntı ve kayıpsız sıkıştırma birlikte, küçük rapor/indeks dosyaları hariç yaklaşık **1,80 GiB** alan kazandırdı. Proje temizlik hemen sonrasında yaklaşık 4,2 GiB idi; canlı gözlem kayıt üretmeye devam ettiği için son ölçümde yaklaşık **4,5 GiB**. Bu fark temizliğin geri alınması değildir.

Makinece okunabilir sonuç ve kanıt yolları: [HISTORY_RETENTION_20261024.json](HISTORY_RETENTION_20261024.json). Yeni biyolojik deney başlatılmadı.

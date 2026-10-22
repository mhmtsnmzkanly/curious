# Av laboratuvarı v2 — Luna uygulaması, root denetimi

Kaynak `34973de30ce0187d84dc4a325d9b1b32a96143c8`; protokol sonuçlardan önce commit edildi.
İki iz, sekiz eğitim/değerlendirme çifti, dört kol: **6400 deneme, 56384 fiziksel tick**. Batch tamamlandı; sürekli deney çalışmıyor.

Kendi avının savunması artık dış saldırı sayılmaz. Görünür hedefte doğrulanmış enerji bütçesi kesintileri yalnız geçmiş gerçek eylem masraflarını puanlar. Hedef değişimi, üçüncü taraf hasarı, kayıp hedef ve laboratuvar sınırı puansız kalır. Ödül katsayıları/keşif/fizik değiştirilmedi.

| İz | Evre | Kol | Beslenen / 320 | Toplam enerji harcaması | Puansız / motor kapanışı |
|---|---|---|---:|---:|---:|
| passive | normal | rule_based | 0 | 11520 | 0 / 0 |
| passive | reversed | rule_based | 320 | 1600 | 0 / 0 |
| passive | normal | naive_online | 298 | 7892 | 1231 / 1551 |
| passive | reversed | naive_online | 320 | 2674 | 115 / 435 |
| passive | normal | experienced_online | 290 | 6700 | 913 / 1233 |
| passive | reversed | experienced_online | 314 | 2928 | 124 / 444 |
| passive | normal | experienced_frozen | 246 | 7134 | 794 / 1114 |
| passive | reversed | experienced_frozen | 302 | 6005 | 432 / 752 |
| defensive | normal | rule_based | 0 | 11520 | 0 / 0 |
| defensive | reversed | rule_based | 320 | 1600 | 0 / 0 |
| defensive | normal | naive_online | 283 | 7594 | 1150 / 1373 |
| defensive | reversed | naive_online | 317 | 2673 | 116 / 436 |
| defensive | normal | experienced_online | 313 | 5946 | 600 / 869 |
| defensive | reversed | experienced_online | 307 | 3628 | 194 / 503 |
| defensive | normal | experienced_frozen | 313 | 5950 | 584 / 854 |
| defensive | reversed | experienced_frozen | 262 | 6835 | 415 / 692 |

## Bulgular ve sınırlar

Her iki izde de eğitimden sonra boş kalan model sayısı 0/8 (V1: 4/8). Defensive ters evrede deneyimli online 307/320 beslenir; bu 307 Fed kapanışının tamamı puanlanır. Normal evrede aynı kolun 313 Fed kapanışından 44 tanesi hâlâ puansızdır; bunlarda başka hedefin verdiği hasar gibi belirsizlikler korunur. TargetChanged kapanışları puansız kaldığından bütün girişimlerden öğrenildiği iddia edilmez.

Deneyimli online normal passive 290/320, naive 298/320; defensive 313/320, naive 283/320. Ters koşulda deneyimli online passive 314/320 ve defensive 307/320; naive 320/320 ve 317/320. Eğitim her koşulda üstün değildir. Kurallı politika normal izlerde 0/320, ters izlerde 320/320: ortamın basit kurala uygunluğu sonucu belirleyebilir.

V1 değerlendirme seedleri 41001–41008, V2 42001–42008 olduğundan iki sürümün oran farkları aynı seedde nedensel karşılaştırma değildir. Yeni seedler artık görülmüştür. Scriptli avlar ve her trial beden/yaş sıfırlaması kullanılır; gerçek yaş, ekosistem, evrim veya genel zekâ kabulü değildir. Avın savunma hasarı mevcut ödülde ceza değildir; enerji başarısı güvenli davranış garantilemez.

## Denetim

123 Rust, 64 Python ve Node deneyim sunumu testleri geçti. HTTP kontrol/auth/writer-lock/SIGKILL ve av öğrenmesi kapalıyken kabul edilmiş biyolojiyle 100/200 tick State/RNG eşitliği geçti. 16 SQLite örneğinin integrity, son tick/enerji ve tüm motor/lab-boundary HuntEpisodeEnded olayları ham JSONL ile birebir karşılaştırıldı.

Ham veri, protokol, immutable probe, SHA manifest, summary ve 16 replay `data/experiments/hunt-lab-v2-20261022` altında korunur. Canlı 7882 motoru ve 7883 viewer güncellenmedi; av öğrenmesi canlıda kapalı. Sonraki çalışma dar kontrollü güvenlik/maliyet ölçümü ve ayrı karma ekosistem doğrulamasıdır; otomatik uzun tarama başlatılmadı.

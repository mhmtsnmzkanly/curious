# Av laboratuvarını çalıştırmak ve okumak

Sözleşme: HUNT_LAB_PROTOCOL.md. Bu sonlu bir kontrollü deneydir; yeni sürekli
simülasyon servisi değildir. Canlı dünya ve replay servisleri değişmez.

```bash
cargo build --release --example hunt_lab
python3 scripts/run_hunt_lab.py \
  --binary target/release/examples/hunt_lab \
  --output data/experiments/hunt-lab-v1-20261021
```

Çalıştırıcı temiz commit ister; mevcut çıktıyı ezmez. İkiliyi ve protokolü çıktı
klasörüne kopyalar, source/ikili/protokol SHA'larını manifestte tutar. En çok300s
çalışır. Başarısızlık raw/stderr/manifestte korunur; başarı diye sunulmaz.

Düzeltmenin ayrı, sonuç öncesi sabitlenmiş protokolü
`HUNT_LAB_PROTOCOL_V2.md` ve seedleri 42001–42008'dir. Yeni temiz bir çıktı
diziniyle çalıştırmak için:

```bash
python3 scripts/run_hunt_lab.py \
  --binary target/release/examples/hunt_lab \
  --output data/experiments/hunt-lab-v2-<new-run-id> \
  --v2
```

V1 varsayılanı ve eski validator yolu korunur. V2 hasar kaynağı toplamlarını
ayrıca doğrular; kaynağı belirlenemeyen hasar unattributed olarak tutulur.

V3, `HUNT_LAB_PROTOCOL_V3.md` içindeki varsayılan kapalı risk seçimi
hipotezini altı kolla sınar. Önce protokol ve kaynak temiz commit olarak
incelenmeli; sonra temiz `hunt_lab` binary'siyle yalnız yeni çıktı dizini
kullanılmalıdır:

```bash
python3 scripts/run_hunt_lab_v3.py \
  --binary target/release/examples/hunt_lab \
  --output data/experiments/hunt-lab-v3-<new-run-id>
```

Çalıştırıcı 43001–43008 eşleşmelerini, source/protocol/binary hash'lerini,
başarı veya hata zamanını ve tamamlanmış sonuçların hash'lerini manifest'e yazar.
Üst sınır 204800 fiziksel tick ve 300 saniyedir. `summary.json` seed başına ve
ilk/son normal/ters 20 trial pencerelerinde bütün altı kolu verir; `report.md`
yalnız validator'dan geçmiş çıktılardan üretilir. Modelin risk sayacı farkı,
resolver kapanışlarından bağımsız ölçülen episode maruziyeti/yaralanma oranının
yerine kullanılmaz. V3 çıktısı risk, besin, enerji, ölüm ve son sağlığı yan yana
gösterir; canlıya kabul ya da dağıtım kararı vermez.

- raw.jsonl: iki iz×sekiz çift; her satır eğitim+4kolun bütün trial/tick auditleri.
- summary.json: her seed/kol için normal/ters pencereler, gerçek enerji/besin,
  puanlanan/puanlanmayan bölüm sayıları ve sekizli ortalamalar.
- samples/: her izde ilk değerlendirme seedinin dört kol×iki koşulu,16kısaSQLite.
- hunt-lab-probe: ölçülen değişmez ikili; derleme temizliğinden etkilenmez.

`HuntEpisodeEnded` yeni gözlem olayı bütün bölüm kapanışlarını aktarır. Bir tickte
hedef değişimi ve ölüm gibi iki kapanış da ayrı kaydedilir; eski last_episode
alanını her tick tekrar saymak kullanılmaz. İki sonuçluk geçici kuyruk World.tick
sonunda boşaltılır; checkpoint/öğrenilmiş tabloya ek durum yazılmaz. Ödül, seçim,
RNG veya fizik değişmez; V1/V2 ölçümleri oluşturulurken checkpointV13/SQLiteV2
kullanılmıştı. Yeni geliştirme V14'e geçti. Ham laboratuvar sınırında
zorunlu kapanış origin=lab_boundary olur, motorun başarısızlığına karıştırılmaz.

Karşılaştırmada reward tek ölçüt değildir. food gerçek EntityAte miktarı;
hareket/saldırı gerçek resolver enerji bedelidir. energy_spent başlangıç+besin−son
beden rezervi, non_action_energy bunun hareket/saldırı dışındaki kısmıdır. Bu
enerji denklemi gözlem içindir; öğreniciye ödül olarak enjekte edilmez. failed_ticks
yalnız hiç besin alamayan trial'ların süresidir. Fed sonucu injury/belirsizlik
nedeniyle puansız olabilir; engine_ends,scored_ends,unscored_ends ayrı okunur.

Bir kayıt örneğini mevcut salt okunur oynatıcıyla incelemek için:

```bash
python3 scripts/archive_replay.py \
  --data-dir data/experiments/hunt-lab-v1-20261021/samples/passive/experienced_online/trial-040 \
  --port 7884
```

Adres http://127.0.0.1:7884/replay. Bu komut canlı7883görüntüleyicisini veya7882
dünyasını değiştirmez. Kayıt trial0beden resetinden başlar; gerçek yaşamla karıştırılmaz.

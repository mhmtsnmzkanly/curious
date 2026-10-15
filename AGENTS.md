# Curious üzerinde çalışma

İşe başlamadan önce `docs/VISION.md`, `docs/CONTROL.md` ve `TODO.md` dosyalarını
okuyun. Hedef ve kullanıcı tercihleri bu dosyalarda korunur. Konuşmada verilen
sonraki talimatlar bu notların önündedir.

## Hedefi koru

- Hedef yalnızca dengede kalan bir ekosistem veya Game of Life benzeri bir demo
  değildir. Basitleştirilmiş bir dünyada, belgesel izler gibi gözlemlenebilen,
  koşullara göre inandırıcı kararlar veren bireyler ve nesiller amaçlanıyor.
- Önce deterministik dünya ve kurallı kararlar; sonra bireysel hafıza/öğrenme;
  ardından ölçülebilir kalıtım ve evrim. Yaş tek başına tecrübe bonusu değildir.
- Dünya, beden/yetenekler, algı, karar, öğrenme ve kalıtım ayrı sorumluluklardır.
  Yeni tür, dünya kuralı veya yetenek eklemek geniş çaplı yeniden yazım gerektirmemeli.
- Uzun süre çalışabilme temel gereksinimdir. Bellek, kayıt boyutu ve tick maliyeti
  izlenmeli. Aylar süren çalışma veya açık uçlu evrim kanıtlanmadan vaat edilmez.
- Eksik özellikleri tasarım hatası saymayın. Davranış sorunlarını önce ölçün;
  yalnızca nüfusu hayatta tutmak için gizli yardımlar eklemeyin.

## Sunucudaki mevcut çalışma

- Repo: `/root/curious`; servis: `curious.service`.
- Önceki dünya: `http://127.0.0.1:7878`, `data/main`, pause durumunda.
- Sürekli çayır: `http://127.0.0.1:7879`, `data/meadow`, `curious-meadow.service`,
  0,1 tick/s. Bu hız disk tüketimi ve gözlem için bilinçli seçildi; gerekçesiz artırmayın.
- `curious-monitor.timer` dakikada bir salt okunur sağlık kontrolü ve Notify uyarısı yapar.
- Systemd birimi `deploy/curious.service` kaynaktan `/etc/systemd/system/` altına
  kurulmuştur. Servis yeniden açılırken checkpoint yükler ve pause durumuna geçer.
- Gerçek durumu `/status` üzerinden kontrol edin; bu dosyadaki eski tick bilgisini
  güncel kabul etmeyin. Çalışan dünyayı test verisiyle değiştirmeyin.
- Geçmiş shardları, canlı verilerini veya checkpointleri derleme temizliğiyle silmeyin.
- Aktif RNG dünya başına ayrıdır. Yeni dünyalar için açık seed kullanan
  `World::with_seed` tercih edilir; global seed yardımcıları eski çağrılar içindir.
- Güncel checkpoint V2; kod `simulation/checkpoint.rs` içinde. SQLite şeması V1.
  Eski kayıt yüklemek aynı geleceği değil, durum göçünü garanti eder.
- Son deney raporu `docs/reports/RENEWABLE_MEADOW.md`: otçul nesilleri doğrulandı;
  öğrenme/evrim ve avcı–av dengesi henüz tamamlanmadı. Kayıpsız gzip arşivi
  ölçümde 5,55 kat tasarruf sağladı. İşletim raporu `docs/reports/STORAGE_AND_OPERATIONS.md`.
- Kapanmış `.sqlite.gz` geçmişini silmeyin. API arşivleri geçici açıp sorgulayabilir.
  V2 yükseltme öncesi yedek `data/backups/pre-checkpoint-v2/` altında.

## İletişim

Kullanıcı önemli aşamalarda Notify üzerinden haber verilmesini açıkça istedi.
İlk temel aşama rapor sayfası: `p_32dc9b49b5101fc1`
(`https://notify.novastrum.dev/p/p_32dc9b49b5101fc1`), aşama durumu completed.
Bu, projenin tamamlandığı anlamına gelmez. Sonraki aşama için yeni görev sayfası açın.
Rutin her komut için bildirim göndermeyin. Tamamlanma, önemli bulgu, engel veya
müdahale gerektiren hata için gönderin. İhtiyaç varsa dinamik rapor sayfası,
grafik/dosya eki ve gerçek ürün tercihleri için form/yanıt özelliklerini kullanın.
Sırf özellik kullanmış olmak için panel veya onay akışı oluşturmayın.

- Sistem: `/root/notify` → `/var/www/notify.novastrum.dev`.
- Önce güncel `docs/AGENT_INTEGRATION.md`, `docs/OPERATIONS.md` ve gerekli CLI
  belgelerini bu sistemin kendi klasöründen okuyun.
- CLI: `node /root/notify/bin/notify.js -t /root/notify/token.json send <message.json> --json`.
- Bu konuşmada kullanılan kullanıcı: `memo`, `u_e049630155fc`. Yalnızca ilgili
  kullanıcıyı hedefleyin; toplu bildirim kullanmayın.
- Token içeriğini çıktıya, Git'e, raporlara veya bildirimlere koymayın.
- Ağ tekrarlarında aynı mesaj için sabit idempotency anahtarı kullanın.

## Doğrulama ve disk bakımı

- İlgili değişikliklerde `cargo test`; servis/kontrol değişikliklerinde ayrıca
  `python3 scripts/check_control.py target/release/curious` kullanın.
- Testler ayrı geçici dünyada çalışmalı; aktif `data/main` test alanı değildir.
- Derleme öncesi/sonrası gerektiğinde `du -sh target/debug target/release` ile
  boyutları inceleyin. Kullanıcı Rust çıktılarının düzenli temizlenmesini istiyor.
- Debug/test çıktıları gerekmediğinde `cargo clean --profile dev` ile temizlenebilir.
- Çalışan systemd biriminin kullandığı `target/release/curious` dosyasını koruyun;
  tam `cargo clean`, sonraki servis başlangıcını bozabilir. Tam temizlikten önce
  servis için ayrı bir binary kurulmalı veya yeniden derleme planlanmalı.
- Cargo registry, başka projeler, SQLite geçmişi ve loglar derleme önbelleği değildir.
- Tamamlanan anlamlı değişiklikleri kullanıcının istediği gibi commit edin.
  Push veya ek dış yayın istenmedikçe ayrıca yapılmaz.

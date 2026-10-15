# 11 — İzleyici gözetimi ve kontrollü dağıtım

Dünya izleyicisi ayrı süreçte kalır. Yeni dünya kendi durum dosyası, label ve
idempotency alanını kullanır; aynı saniyedeki iki dünya uyarısı birbirine çarpmaz.
Notify kabul edilmeyen mesaj kalıcı outbox'ta kalır. `monitor_watchdog.py`, iki
izleyicinin 3 dakikadır, iki kaynak örnekleyicinin 35 dakikadır güncellenmemesini
ve 15 dakikadır boşalmayan bildirim kuyruğunu ayrı timer üzerinden fark eder.
Tek olaya her dakika tekrar bildirim göndermez; düzelme de kaydedilir. Bu gözetim
bütün sunucunun veya Notify hizmetinin aynı anda arızasını dışarıdan tespit eden
bir altyapı değildir; host dışı izleme ileride gerekir.

Sekiz Python testi, gecikme/tekrar/düzelme, bağımsız dünya anahtarları ve mevcut
uyarı davranışlarını doğrular. Systemd tekrar başlatma ve süre sınırları vardır.

`scripts/deploy_lab.py` yalnız duraklatılmış laboratuvarı yükseltir: değişmez
ikili kopyası, kapalı veri dizini yedeği, SQLite integrity_check, atomik symlink,
yeniden başlatma ve eski durum alanlarının/RNG/tick'in korunduğunu doğrulama.
Başarısız yükseltmede yeni veriyi de saklayarak eski veri+ikiliye döner. Eski
motoru yeni checkpoint üstüne körlemesine açmaz. Commit temiz olmalı.

Yeni dünya `curious-evolution.service`: 7881, `data/evolution`, 0,1 tick/s,
öğrenme/kalıtım açık otçul çayırı. Karma denge sağlandı gibi gösterilmez. Eski
7879 çayırı kesintisiz eski sürümde kalır. 7878 eski dünya pause'dur. 7880
laboratuvar yeni sürümde pause'dur. Gözlem ekranı yeni sürümlerin `/` yolundadır.
Tüm veri/komut uçları eski token sözleşmesine ek olarak tarayıcının yabancı
Origin isteklerini reddeder; kendi origin arayüzü çalışır.

Dağıtımın gerçek revision, hash, yedek ve tick sonuçları `data/lab/deployment.json`
içindedir; notlar AGENTS.md'ye kaydedilir. Gerçek süreli gün/hafta deneyi (9)
hâlâ devam eder. Kalıcı karma ekosistem, öğrenmenin farklı görevlere yayılması,
açık uçlu evrim ve daha büyük nüfuslar bu aşamayla tamamlanmış sayılmaz.

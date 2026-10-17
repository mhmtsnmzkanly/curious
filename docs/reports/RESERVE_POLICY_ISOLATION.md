# Reddedilen rezerv politikasını ayırma

## Düzeltilen iki sorun

Dad97e9'ın iki kolu bitti: şoksuz 7/8, şoklu 6/8. Şoksuz seed 3'te otçullar
5016. adımda tükenirken eski V9 kontrol aynı seedde 20.000 adımı geçmişti.
Bu nedenle adayın ortak, varsayılan karar kuralında kalması kabul edilmedi.

V11 normal açlık kodunu ae7f0df ile birebir aynı hale getirir. Üreme enerji
eşiğini hesaplayan ortak LifeState metodu korunur. Deneysel bir davranış artık
varsayılan kod yolunu değiştirmez ve her dünya açık/kapalı ayarını kaydeder.
Açma seçeneği deney manifestinde olduğundan açık ve kapalı sonuçlar aynı
koşunun devamı gibi birleştirilemez.

İkinci sorun, bitişikteki eşi gerekçe gösterip uzaktaki besine yürüme, sonraki
adımda eşi bitişikte göremeyince amacı terk etmeydi. Yeni deneysel davranış
bu çok adımlı işi başlatmaz: sadece hareket gerektirmeyen bir öğünü, tek öğünde
üreme eşiğine ve sonraki bazal adıma yetecek miktarda ise seçer. Yetersiz/uzak
besin veya hazır eşin yokluğunda eski karar korunur. Dolayısıyla örtük olarak
başlatılıp unutulan yolculuk yoktur. Çok adımlı genel hedef takibi tamamlandı
denmez; bu aday tek adımlık bir fırsattır.

## Mimari ve doğrulama

`policy.rs` dünya ayarını ve saf fırsat seçimini taşır. Yalnız AI'nin olağan
Rest/Explore/NoEligibleMate kararını değiştirebilir. Tehdit, susuzluk, normal
açlık ve özel/dış politika korunur. IntentCollection açık ayarı uygular;
öğrenme, hafıza ve fiziksel çözümleme ayrı kalır. Özel gerekçe
PrepareReproduction gözlemde ve günlükte görünür. Canlı dünyanın ayarı bu
çalışmada değiştirilmez; yeni senaryolar false başlar.

Testler, varsayılanın besini tüketmediğini, açık ayarın mevcut besinden
enerji toplayıp sonraki adımda gerçek doğumla sonuçlanabildiğini doğrular.
Uzak/küçük besinle amaç başlatılmaz. Ayar checkpointten aynı durum/RNG ile
sürer; eski kayıt eksik alanda false başlar. Deney ayarı değişince resume
reddedilir. Tüm hedeflerde 58 Rust ve 24 Python testi geçti; release ve
HTTP/auth/yazıcı kilidi/SIGKILL kurtarma kontrolü geçti. Arayüz JS sözdizimi
kontrol edildi; görsel tarayıcı testi bu aşamada tekrar çalıştırılmadı.

## Sınırlı tekrar

Önceden çöken seed 3, aynı senaryoda 6000 adım için iki kolda yeniden oynatılır:
varsayılan ve açık deneysel fırsat. Bu, bilinen 5016. adım gerilemesini kapsar.
Kayıtlar `data/experiments/policy-isolation-replay/{default,experimental}`.
İlk geliştirme tekrarı dirty etiketi taşır; bu uzun ekolojik kabul değildir.
Varsayılanın nüfus/enerji/sığınak örnekleri eski V9 kontrolün aynı adımlarıyla
karşılaştırılır; bu tam beden+RNG eşitliği iddiası değildir.

Şok dayanıklılığı hâlâ açıktır. Yeni aday kısa tekrarı geçse de yeni bir
şok/şoksuz karşılaştırma ve sonrasında yeni, görülmemiş seedler gerektirir.
Ayarın false kalması eski kaynak şoku zayıflığını çözmez; başarısız değişikliğin
sessizce varsayılan olmasını engeller. Mevcut canlı dünyalara dağıtım yapılmaz.

## Otomatik, sınırlı sonraki kontrol

`check_reserve_policy.py` açık aday için önce sekiz eğitim seedi ×20.000 adım
şoksuz kontrol yürütür. Ancak 8/8 geçerse aynı sekiz seed ×20.000 adım kaynak
şokuna geçer. Normal kontrol geçmezse durur; şoku başlatmaz, parametre değiştirmez
ve dağıtım yapmaz. Üç protokol testi ret/eksik sonucun sonraki kapıyı açmadığını
ve yalnız normal başarının şoku başlattığını doğrular.

Yeni çalışmanın kaynak kimliği `data/experiments/policy-isolation-validation/study.json`,
anlık aşaması `state.json`, ayrıntıları normal/shock alt dizinlerindedir.
Servis `curious-policy-validation.service`; ayarlar ve ikili sabit kaynakla
çalışır. Bu yollar başlatılmadan önce çalışan deney kanıtı değildir.
Sonuçlar Notify ile bildirilir. Bu çalıştırıcı kod yazmaz ve otomatik iyileştirme
yapmaz; önceden tanımlı iki aşamayı yürütür.

## Tekrar sonucu

Seed 3 her iki kolda da 6000 adımı geçti. Varsayılan sonunda 41 avcı/559 otçul,
açık adayda 52 avcı/513 otçul var. Varsayılanın 61 nüfus/enerji/sığınak örneği
eski V9 kontrolle birebir eşleşti. Bilinen 5016. adım çöküşü varsayılanda
giderildi; bu kaynak şoku veya açık adayın uzun kabulü değildir. Makinece
okunur karşılaştırma `POLICY_REPLAY_RESULT.json` içindedir.

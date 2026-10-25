# Dokümantasyon standardı

Bu, Curious deposunun yeni ve düzenlenen belgeleri için proje standardıdır;
resmî bir ISO veya genel yayın standardı iddiası değildir. Amaç tek bir doğru
giriş noktası, izlenebilir kanıt ve mevcut olmayan sonuçları varmış gibi
göstermeyen kısa belgeler tutmaktır. Dondurulmuş geçmişi geriye dönük bu
kurallara uydurmak gerekmez.

## Belgelerin tekil rolleri

- [README.md](../README.md) projenin kısa tanıtımı, güncel sistem ayrımı ve
  yerel başlangıç noktasıdır.
- [docs/README.md](README.md) belge dizini ve belge/artifakt politikasıdır.
- [ROADMAP.md](ROADMAP.md) K1–K8 kategorilerinin mevcut durumu, açık boşlukları,
  önerileri ve kabul ölçütleri; P0–P3 ise ayrı yürütme sırasıdır.
- [TODO.md](../TODO.md) kısa, eyleme dönük sıradaki işlerdir. Geçmiş günlük veya
  rapor kopyası tutmaz.
- [AGENTS.md](../AGENTS.md) çalışma alanındaki ajanlar için işletim, review,
  onay ve güvenlik kurallarıdır.
- Alan sözleşmeleri (ör. [CONTROL](CONTROL.md), [LEARNING_NEXT](LEARNING_NEXT.md))
  arayüz ve davranış kontratlarının sahibidir. Aynı kontratı README, roadmap
  ve raporlarda yeniden tanımlamayın; oraya bağlantı verin.
- Protokoller ölçüm tasarımını; `docs/reports/` tarihli deney/işletim kanıtını;
  rapor JSON'ları ise yapılandırılmış, isteğe bağlı makinece okunabilir kanıtı
  taşır. JSON her belge için zorunlu değildir.

Güncel bilgi için bu rollerden birini tek kaynak seçin. Tarihçe gerekiyorsa
bağlantılı kanıtı belirtin; aynı tick, log, deney günlüğü veya olay dizisini
birden çok giriş belgesine kopyalamayın.

## İddia ve durum dili

Durum etiketlerini birbirinin yerine kullanmayın:

- **Önerildi:** fikir veya sıradaki tasarım; uygulanmış değildir.
- **Uygulandı:** kaynakta vardır; tek başına test veya fayda kanıtı değildir.
- **Mekanizma testi geçti:** belirtilen kod yolu/test sınırında çalıştı; biyolojik
  veya ekolojik yarar kabulü değildir.
- **Ampirik olarak kabul edildi:** yalnız belirtilen protokol, seed/rol, fixture,
  süre ve metrik kapsamı için bağımsız kabul ölçütü geçti.
- **Dağıtıldı:** belirtilen binary/source, checkpoint/schema ve canlı ortamda
  doğrulandı. Kaynak dalındaki özelliklerin tümünün canlıda olduğu anlamına
  gelmez.

Her ampirik sonuçta test edilen nüfusu, fixture'ı, karşılaştırmayı, süreyi,
ölçütü ve önemli sınırlamayı belirtin. “Kabul edildi” ifadesini yalnız bu kapsam
adlandırıldığında kullanın; fixture sonucu bütün habitatlara genellenmez.

`Unknown` değerini ölçülmüş sıfır gibi yazmayın. Hasar, risk ve başarı için hedef
ufkunu belirtin. Tamamlanmış ilk resolver aralığı sonraki tam-rota sansüründen
ayrı hedef olabilir; hedef ufku içinde kesinti, eksik atıf veya gözlenmeyen yol
ise `Unknown` kalır. Gözlenen etiketle tahmin edilen/counterfactual sonucu ayrı
tutun. Veri aralık ve belirsizlik taşıyorsa bunları typed alanlarla koruyun;
keyfî nokta tahmini ya da belirsizliğin sıfıra iması yapmayın.

Canlı çalışma notlarında zamanı, portu, sürümü ve checkpoint/source kimliğini
yalnızca gerçekten ölçülmüş runtime bilgisi olarak verin. Dosya tarihinden veya
güncel tarihten canlı tick türetmeyin; her gün sahte “son güncelleme” tarihi
eklemeyin. Geliştirme kaynağı, canlı sistem ve ayrı çalışan replay ikilisinin
kimliklerini ayırın; sürümlerinin aynı olduğunu varsaymayın.

## Protokol, rapor ve eski belge bakımı

- Dondurulmuş protokol, seed manifesti, source/hash ve rapor sonuçlarını
  sessizce değiştirmeyin. Hata veya yeni yorum varsa eski artefaktı koruyup yeni
  tarihli düzeltme/ek rapor oluşturun ve önceki kanıta bağlantı verin.
- JSON eşlikçisi yararlıysa aynı iddia, sayım, kapsam ve hash'leri Markdown ile
  tutarlı taşısın; JSON yeni deney/kanıt üretmez.
- Tamamlanmış veya eskimiş plan özetini kaldırmadan önce açık işleri ROADMAP/TODO
  veya alan sahibi belgeye taşıyın, inbound bağlantıları güncelleyin ve eski
  metnin Git geçmişinden geri alınabildiğini doğrulayın. Kaldırma, adı geçen her
  özelliğin tamamlandığını kanıtlamaz.
- `data/` ve `bin/` yerel/ignore edilen deney ve ikili artifaktlarıdır; GitHub'a
  yüklenmiş gibi bağlantı vermeyin. Ham kanıtın lossless arşivi varsa raporun
  arşiv indeksini belirtin. Ham dosya gereken analizden önce belgelenmiş restore
  aracını kullanın; doküman düzenlemesi için artefaktı geri yüklemek gerekmez.

## Dil, Markdown ve bağlantılar

- Bakımı yapılan proje açıklamalarında Türkçe, doğal ve kısa anlatım kullanın;
  API, type, status label, komut, path ve sabit terimler gerektiğinde İngilizce
  kalabilir. Sağlayıcıdan bağımsız skill yönergeleri mevcut katalog düzeninde
  İngilizcedir; kullanıcıya yanıt dili Türkçedir. Tarihsel İngilizce raporları
  dil birliği için yeniden yazmayın.
- Markdown göreli bağlantılarını ve K1–K8 için [ROADMAP](ROADMAP.md)'teki kararlı
  anchor'ları kullanın. Taşınan başlığın anchor'ını veya bağlantılarını birlikte
  güncelleyin. Birincil sahibini ve çapraz kategori bağımlılıklarını açık tutun;
  aynı öneri için çatışan ikinci backlog oluşturmayın.
- Komutları kaynak, `--help` veya uygun test sözleşmesiyle doğrulayın; README
  örneğini canlı servise karşı denemeyin. Dokümanda token, parola, özel anahtar,
  kullanıcı verisi veya gizli environment değeri yer almaz.

## Düzenleme akışı ve gereken bağlam

1. Değişikliğin belge sahibini ve okuyucusunu belirleyin; güncel kaynağı,
   raporu/protokolü veya gerçek runtime kaydını gerektiği kadar okuyun.
2. Canonical belgeyi güncelleyin; özeti değiştirdiyseniz yalnız gerekli giriş
   noktalarını ve cross-reference'ları güncelleyin. Tarihsel/frozen kanıtı
   düzenlemeyin.
3. Yerel Markdown bağlantılarını ve kategori anchor'larını kontrol edin; diff'i
   `git diff --check` ile inceleyin. Belge-only iş için Rust build veya World/lab
   çalışması gerekmez.
4. Commit/push mevcut kullanıcı yetkisinin kapsamına göre yapılır; zaten
   verilmiş yetki gereksiz yere yeniden istenmez.

Şablonları belge türüne göre uyarlayın; her belgeye zorunlu YAML/frontmatter
veya rapor şeması eklemeyin. Güncel durum belgesi genellikle **tarih ve kapsam**,
**ölçülmüş mevcut durum**, **boşluk/Unknown**, **önerilen sonraki iş ve kabul
kapısı**, **kanıt bağlantıları** alanlarını ister. Protokol **source/seed/fixture,
karşılaştırma, hedef ufku, metrik, süre/kaynak bütçesi, durma/sansür ve hash**
bilgilerini; rapor ise **ne çalıştı,
sonuç, sınırlama, kaynak/protokol ve artifact kimliklerini** taşır. Salt dizin,
TODO veya arayüz belgesi bu alanların tümünü tekrar etmez.

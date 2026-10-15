# Seed 2: karar kanıtı ve demografik kopuş

Önceki otçul çayırının seed 2 koşusu, 270. tickte tükeniyordu. Karar gözlemi
eklenip aynı başlangıç tekrar çalıştırıldı: 300. tickteki dünya ve RNG önceki
kayıtla birebir aynı kaldı. Gözlem katmanı davranışı değiştirmedi.

## Son birey

Son yavru (id 20), 164. tickte id 18 ve 19'dan doğdu. Ebeveynler 181 ve 183'te
yaşlılıktan öldü. Yavru 184'te olgunlaştı; üreyebilir olduğu 86 tick boyunca
yaşayan bir eş kalmadı. 270'te yaşlılıktan öldü. Bu koşudaki 20 ölümün tamamı
yaşlılık: doğrudan son neden açlık veya susuzluk değil, nesillerin zamanlaması.

## Önceki fırsatlar

Gerçek karar anında, bireyin minimum üreme koşullarını sağladığı 328 birey-tick var:

- 203'ünde görülebilir uygun eş vardı.
- 96'sında uygun eş komşuydu.
- Bu 96 kararın 68'i yiyecek, 12'si su aramayı; 16'sı çiftleşmeyi seçti.

Bunlar **birey-tick** sayılarıdır; bağımsız çift veya kaçırılmış 80 doğum anlamına
gelmez. Sonraki kararlar birbirini etkiler. Bulgular, rezervleri %75'e tamamlama
tercihinin %50'de izin verilen üremeyi sık sık ertelediğini gösteriyor.

Bu bir seçilim/öğrenme sonucu değil, mevcut sabit öncelik politikasının davranışı.
Bir sonraki karşılaştırma: tehdit yokken ve iki birey minimum güvenli koşulları
sağlıyorken komşu eş fırsatını kullanmak; aynı seed matrisiyle sonucu ölçmek.

## Yeni inceleme araçları

- `GET /decisions`: son tickteki gerçek karar kanıtı ve sonucu.
- `POST /control` → `{"action":"observation","full_all":false,"full_for":[20]}`:
  seçilen bireyin tam algısını da kaydet. Varsayılan bütün bireylerde kompakt özet.
- SQLite `decision_traces`: beden/algı özeti, karar dalının gerekçesi, hedef,
  karar öncesi RNG ve gerçek sonuç aynı tick transactionında tutulur.
- `python3 scripts/diagnose_decisions.py <run-directory>`: normal veya sıkıştırılmış
  shardlardan fırsat ve gerekçe dağılımını çıkarır. Salt okunur çalışır.

Eski kayıtlara sonradan gerekçe uydurulmaz. V1 shardlarda karar kanıtı bulunmaz;
aynı eski davranışın tekrar üretilebildiği koşu kullanılarak kanıt toplandı.

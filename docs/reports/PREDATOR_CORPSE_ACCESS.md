# Avcının erişilebilir leşi algılaması

## Hata ve düzeltme

f2ea544 geç kopyasında 11.804 avcı kararı incelendi. 73 kararda mevcut,
yanındaki leş algıda yoktu; 72 karar sığınak leşi içeriyordu. 43 karar açlık
hedefinin altındaydı. Bunlar 42 farklı leş; tüketim garantisi değildir.
Gözlemci politikaya bilgi vermez; yeni ölümler ve süresi dolmak üzere olan
leşler örneklem dışında. Gözlemli eski tekrar biyolojik durum ve RNG açısından
eski sonuçla eşit (yalnız gözlem seçeneği normalize edildi).

Algı yürünebilir hücreleri besine erişimle eşitliyordu. Oysa mevcut Eat
çözücüsü bitişik hücreden leş yiyebilir. Algı artık yerel görüş sınırında,
ulaşılabilir ve boş bitişik yaklaşım üzerinden son etkileşim adımını kurar.
Eat son adımı yürümez. Canlı avın sığınak koruması, duvarlar, görüş yarıçapı,
enerji/maliyet ve fizik kuralları korunur. Küresel bilgi veya nüfus takviyesi yok.

## Kontrol noktası sonuçları

| Tekrar | Önceki kaynak | Düzeltme |
|---|---|---|
| 17.400 → 20.000 | 646 otçul / 0 avcı | 628 otçul / 5 avcı |
| Yeni 20.000 → 24.000 devamı | — | 606 otçul / 28 avcı |
| 16.000 → 20.000 | 597 otçul / 45 avcı | 576 otçul / 69 avcı |

Devam kolunda 53 avcı doğumu, 30 yaşlılık ölümü var; avcı açlık/susuzluk
ölümü yok. Bu sadece ilk bireylerin hayatta kalması değildir. Geç ilk kolda
9 avcı doğumu vardır. Tek tohumdaki toparlanma kalıcı denge kanıtı değildir.
Ham veriler `data/experiments/predator-{late-audit,corpse-access}` altında;
SHA ve tam son metrikler eşlik eden JSON'da. Önceki deneyler korunur.

## Doğrulama ve sonraki kapı

81 Rust, 35 Python testi; Node arşiv kontrolü ve HTTP/auth/pause/speed,
yazıcı kilidi ve SIGKILL kurtarma kontrolü geçti. Beş yeni Rust testi gerçek
Eat sonucu, zorlanan mevcut fizik ile durum+RNG eşitliği, canlı sığınak
koruması, görüş ve yol engellerini kapsar. İki Python testi audit analizini sınar.

Yeni temiz kaynak ve değişmez probe ile `check_reserve_policy.py` kullanılır:
normal 8 tohum × 20.000 adım; yalnız 8/8 geçerse kaynak şoku 8 tohum.
Rezerv ve buluşma açık; sabit genetik, öğrenme kapalı, iki işçi. Biyolojik
ret sonraki aşamayı durdurur. Başlatılmış çalışmanın güncel durumu
`data/experiments/corpse-access-validation/state.json`, kaynak/binary kimliği
`study.json`, servis `curious-corpse-access-validation` içindedir.
Sonuç bildirimlerini mevcut doğrulayıcı yollar. Bu çalışma otomatik kod
üretmez, dağıtım yapmaz; 30×100.000 uzun kabulün yerine geçmez.

Checkpoint V12, SQLite V2 değişmedi. Rezerv/buluşma varsayılanları kapalı.
Algı düzeltmesi temel kaynak davranışıdır; çalışan eski dünyalar yükseltilmedi.

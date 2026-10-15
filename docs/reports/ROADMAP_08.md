# 8 — Gözlem ekranı

Motor HTTP servisinin `/` adresinde yerel, bağımsız gözlem arayüzü. Harita,
türleri ayıran şekiller, canlı/kaynak miktarı, seçili birey, yaşam rezervleri,
yavru/yetişkin bilgisi, ebeveynler, nesil, gerçek karar gerekçesi, hafıza,
öğrenilmiş su tercihleri ve kalıcı karar geçmişi görülebilir. Takip, duraklatma,
hız ve adım kontrolleri vardır. Boşalan yenilenme konumu kesikli çizilir.

`GET /observation` tek tick sınırında durum/karar/servis özetini birlikte verir;
ayrı HTTP yanıtlarını yanlışlıkla farklı ticklerden birleştirmez. Ekran bütün
kararları her hızda canlı okuyabildiğini iddia etmez; kaçırılan adımlar SQLite'tan
istenebilir. Sahte olay, yumuşatılarak uydurulmuş hareket veya canlı ekleme yoktur.

Statik HTML/CSS/JS anahtar girmeye izin vermek için açıktır; bütün veri ve kontrol
uçları token varsa korunur. Anahtar yalnız sekme belleğinde tutulur. Kullanıcı
verileri textContent ile çizilir. Dış CDN/font/model yoktur. Tarayıcı sürümü için
ayrı Node bağımlılığı gerekmez; motor ikilisine kaynak olarak gömülür.

Playwright Chromium ile masaüstü ve 390px telefon kontrol edildi: auth, birey
seçimi, kalıcı geçmiş, tek adım, hız, başlat/duraklat ve yatay taşma. JavaScript
sayfa hatası yok. Ekran görüntüleri `data/ui-validation/` altında; test
`scripts/check_ui.cjs` (Playwright test aracı dışarıdan sağlanır).

Açık kapsam: dünya henüz gerçekçi çizimli canlılar/animasyonlar içermez. Bu,
canlıların kararlarını inceleyen ilk kullanılabilir gözlem ekranıdır. Laboratuvara
dağıtım 11. aşamadaki yedekli süreçle yapılacak.

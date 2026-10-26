# Ödenemeyen su adımından önce bitişik öğün

**Kapsam:** K2 karar/beden sınırı; K3 su denemesi atfı ve K8 checkpoint bağımlılığı. Geliştirme checkpoint'i V21 / SQLite V2'dir. `policy.water_meal_priority_enabled` varsayılan kapalıdır. Bu bir öğrenici veya fizik değişikliği değildir; canlı V12'ye dağıtılmadı.

## Dar karar koşulu

Normal politika ve su öğrenicisi önce seçimini yapar. Aday yalnız AI'nin `SeekWater`, `LearnedWater` veya `RecallWater` nedenli gerçek `Drink` kararını inceleyebilir. Dış komut, oyuncu, özel politika ve tehdit/kaçış kararlarını değiştirmez.

Seçilen su yolu güncel algıdaki pozitif miktarlı suyla eşleşmelidir. `approach_steps = max(path.len - 1, 0)`; son adım etkileşim içindir, hareket değildir. Şu koşulların tamamı gerekir:

- Su için en az bir hareket adımı gerekir. Bitişik veya aynı hücredeki ücretsiz Drink korunur.
- Hareket hakkı ve dayanıklılık ilk adıma yeterlidir; yalnız enerji ilk adımı ödeyemiyordur: `move_energy_cost > 0` ve `energy < move_energy_cost`. Karar enerjisi bu tick'in metabolizmasından sonradır, BMR tekrar çıkarılmaz.
- Algıda bitişik/aynı hücrede, pozitif miktarlı, bitki yiyen tür için uygun bir bitki öğünü vardır. `is_corpse=false` ve `corpse_id=None`; alınabilecek gerçek miktar `min(amount, max_meal, max_energy-energy) > 0` olmalıdır. Yemek için hareket gerekmez.

Birden fazla uygun öğünde mevcut foraging skoruyla, ardından daha kısa yolla deterministik seçim yapılır. Son karar `EatBeforeWater` nedenli `Eat` olur. Yeni RNG çekilmez. Hiçbir koşul sağlanmıyorsa normal karar aynen kalır. Uzun su yolunda bir adım bile ödenebiliyorsa ilerleme korunur; bütün rotayı tek tick'te ödeyememek tek başına müdahale nedeni değildir.

## Fizik ve öğrenme atfı

Karar değişmeden önce açık su denemesi `WaterEnd::Interrupted` ile kesilir. Kesinti su kaynağının başarısızlığı veya ödüllü sıfır maliyet değildir. Aynı tick'in yemek kazanımı/hareketi su sonucuna yazılmaz; resolver gerçek yemeği ve maliyeti normal kurallarla uygular. Normal karar öncesi TTL temizliği hâlâ çalışabilir. Eski su seçiminde kullanılan RNG geri sarılmaz.

Karar izindeki opsiyonel `water_meal_override`, değiştirilen orijinal Drink kararını ve o andaki algılanmış su/bitki miktarlarını taşır. Gerçek `decision` ise Eat'tir; beden de normal `before` alanındadır. Alan boşken serileştirilmez, eski kayıtlarda yoksa `None` okunur. Bu geçici gözlem kanıtıdır; yavruya veya öğrenilmiş belleğe kopyalanmaz.

Mevcut su öğrenicisi yalnız tek `last_episode` tutar; bir tick içinde eski kesinti üzerine yeni kesinti yazılabilir. Bu alan tam olay geçmişi veya benzersiz kapanış sayacı değildir. Lab yalnız gözlenen geçişleri, açık bölümün ufuk sonunda sansürlü olduğunu ve eksik atfı Unknown olarak raporlar; sentetik su olayı üretmez.

## Sürüm ve sınırlar

V21 açık bayrağı korur; V1–V20 migration bu yeni bayrağı kapatır. V20'den gelen önceki `food_search_reserve_enabled` kendi mevcut kuralıyla korunur. V22 ve bilinmeyen ileri checkpoint'ler reddedilir. SQLite tabloları değişmez. V21 checkpoint eski V20/V12 ikiliyle açılmaz. Yeni bayrak için HTTP kontrol komutu eklenmedi; senaryoda veya izole lab kurucusunda açıkça seçilir.

Enerji üretilmez; kaynak yenilenmesi, dönüş rotası, suya ileride varış ve hayatta kalma garantisi yoktur. Kural yalnız anlık olarak ödenemeyen su hareketi ile mevcut ücretsiz bitki öğününü karşılaştırır. Nüfus, başka habitatlar, başka türler ve uzun vadeli yarar ayrı ölçülür.

[Sabit dört kollu protokol](WATER_MEAL_PRIORITY_V1_PROTOCOL_20261026.md), önceki yem arama bütçesini yeni kuraldan ayırır. Kod/mekanizma kontrolü, kayıtlı lab sonucu ve canlı kabulü farklı aşamalardır.

# Yem aramasında enerji bütçesi

**Kapsam:** K2 karar/beden sınırı, K5 beslenme ve K8 checkpoint bağımlılığı. Geliştirme checkpoint'i V21, SQLite şeması V2'dir. Canlı V12'ye dağıtım değildir. `policy.food_search_reserve_enabled` varsayılan kapalıdır; yeni bir senaryoda açıkça seçilebilir. Bu çalışma bir öğrenici veya öğrenilmiş bellek eklemez.

## Davranış sözleşmesi

Normal karar ve RNG seçimi önce çalışır. Yalnız AI tarafından seçilmiş `Reason::SearchFood` + `Intent::Move` yolu kısaltılabilir. Dış komut, oyuncu ve diğer nedenler aynen korunur: görünür yeme yönelme, su arama/içme, kaçış, savunma, sığınak, çiftleşme ve keşif bu seçeneğin hedefi değildir.

Karar anındaki enerji `E` zaten bu tick'in bazal metabolizma, ifade edilmiş beden maliyetleri ve varsa mevsimsel giderlerinden sonradır. `b = BMR_ENERGY_COST + vision_metabolic_cost + stamina_metabolic_cost`, bir sonraki tick için bilinen beden gideridir. `c = move_energy_cost` ve seçilmiş yolun uzunluğu `p` ise:

```text
energy_cap = floor(max(E - b, 0) / (2*c))
cap = min(p, effective_speed(injury_capacity_penalty) - moves_used, energy_cap)
```

Çıkarma doygunluklu yapılır. İki hareket payı mevcut hareket ve aynı sayıda adımlık olası dönüş hareketi bütçesidir. Yol zaten sınır içindeyse karar/neden değişmez. Kısaltılırsa `FoodSearchReserve` nedeni verilir; `cap=0` için bir tick `Idle` seçilir. Yeni RNG çekimi yoktur. Hareketi, dayanıklılığı, engeli ve ödemeyi normal resolver yeniden doğrular; enerji oluşturulmaz.

Bu, gerçek dönüş planı değildir. Başlangıç hücresinde yem/güvenlik, yemin yenilenmesi, dönüş yolunun açık kalması veya ileride geri dönülmesi varsayılmaz. Gelecek mevsim gideri ve koşula bağlı iyileşme `b` içinde değildir. Düşük enerjide `Idle`, besin olmayan yerde ölümü hızlandırabilir; üreme ve arama kayıpları ayrı ölçülmelidir. Hareketsizlik veya daha geç ölüm tek başına çözülmüş beslenme sorunu değildir.

## Checkpoint ve kapalı yol

V20 `PolicySettings` içinde bayrağı korur. V1–V19 göçü bayrağı kapalıya zorlar; V22 ve bilinmeyen ileri sürümler reddedilir. Bayrak kapalıyken JSON anahtarı atlanır ve karar/RNG yoluna ek iş girmez. V19'dan V20'ye capture sürüm metaverisi değişir; eski checkpoint ile byte eşitliği iddia edilmez. State ve RNG devamı mevcut V21 kaynağı içinde doğrulanır. SQLite tabloları değişmez.

V20 checkpoint eski V19/V12 ikiliyle açılmamalıdır. Bu bayrak için HTTP canlı kontrol komutu eklenmedi; açık konfigürasyon senaryo veya izole deney kurucusundadır. Mevcut `examples/operations_readiness.rs` yeni kaynakta V21 kontrol eder; [26 Ekim V19 koşusu](reports/OPERATIONS_READINESS_20261026.md) kendi sabit kaynak/ikilisinin tarihsel sonucudur, V20 runtime kanıtı değildir.

## Doğrulama ve kanıt sınırı

[Sabit lab protokolü](FOOD_SEARCH_RESERVE_GUARD_V1_PROTOCOL_20261026.md) üç fixture'da 6 seed ve iki kolu, 300 tick ufkunu tanımlar. Mekanizma testi, açıklanabilir koşulda daha kısa aramayı izleyen gerçek ücretli hareket ve öğünü sınar. Genel ekolojik veya canlı fayda kabulü bundan ayrıdır. Güncel sonuç tarihli rapora ve [TODO](../TODO.md)'ya kaydedilir; bu sözleşme tek başına ampirik fayda iddiası taşımaz.

## 26 Ekim sınırlı lab sonucu

[36 koşuluk rapor](reports/FOOD_SEARCH_RESERVE_V1_20261026.md): tek yemli fixture'da toplam sağkalım 624→812 birey-tick ve yem kazanımı 566→794 oldu; altı seedin dördü iyileşti, ikisi nötr kaldı. Hareket enerjisi 594→592; fakat bu fixture'daki 12 bireyin tamamı 300 tick dolmadan öldü. Çayır ve seyrek kaynak fixture'larında kural hiç tetiklenmedi; bu iki ortamdaki nötrlük yarar veya güvenli genelleme kanıtı değildir. Seçenek varsayılan kapalı kalır; canlıya taşınmadı.

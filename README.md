# Curious Simulation

`Curious`, **kendine yetebilen ekosistem** oluşturmayı amaçlayan bir simülasyon ve simülasyon motorudur.
Döngüyle _(Tick)_ çalışan zamanlama ile Dünya _(World)_,
Karar verebilicek durumda olan Canlılara _(Entity)_ bir Görüş açısı _(Perception)_ sunan,
Kendi işlevleri ile bir Niyet _(Intent)_ alan,
Çakışan Niyetleri çözümleyen _(Intent Resolver)_,
Kuralları uygun olanları işleyen bir mekanizmaya sahiptir.
Her Niyetin bir maliyeti vardır.

Her döngüde şu işlemler yapılır:
- Canlıların metabolizmaları çalışır
- Ölmesi gerekenler Ceset durumuna alınır ve karar veremez.
- Süresi dolan cesetler kaldırılır.
- Karar verebilicek durumda ki canlılara bir görüş açısı sunulur
- Görüş açısı yapılması gereken niyetleri toplayıp
- Çakışanları kendi içeriside çözerek
- Son adımda niyetleri dünyaya uygular

----

Bu projede amacım, olabildiğince gelişmiş bir simülasyon motoru sunmak.
İstersen kendi kurallarına uygun bir dünya inşaa edersin,
İstersen kendi canlılarını oluşturabilirsin.

Proje üç parçaya ayrılmış durumda; Map, World ve Entity.
İsimlendirmeler geçici, ileride değişmeleri yüksek olasılık.
Şuan ki durumda en iyi benzetmem; Ring, Hakem ve Boksörler.

Şuan hedeflediğim noktalar ile mevcut durum arasında dağlar var,
İleride çevreyi dinamik, daha esnek canlılar ve adil bir sistem yapmak istiyorum.

İlk hedefim dış mühahale olmadan uzun bir süre yaşayabilen bir ekosistem.
Sıralama olarak kısıtlı ama az kusurlu bir Entity iskeleti kurup,
Kendiyle çelişmeyen ve kısır döngüye yer vermeyen bir hale getirmek.
Map-Entity-World üçlüsü bir noktadan sonra doygun hale geldiğinde;
bir adım öteye gidip daha karışık sistemlere girişeceğim.

Projeye şuanlık bir arayüz geliştirmeyi düşünmüyorum.
Ne kadar yalın kalırsa, başka bir ortama geçirmesi o kadar kolay olur.
Wasm gibi yada Godot gibi bir motor için Extension olarak.
Game of Life ile yola çıkıp, günün birinde Dwarf Fortress gibi bir seviyeye gelmek istiyorum.

## Yeni Eklenenler (Özet)
- Su kaynağı ve susuzluk mekanizması.
- `Herbivore`, `Carnivore`, `Omnivore` türleri için temel davranışlar.
- `Drink` niyeti ile su tüketimi.

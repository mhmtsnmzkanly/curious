# 5 — Bireysel hafıza

Birey başına en fazla 32 kaynak, 8 tehlike konumu. Bilgi yalnız algıdaki
hücrelerden gelir; görüş dışı dünya sorgulanmaz. Kaynak bilgisi 100, tehlike
bilgisi 30 tickte eskir. Görülen boş kaynak kaydı kaldırılır. Etçil bitkiyi
hatırlanacak yiyecek olarak kullanmaz.

Görüşte su/bitki bulamayan birey hatırladığı konuma, şu an gördüğü yürünebilir
yönleri kullanarak yaklaşabilir. Gerekçe RecallWater/RecallFood olur. Bu henüz
öğrenme değildir; başarısına göre tercih güncellemez. Öğrenilmiş bilgi yavruya
aktarılmaz. Duvarın arkasındaki kaynağı bildiğini varsaymaz; karmaşık sapma
güzergâhı planlayan bir hafıza haritası henüz yoktur.

Checkpoint V5; eski checkpoint boş hafızayla yüklenir. Kaynak silinmesi/eski bilgi,
sınır, checkpoint ve aynı gelecekle devam testleri geçti. Tam gözlem ayarı RNG'yi
ve hafızanın geleceğini değiştirmez.

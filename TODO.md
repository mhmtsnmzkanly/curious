# Sıradaki işler

## 24 October 2026 — Wave 6 genotype measurement handoff

The bounded genotype measurement engineering run completed once at source
`0c06d72`, seen seed 123, 200 ticks per arm. Root's independent audit passed:
400 event totals and food/water stock-flow ticks total (200 per arm), lineage
and genotype joins, and the recorded tick-100 State+RNG round trips
reconciled. The raw capture is
losslessly archived at
`data/experiments/genotype-measurement-v1-engineering-20261024/raw.jsonl.zst`;
its verified index records original and archive hashes. Result and limits:
[`GENOTYPE_MEASUREMENT_V1`](docs/reports/GENOTYPE_MEASUREMENT_V1.md) and its
JSON companion. This is an engineering measurement pass only: one seen seed,
non-counterfactual paths, no fitness/selection conclusion, no formal seed, and
no deployment. Root reports 246 Rust tests / 0 failures (47 summaries), 177
Python tests / 0 failures, release app/probe, and scoped rustfmt passed. No
repeat lab occurred. A host reboot at 14:58:05 UTC preceded service startup
at 14:58:08; root did not deploy or restart services. Coexistence/replay were
observed active as PIDs 1136/1137 with restart count zero; retention timer was
active. `/tmp/curious-wave6-target` was already absent on resume; no cleanup
bytes are attributed. Check manifest:
`data/operations/wave6-checks-20261024/manifest.json`.

Next work requires a separately frozen design for a trait study, including
genotype-linked costs and food acquisition, before any formal run. Do not infer
trait benefit from this engineering result or extend/retry this run. The raw
JSONL is archived; restore it before use with readers that require uncompressed
JSONL. A separate read-only refuge opportunity diagnosis recorded eight fresh
choices and 17 candidate-row comparisons, with zero exact different-ID matches
and zero supported choices; it used zero World ticks. See
[`REFUGE_OPPORTUNITY_DIAGNOSIS`](docs/reports/REFUGE_OPPORTUNITY_DIAGNOSIS.md).

## 24 October 2026 — Wave 5 genotype observer handoff

Three pure observer files are integrated in the workspace: bounded ledger
[`genotype_audit.rs`](examples/support/genotype_audit.rs), pure fixtures
[`genotype_audit_check.rs`](examples/genotype_audit_check.rs), and contract
[`GENOTYPE_AUDIT.md`](docs/GENOTYPE_AUDIT.md), committed as `6f60d9c`. This is
read-only observation of provided checkpoints, events, and traces; measured
scope for the six new genotype fixture tests is zero World ticks; the full
suite includes existing World tests. There is no genotype World driver, policy
input, trait selection, World run, seed role, or live deployment. Root reports
241 all-target Rust tests / 0 failures (46 groups), 177 Python tests / 0
failures, rustfmt for two Rust files, `git diff --check`, and release build
(exit 0, 1m44s) passed. Release log:
`data/operations/wave5-release.log`. Cleanup removed 608,011,561 logical bytes
(~579.84 MiB) from the inactive genotype audit target; proof:
`data/operations/wave5-cleanup-20261024.json`. Live PIDs 1126/75038 stayed
active and unchanged with no restart; no deployment. See
[`GENOTYPE_OBSERVER_WAVE5`](docs/reports/GENOTYPE_OBSERVER_WAVE5.md) and
`data/operations/wave5-integration-inputs.json` for source hashes/scope.

Next: define a new refuge opportunity protocol; then add and validate a
genotype observer driver with food and paid-cost measurement gates before any
habitat/trait-selection run. A separate refuge technical validation completed
270 planned ticks; see
[`REFUGE_REPAIR_VALIDATION`](docs/reports/REFUGE_REPAIR_VALIDATION.md). Wave 4
notes below preserve their historical results.

## 24 October 2026 — Wave 4 final handoff (engineering incomplete)

G archive-budget source `8f3f6d0` is integrated at main `42fc7f6`; selected
shards are recounted before a cached-budget rejection, and stat errors fail
closed. This remains a soft history guard, not a strict quota. Root's combined
checks passed: 235 Rust tests / 0 failures (45 summaries), 177 Python tests / 0
failures, release app/probe/replay, HTTP/auth/writer-lock/SIGKILL. Candidate
`bin/releases/curious-budget-8f3f6d0` is retained; live V12 was not upgraded.

C protocol `1a074ba` / main `935015c` is design only: one accepted habitat,
expressed vision allele versus neutral marker, no seeds or World runs. The extra
RNG consumption from the feature toggle is a confound; genotype-linked food and
energy audit must exist before any trait run.

B's single engineering World run (seen seed 123, focal 15) failed at 225
complete audit records from a 270-tick plan under a 1 GiB allocation cap. Rule
arm has 15/60 records; supported greedy choices are zero. No formal seed or
benefit, ineffectiveness, utility, or biological rejection conclusion follows.
The memory repair `3c2a83d` / main `e551263` passed six pure tests and compiled
release lab/replay. Zero-tick replay read 225 records / 49,022,469 bytes in 0.91
s at 40,820 KiB peak RSS; it stored 120 individual states (60 pairs) as
26,294,126 canonical bytes (max 240,987). Only 14/60 normalized online/frozen
full-state pairs matched. It ran zero World ticks, made no summary, and did not
complete or rerun the probe. Key diagnosis saw only threat 37; tick 56 had four
candidates versus one row, with 8–9 differing key fields. No cross-ID exposure
was observed; this does not isolate exact-key strictness as the cause.

The raw source was losslessly archived: 49,022,469 to 2,723,477 bytes; original
SHA-256 `15ce046035061e24fbd969cc9056ae32cc8f423926705b5e168c55a0a830c4a0`,
archive SHA-256 `4b4c9bb594d1a9118ff2a74338bdb8a629928d786032f501d89b12dbe02c0464`.
Index: `data/experiments/refuge-escape-v1-engineering-20261024/raw.jsonl.archive.json`.
Restore raw with the evidence helper before using readers that need the original
path. The failed manifest remains `failed`; protocol snapshot, executed source
`5ae985e`, and three binaries remain preserved. Hash-bound replay proof is
`data/experiments/refuge-escape-v1-engineering-20261024/memory-fix-validation.json`.

Root cleanup removed 984,738,373 logical bytes (~939.12 MiB) from three inactive
build caches; proof: `data/operations/wave4-cleanup-20261024.json`. Services
7882/7883 and the retention timer remain active and unchanged. No Wave 4
biology/ML/formal worker or wake is pending. Next: define a new protocol for
refuge context opportunity; implement genotype-linked cost/food auditing before
trait selection. No engine upgrade without qualification. Source and diagnosis
stages are complete; refuge engineering remains incomplete. See
[WAVE4_PROGRESS](docs/reports/WAVE4_PROGRESS.md) and [JSON](docs/reports/WAVE4_PROGRESS.json).

Follow-up: the fixed technical validation passed with all 270 planned ticks in
1.578 s at 44,480 KiB peak RSS. All 235 common JSONL records were equal,
including all 225 earlier tick records; treatments and frozen rows passed the
root audit. The earlier run remains failed. No post-prefix opportunity was
identified (zero supported fresh greedy choices, zero different-ID supported
choices, zero aligned online/frozen contrasts), so there is no utility or
biological conclusion. No formal seed role or deployment. Refuge opportunity
remains unresolved; raw is losslessly archived and must be restored before
frozen readers. See
[report](docs/reports/REFUGE_REPAIR_VALIDATION.md) and
[JSON](docs/reports/REFUGE_REPAIR_VALIDATION.json).

## 24 October 2026 — current retention and evidence handoff

At main `904d4cc`, root independently passed all 8 retention tests and a
real closed-Zstandard integration check (17 records, 15,876 events); the live
source was untouched by those checks. The frozen retention script is
`/root/curious-runtime/releases/history-retention-904d4cc/history_retention.py`
(SHA-256 `2331196cec3c827f03546102265bd1b331b2c7d81bcf1cff8f5ecd6d3de8bee9`).
Its runtime path is set in `data/history-retention-runtime.env`. The 15-minute
systemd timer is enabled; first service result was success with no eligible
closed shards. Policy: retain 7 days, target 3 GiB compressed closed shards,
maximum 2 shards/run. Raw files, newest shard, WAL, and unsupported/corrupt
archives remain protected. The compressed-shard budget is not a total storage
bound: protected files can exceed it and permanent summaries continue to grow.

Root retired the detailed histories of the stopped `main`, `meadow`, `lab`, and
`evolution` worlds. Four minimal resume SQLite files and checkpoint proofs were
retained. Final ticks were 10, 77,974, 30, and 74,075; plan and source hashes
are in `data/retired-worlds/20261024T015208Z/plan.json`. Net detailed-history
bytes removed: 929,698,848; checkpoint backup bytes: 292,787. Running PIDs for
ports 7882/7883 were unchanged. The prior four services are off, the old
watchdog is disabled, and there is no pending wake. No biological/ML experiment or archive batch remains running.

Root archived all 21 selected completed experiment JSONL files of at least
8 MiB with the lossless helper. Per-file archive and index retain verified original and
compressed byte hashes and the recognized seed/role index; protocols and
manifests remain unchanged: 710 original metadata files were hash-checked.
The JSONL total fell from 1,060,085,984 to 52,274,679 bytes. All 2,607 declared
seed values remain visible through the existing scanner. A real 200-tick
calibration was restored byte-for-byte and read by its frozen validator, then
repacked. The resumed full Python suite passed 177/177. See
[retention report](docs/reports/HISTORY_RETENTION_20261024.md). Restore raw JSONL
before running frozen analyzers or scanners that require original paths. The
engine's in-memory `Store.archived_bytes` remains stale after external pruning;
its 32 GiB ceiling is therefore not a durability guarantee. Do not restart the
engine to refresh it or treat external pruning as changing the live engine.

No automatic live binary upgrade or broad data deletion is authorized by this
handoff. See [history retention](docs/HISTORY_RETENTION.md) and
[experiment evidence archives](docs/EVIDENCE_ARCHIVES.md).


Hedef [VISION](docs/VISION.md), kullanıcı sırası ve dürüst durum
[ROADMAP](docs/ROADMAP.md), ölçümler [sonuç raporu](docs/reports/ROADMAP_RESULTS.md).

## 24 Ekim — güncel wave-3 A/B/C handoff

A zero-tick tanısı (`d695a0d`): 6/257 fresh greedy sıralaması residual
kaldırılınca değişti; 281 eligible online residual örneği 1 pozitif, 261
sıfır, 19 negatif (toplam −25.000), kayıtlı update toplamıyla eşleşti. Ön
kayıtlı physical utility geçmedi. [Rapor](docs/reports/WATER_RESIDUAL_DIAGNOSIS.md).

B V18 represented-context adayı `9ae92c3`, main `1c5aefe`: default-off exact
context, gerçek World/resolver/checkpoint doğrulaması ve 225 Rust test geçti;
ekolojik utility henüz doğrulanmadı.

C current V17 accepted fixture profili koşullu olarak geçti: calibration
53099×200 ve formal 53001–53003×500; 1.700 flow tick. Tür tükenmesi yok, iki
türün yaşayan üreyen yavru-soyları ve son 50 tickte doğumları var. Root flow
ticklerini/soy ağacını ve 19 tam durum/olay kaydında pre/post stok, typed-resource/
event-count eşitliği ve tick sürekliliğini doğruladı (RNG geçiş testi değil).
Kaynak `466f9a0`, main `1f791c1`. Bu V18 qualification veya eski binary
reproduction değildir. [C raporu](docs/reports/ECOLOGY_FIXTURE_V1.md).

Root'un elle yaptığı denetim post-freeze kaynak farkını bulup zero-World
preflight'ı durdurdu; runner bu farkı startup sonrası kendisi yakalayamadı.
Kaynak değişikliği geri alındı. Registered seed listeleri sonra birer kez çalıştırıldı.
Birleşik doğrulama: 225 Rust, 159 Python, release app/probe, HTTP/auth/SIGKILL;
V17/V18 default-off 40 frame full State+RNG eşitliği (yalnız version normalize).
1.148,7 MiB inactive target temizliği raw/probe/frozen source/app'i korudu;
7882/7883 etkin ve değişmedi, bounded lab/pending wake yok. Yeni batch/retune
yok. Gelecek refuge utility, water context-staleness ve habitat/trait-selection
işleri ayrı protokoller gerektirir.

## 24 Ekim — güncel A/B/C handoff

A V17 calibration 52099 1.400 tickte preregistered gates 8/8 geçti (20 prefix,
35 trained-online suffix fresh choices). Formal 52001–52008 toplam 11.200 tick
ve 164.554.619 byte raw ile tamamlandı; 32 focal branch'in tümü 500 tick yaşadı.
Root 32 physical vector'ı ve üç full normalized state/perception/body/RNG
greedy-score/resolver tanığını doğruladı (52002/247, 52003/415, 52006/241).

Bounded utility **FALSE**. Trained-online vs rules: acquired water −9 / paid
movement energy +32 / terminal energy +4 / terminal water −9 / health-survival
0. Vs trained-frozen: acquired water +5 / movement energy −13 / terminal energy
−3 / terminal water +5 / health-survival 0. Rules'a karşı 8/8 tradeoff;
frozen'a karşı 1 dominance, 2 tradeoff, 5 identical. V17 default-off kalır;
genel fayda, qualification, deployment, retune veya yeni batch yoktur. Tüm dokuz
registered seed rolü seen. [Rapor](docs/reports/WATER_RESIDUAL_V1.md) ve
[JSON](docs/reports/WATER_RESIDUAL_V1.json).

B refuge schema 13 synthetic check; C resource/lineage measurement 9 check ve
13 physical World tick ile doğrulandı. B/C production integration veya fitness
kanıtı değildir. C body energy unknown kalır; audit detail 2.000 satırla
sınırlıdır, identity/event state'i büyüyebilir. Seed123 engineering twin
2×1.400 tick / 1.400 matched frame formal evidence değildir. Root combined
checks: 211 Rust, 140 Python, release probe/app passed. Bu formal lab için aktif
process/pending wake yoktur; coexistence/replay services aktif ve değiştirilmedi.
Root target cleanup 967.6 MiB logical; raw/probe/protocol/frozen source korundu.
Yeni A işi context-stale residual/current-baseline redundancy için yeni
preregistration gerektirir; immediate retune yoktur. B/C integration root'un
ayrı kararıdır.

## 23 Ekim — paralel ekosistem planı ve sıradaki işler

Root, A/B/C ilk dalga teslimlerini inceledi; A'nın source integration adayı V17
olarak tamamlandı. `policy.water_residual_enabled` varsayılan kapalıdır; açıkken
güncel foraging puanına bireyin own-source resolver artığı eklenir. Choice-time
baseline episode boyunca sabit kalır. Exact bounded sum/count, own actor/source
attribution, persistent censoring, frozen update, V1–V16 migration, SQLite V2
unchanged, default-off RNG goldens ve yavruya boş learner doğrulandı.

Root final checks: Rust 191 passed / 0 failed (39 summaries), Python 132/0,
release build exit 0, HTTP/control/auth/SIGKILL-resume check exit 0, scoped
Rust formatting passed. A için formal deney veya seed seçilmedi; deployment,
live world ve yeni ecology batch yok. Sıradaki iş root preregistration ve seed
audit'tir: strong rule baseline, naive online, trained online, aynı eğitimli
tabloyla trained-frozen. Önceki 51001–51008 ve tarihsel su seedleri held-out
olarak tekrar kullanılmaz. B'nin compatibility calibration/schema tasarımı ve
C'nin fixture/resource muhasebesi ayrı açık işlerdir. Ayrıntı:
[PARALLEL_CAPABILITY_PLAN](docs/PARALLEL_CAPABILITY_PLAN.md).
- [ ] Kaynak şokundan toparlanmayı çöz: V9 sabit kurallı uzun doğrulama 30/30
  geçti; kaynak şoku 4/8 ile araştırmayı durdurdu. Teşhis ve sonraki sınırlı
  deney [RESOURCE_SHOCK_ANALYSIS](docs/reports/RESOURCE_SHOCK_ANALYSIS.md).
  202 beslenme hedefi / 238 üreme eşiği uyumsuzluğunu ve daralmada yerel eş
  bulmayı ayrı ölç. Otçul toparlanması avcı soyunun toparlanması anlamına gelmez.
  Öğrenme/mutasyon kapıları çalışmadı, yeni dünya açılmadı. Gizli takviye ekleme.
  Dad97e9 adayı normalde 7/8, şokta 6/8 ile reddedildi. V11 varsayılanı eski
  kurala döndürür; tek adımlık rezerv adayı açık tercih gerektirir.
  D36a10a normal 8/8, şok 7/8: seed 2 yaşlılıkla tükendi. Karşılıklı hazır
  olma şartını kaldıran yeni deneysel aday önce seed 2/3 hedefli şok elemesinden
  geçmeli; sonra normal/şok sekizli kontroller. Varsayılan kapalı kalır.
  C4d2b35 hedefli şokta 2/2 geçti, normal kontrolde 7/8 ile reddedildi:
  seed 2 otçulları 17.660 adımda tükendi; kalan şok sekizlisi başlatılmadı.
  Teşhis: av baskısı altında yerel gruplar dağılıyor; son iki birey hazır ama
  görüş dışındaki eşi bulamadan yaşlanıyor. Aynı 16.000 kaydında hazırlık
  kapalıysa 17.660'ta 535 otçul/44 avcı var. Ayrıntı/karşı-olgular:
  [PREY_COLLAPSE_SEED2](docs/reports/PREY_COLLAPSE_SEED2.md). Sıradaki dar
  mekanizma testi küçük grupta güvenli eş arama. V12 opt-in ilk buluşma adayı
  erken kopyada20.000'e449 otçul/51 avcıyla ulaştı,17.400 geç kopyasını
  kurtaramadı; kabul ve uzun tarama yok. [MATE_RENDEZVOUS](docs/reports/MATE_RENDEZVOUS.md).
  Bu incelemede bilinen dolu sığınağa kaçma ve yeniden denemede güvenli hazır
  eşi kaçırma düzeltildi. Birleşik erken597 otçul/45 avcı; geç646/0.
  [LATE_RENDEZVOUS_FIXES](docs/reports/LATE_RENDEZVOUS_FIXES.md).
  Erişilebilir leş algısı düzeltildi; geç24.000 kopyasında606 otçul/28 avcı.
  [PREDATOR_CORPSE_ACCESS](docs/reports/PREDATOR_CORPSE_ACCESS.md).
  Yeni kaynak normal8/8 geçti; shock6/8 reddedildi. Seed3/2026 avcıları
  şok sonrası az sayıda doğumla yaşlanarak tükendi. Aynı durum+RNG auditinde
  27/23 yerinde rezerv öğünü yalnız eş mesafesi yüzünden engellenmiş.
  Deneysel avcı hazırlığında görünür yetişkin yeterli; yerinde öğün şartı
  korunur. Otçul ve fizik değişmedi.
  [SHOCK_PREDATOR_RECOVERY](docs/reports/SHOCK_PREDATOR_RECOVERY.md).
  32ddfb2 hedefli2/2, normal8/8, shock8/8 geçti.
  [Uzun protokol](docs/RESERVE_QUALIFICATION.md):±%10 duyarlılık, yeni
  20001–20030 fixed100k, öğrenme/mutasyon kontrolleri, both100k.
  20 Ekim: bütün11kapı geçti; fixed30/30 ve both30/30×100k.
  Ayrı karma dünyanın kayıtlı başlangıcı ve gerçek süreli gözlemi:
  [QUALIFIED_OBSERVATION](docs/reports/QUALIFIED_OBSERVATION.md).
  [RESERVE_POLICY_ISOLATION](docs/reports/RESERVE_POLICY_ISOLATION.md) ve
  [önceki aday](docs/reports/RESERVE_OPPORTUNITY.md).
- [ ] Su öğrenmesinin ortam değişimi deneyini değerlendir (`learning_trial` reversal);
  V10 su denemeleri gecikmiş sonucu ve gerçek hareket maliyetini izliyor.
  [Öğrenme sözleşmesine](docs/LEARNING_NEXT.md) göre sıradaki iş sınırlı av
  tercihi; yeni öğreniciyi ayrıca karma ekosistemde doğrula.
- [ ] Gerçek 1/7/30 gün gözlemini değerlendirme; raporlayıcı otomatik Notify gönderir.
- [ ] Kalıtım deneyini av baskısında tekrarlama; dağılım değişimini sürüklenmeden ayırma.
- [ ] Ön kayıtlı kaynak döngüsü ve bedelli görüş/dayanıklılık alleli adayını
  mekanizma testleri, temiz kaynak incelemesi ve sınırlı deneyden geçir;
  held-out sonuçları görülmeden protokolü veya parametreleri değiştirme.
- [x] İlk soy görünümü: ebeveyn–birey–yavru bağlantıları, numarayla ölmüş birey
  seçimi ve sayfalı karar arşivi. Kullanım/sınırlar CONTROL.md soy bölümü.
- [ ] Daha büyük nüfusta SQLite/HTTP maliyetini ölçme; motor-only hızını kayıt hızı sanma.
- [ ] Disk dolmadan uzun vadeli arşiv kapasitesi planı; geçmişi sessizce silme.
- [ ] Host dışından gözetim: aynı sunucudaki bütün izleyiciler birlikte durursa içeride
  çalışan bir timer bunu dışarı bildiremez.

Tamamlanan ilk uygulamalar: kaynak yenilenmesi/nesiller, deterministik RNG,
SQLite+arşiv+checkpoint, kontrol/gözlem, hareket sözleşmeleri, bireysel hafıza,
sınırlı öğrenme, bedelli kalıtım/mutasyon, responsive gözlem ekranı, nüfus ölçümü,
izleyici gözetimi, sürümlü ikili/yedekli laboratuvar dağıtımı. Test/commit/temizlik/
önemli Notify döngüsü devam eder.

21 Ekim öğrenme geliştirmesi: varsayılan kapalı av seçimi adayı+fiziksel
muhasebe+V13 kurtarma ve deneyim görünümü eklendi. Av öğrenmesinin gerçek
karşılaştırmalı eğitim/frozen/naive deneyleri ve ayrı ekosistem kabulü açık;
mevcut7882dünya bu adayla yükseltilmez. Su ölçümü EXPERIENCE_PROTOCOL.md
sözleşmesiyle çalışır; sonuçlar deneyim raporunda tutulur.

# Genotype lifecycle observer

This is a pure observer helper for the proposed habitat and heritable-trait
study. `examples/support/genotype_audit.rs` consumes explicit before and after
checkpoint `State` values, the events produced by the tick, and recorded
`DecisionTrace` values. It does not construct or advance a World and must not
be connected to policy input.

The initial State passed to `seed` defines the founder cohort for this audit,
even if some initial individuals already carry nonzero generation or parent
metadata. It is an experimental starting cohort, not necessarily biological
generation zero. The observer rejects reseeding; create a fresh observer for a
different cohort. `observe` rejects calls before a successful seed.

The ledger is keyed by entity ID and retains initial-cohort status, species, full
mobility/vision/stamina genome, generation, parents, current expressed vision
range and stamina, mating/birth/death status, event-backed maintenance
request/payment, acquired plant food, corpse food, water, and trace encounter
counts. Genotype summaries use species plus all three alleles. Food perception
counts each trace food entry as an encounter, so repeated observations remain
repeated counts; plant and corpse entries are separate. These are recorded
perceptions, not discoveries or mental state. A missing trace or absent full
perception increments an explicit unknown-coverage counter and never implies
zero food encounters.

Same-tick mating and birth events join to the child in the after-state and
parents in the preexisting ledger. Parent/generation mismatches and missing
joins are issues. A child genome outside both parental genomes is an issue
when configured genetics cannot produce it. With genetics disabled, the engine
copies the target/event parent 2 genome. With genetics enabled, mobility may
mutate; vision and stamina can mutate only when extra traits are enabled. A
legal differing genome at a positive configured mutation rate is annotated as
a possible mutation and is not called a transmission error. This observer does
not reconstruct which mutation occurred. After-state IDs without a birth event
are retained with unknown lineage and excluded from descendant counts; a birth
without valid mating/parent lineage is also unknown. It does not infer
basal metabolism from trait cost, fake a neutral phenotype, infer causal trait
benefits, or assume learned knowledge is inherited.

The summary reports allele value counts and frequencies for founder, living,
descendant, living descendant, and living descendant breeder cohorts, grouped
by species. Genotype rows retain full allele identity; phenotype range and
stamina are reported as sums and should be interpreted alongside individual
ledger records.

The ledger and issue list are exposed through read-only accessors. The fixed
caps are 50,000 retained IDs and 2,000 issue records. `seed` checks
ID capacity before changing state. `observe` works on cloned bounded ledger
and issue vectors and commits only after all capacity checks succeed; a budget
error leaves the observer unchanged. A real driver must surface that error and
must also bound its own event and trace input allocations.

Run the hand-built pure fixtures with:

```sh
CARGO_TARGET_DIR=/tmp/genotype-audit-wave5-target cargo test --example genotype_audit_check -j1
```

The fixtures exercise event-backed payment/acquisition totals, unknown trace
coverage, same-tick lineage joining, mutation-aware transmission checking, and
atomic cap failure. They do not establish World behavior or trait fitness.

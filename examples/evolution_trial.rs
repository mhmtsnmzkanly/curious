//! Standing variation and mutation measured in a common, finite-resource habitat.
use curious::{entity::genetics::Genome, scenario::Scenario};
fn main() {
    for seed in [1, 2, 3, 4, 5, 42, 12345, 2026] {
        for mutation in [0, 20] {
            let mut w = serde_json::from_str::<Scenario>(include_str!("../scenarios/meadow.json"))
                .unwrap()
                .build(seed)
                .unwrap();
            w.genetics.mutation_per_mille = mutation;
            for (n, e) in w.entities.iter_mut().enumerate() {
                let g = Genome {
                    mobility: (n % 2) as u8,
                };
                g.express_from(&e.genome, e.base.life_mut());
                e.genome = g;
                e.learner.enabled = false;
            }
            let mut samples = vec![];
            for tick in 0..=2000 {
                if [0, 100, 500, 1000, 2000].contains(&tick) {
                    let mut alleles = [0usize; 3];
                    let mut generation = 0;
                    for e in &w.entities {
                        if e.base.life().is_alive() {
                            alleles[e.genome.mobility as usize] += 1;
                            generation = generation.max(e.generation);
                        }
                    }
                    samples.push(serde_json::json!({"tick":tick,"alleles":alleles,"max_living_generation":generation}));
                }
                if tick < 2000 {
                    w.tick();
                }
            }
            println!(
                "{}",
                serde_json::json!({"seed":seed,"mutation_per_mille":mutation,"samples":samples})
            );
        }
    }
}

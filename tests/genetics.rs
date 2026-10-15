use curious::{
    entity::{
        Entity,
        genetics::{Genetics, Genome},
        intent::Intent,
    },
    random::Random,
    scenario::Scenario,
};
#[test]
fn mutation_is_bounded_and_expression_has_a_real_energy_cost() {
    let mut rng = Random::new(42);
    let settings = Genetics {
        enabled: true,
        mutation_per_mille: 1000,
    };
    let mut g = Genome::default();
    for _ in 0..1000 {
        g = g.child(&g, &mut rng, &settings);
        g.validate().unwrap();
    }
    let mut w = serde_json::from_str::<Scenario>(include_str!("../scenarios/meadow.json"))
        .unwrap()
        .build(1)
        .unwrap();
    let mut body = curious::creatures::herbivore::HerbivoreEntity::default();
    let energy = body.life().energy;
    let speed = body.life().speed;
    Genome { mobility: 1 }.express_from(&Genome::default(), body.life_mut());
    assert_eq!(body.life().speed, speed + 1);
    body.life_mut().on_move(3);
    assert_eq!(body.life().energy, energy - 6);
    w.genetics.mutation_per_mille = 1000;
    for e in &mut w.entities {
        e.base.life_mut().age = 30;
    }
    w.push_command(1, Intent::Mate { target_id: 2 });
    w.tick();
    let child = w.entities.iter().find(|e| e.generation == 1).unwrap();
    assert_eq!(child.genome.mobility, 1);
    assert!(child.learner.values.is_empty());
    assert!(child.memory.sites.is_empty());
}

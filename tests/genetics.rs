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
        extra_traits_enabled: false,
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
    Genome {
        mobility: 1,
        ..Genome::default()
    }
    .express_from(&Genome::default(), body.life_mut());
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

#[test]
fn paid_alleles_are_opt_in_expressed_and_inherited() {
    let mut settings = Genetics::default();
    settings.mutation_per_mille = 0;
    settings.extra_traits_enabled = true;
    let parent = Genome {
        vision: 2,
        stamina: 1,
        ..Genome::default()
    };
    let child = parent.child(&parent, &mut Random::new(9), &settings);
    assert_eq!(child.vision, 2);
    assert_eq!(child.stamina, 1);

    let mut body = curious::creatures::herbivore::HerbivoreEntity::default();
    child.express_extra_from(&Genome::default(), body.life_mut());
    assert_eq!(body.life().vision_range, 8);
    assert_eq!(body.life().max_stamina, 20);
    assert_eq!(body.life().stamina, 20);
    assert_eq!(body.life().vision_metabolic_cost, 2);
    assert_eq!(body.life().stamina_metabolic_cost, 1);
    body.life().validate().unwrap();

    settings.extra_traits_enabled = false;
    let mut expected_rng = Random::new(991);
    let expected_child = if expected_rng.range(0, 1) == 0 {
        parent.clone()
    } else {
        parent.clone()
    };
    let _ = expected_rng.range(0, 999); // mutation rate is zero
    let mut actual_rng = Random::new(991);
    let actual_child = parent.child(&parent, &mut actual_rng, &settings);
    assert_eq!(expected_child, actual_child);
    assert_eq!(expected_rng.state(), actual_rng.state());
}

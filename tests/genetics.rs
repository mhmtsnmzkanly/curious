use curious::{
    entity::{
        Entity,
        genetics::{Genetics, Genome},
        intent::Intent,
    },
    random::Random,
    scenario::Scenario,
    simulation::SimulationEvent,
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
    w.entities[0]
        .learner
        .residuals
        .push(curious::entity::learning::ResidualValue {
            pos: curious::map::movement::Position::new(3, 3),
            sum: 700,
            visits: 2,
        });
    w.push_command(1, Intent::Mate { target_id: 2 });
    w.tick();
    let child = w.entities.iter().find(|e| e.generation == 1).unwrap();
    assert_eq!(child.genome.mobility, 1);
    assert!(child.learner.values.is_empty());
    assert!(child.learner.residuals.is_empty());
    assert!(child.learner.episode.is_none());
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

#[test]
fn allele_cost_payment_and_reproducing_offspring_lineage_are_separated_from_mere_survival() {
    let mut w = serde_json::from_str::<Scenario>(include_str!("../scenarios/meadow.json"))
        .unwrap()
        .build(1)
        .unwrap();
    w.genetics.enabled = true;
    w.genetics.extra_traits_enabled = true;
    w.genetics.mutation_per_mille = 0;

    // Truncate to 4 founders with known roles:
    // Founders 1 & 2: high alleles (mobility: 2, vision: 2, stamina: 2) -> will reproduce
    // Founders 3 & 4: low alleles (mobility: 0, vision: 0, stamina: 0) -> 3 & 4 will survive without reproducing
    w.entities.truncate(4);

    for i in 0..2 {
        let g = Genome {
            mobility: 2,
            vision: 2,
            stamina: 2,
        };
        w.entities[i].genome = g.clone();
        g.express_from(&Genome::default(), w.entities[i].base.life_mut());
        g.express_extra_from(&Genome::default(), w.entities[i].base.life_mut());
        w.entities[i].base.life_mut().age = 20;
        w.entities[i].base.life_mut().energy = 100;
        w.entities[i].base.life_mut().water = 100;
        w.entities[i].base.life_mut().reproduction_cooldown = 10;
    }

    for i in 2..4 {
        let g = Genome {
            mobility: 0,
            vision: 0,
            stamina: 0,
        };
        w.entities[i].genome = g.clone();
        g.express_from(&Genome::default(), w.entities[i].base.life_mut());
        g.express_extra_from(&Genome::default(), w.entities[i].base.life_mut());
        w.entities[i].base.life_mut().age = 20;
        w.entities[i].base.life_mut().energy = 100;
        w.entities[i].base.life_mut().water = 100;
        w.entities[i].base.life_mut().reproduction_cooldown = 10;
    }

    // 1. Verify per-step movement energy payment: 1 + mobility
    assert_eq!(w.entities[0].base.life().move_energy_cost, 1 + 2); // 3
    let e0_before = w.entities[0].base.life().energy;
    w.entities[0].base.life_mut().on_move(2);
    assert_eq!(w.entities[0].base.life().energy, e0_before - 2 * 3);

    assert_eq!(w.entities[2].base.life().move_energy_cost, 1 + 0); // 1
    let e2_before = w.entities[2].base.life().energy;
    w.entities[2].base.life_mut().on_move(2);
    assert_eq!(w.entities[2].base.life().energy, e2_before - 2 * 1);

    // Reset energy for metabolism and mating verification
    w.entities[0].base.life_mut().energy = 100;
    w.entities[2].base.life_mut().energy = 100;

    // Tick 1: verify per-tick metabolic maintenance
    w.push_command(1, Intent::Idle { duration: 1 });
    w.push_command(2, Intent::Idle { duration: 1 });
    w.push_command(3, Intent::Idle { duration: 1 });
    w.push_command(4, Intent::Idle { duration: 1 });
    let events = w.tick();
    let meta_e0 = events
        .iter()
        .find_map(|e| match e {
            SimulationEvent::EntityMetabolized {
                id,
                vision_cost,
                stamina_cost,
                vision_paid,
                stamina_paid,
            } if *id == 1 => Some((*vision_cost, *stamina_cost, *vision_paid, *stamina_paid)),
            _ => None,
        })
        .expect("Entity 1 (high vision/stamina) must emit EntityMetabolized");
    assert_eq!(meta_e0, (2, 2, 2, 2));

    // Entity 3 has vision 0, stamina 0, so no extra metabolic cost is charged
    let meta_e2 = events.iter().find_map(|e| match e {
        SimulationEvent::EntityMetabolized { id, .. } if *id == 3 => Some(()),
        _ => None,
    });
    assert!(meta_e2.is_none());

    // 3. Populate parents with learned experience and verify newborn starts completely empty
    w.entities[0]
        .learner
        .values
        .push(curious::entity::learning::Value {
            pos: curious::map::movement::Position::new(1, 1),
            visits: 5,
            value: 120,
        });
    w.entities[0]
        .learner
        .residuals
        .push(curious::entity::learning::ResidualValue {
            pos: curious::map::movement::Position::new(2, 2),
            sum: 300,
            visits: 1,
        });
    w.entities[0].learner.episode = Some(curious::entity::learning::WaterEpisode {
        target: curious::map::movement::Position::new(1, 1),
        ticks: 3,
        movement_energy: 9,
        incoming_damage: 0,
        choice_baseline: None,
        outcome_overflowed: false,
        candidate_censored: false,
    });
    w.entities[0]
        .hunt_learner
        .values
        .push(curious::entity::hunt_learning::HuntValue {
            context: curious::entity::hunt_learning::HuntContext {
                energy: 1,
                stamina: 1,
                distance: 1,
                health: 1,
            },
            visits: 3,
            value: 50,
        });
    w.entities[0]
        .memory
        .sites
        .push(curious::entity::memory::KnownSite {
            pos: curious::map::movement::Position::new(3, 3),
            kind: curious::environment::ResourceKind::Water,
            seen: 1,
        });
    w.entities[0]
        .memory
        .dangers
        .push((curious::map::movement::Position::new(4, 4), 1));
    w.entities[0]
        .memory
        .refuge_outcomes
        .push(curious::entity::refuge_learning::RefugeOutcome {
            key: curious::entity::refuge_learning::RefugeKey {
                threat_id: 10,
                threat_species: curious::entity::species::Species::Carnivore,
                bearing: 0,
                threat_distance: 1,
                energy_quartile: 2,
                stamina_quartile: 2,
                refuge_options: vec![],
                endpoint: curious::map::movement::Position::new(0, 0),
            },
            last_observed_tick: 1,
            completed: 1,
            safe_arrivals: 1,
            threat_damage: 0,
            movement_energy: 4,
        });

    // Make parents ready to mate
    w.entities[0].pos = curious::map::movement::Position::new(-2, -2);
    w.entities[1].pos = curious::map::movement::Position::new(-1, -2);
    w.entities[0].base.life_mut().age = 30;
    w.entities[1].base.life_mut().age = 30;
    w.entities[0].base.life_mut().energy = 100;
    w.entities[1].base.life_mut().energy = 100;
    w.entities[0].base.life_mut().water = 100;
    w.entities[1].base.life_mut().water = 100;
    w.entities[0].base.life_mut().reproduction_cooldown = 0;
    w.entities[1].base.life_mut().reproduction_cooldown = 0;

    w.push_command(1, Intent::Mate { target_id: 2 });
    w.push_command(2, Intent::Idle { duration: 1 });
    w.tick();

    let f1_child = w
        .entities
        .iter()
        .find(|e| e.generation == 1)
        .expect("F1 child must be born");

    // Inherits finite alleles in 0..=2
    assert!(f1_child.genome.validate().is_ok());
    assert_eq!(f1_child.genome.mobility, 2);
    assert_eq!(f1_child.genome.vision, 2);
    assert_eq!(f1_child.genome.stamina, 2);

    // Learned experience is NOT inherited: empty on newborn
    assert!(f1_child.learner.values.is_empty());
    assert!(f1_child.learner.residuals.is_empty());
    assert!(f1_child.learner.episode.is_none());
    assert!(f1_child.hunt_learner.values.is_empty());
    assert!(f1_child.hunt_learner.risk_values.is_empty());
    assert!(f1_child.hunt_learner.episode.is_none());
    assert!(f1_child.memory.sites.is_empty());
    assert!(f1_child.memory.dangers.is_empty());
    assert!(f1_child.memory.refuge_outcomes.is_empty());

    // 4. Have F1 child reproduce to create F2 (forming a reproducing lineage)
    let f1_id = f1_child.id;
    let f1_idx = w.entities.iter().position(|e| e.id == f1_id).unwrap();
    w.entities[f1_idx].base.life_mut().age = 30;
    w.entities[f1_idx].base.life_mut().energy = 100;
    w.entities[f1_idx].base.life_mut().water = 100;
    w.entities[f1_idx].base.life_mut().reproduction_cooldown = 0;

    let p2_idx = w.entities.iter().position(|e| e.id == 2).unwrap();
    w.entities[p2_idx].base.life_mut().age = 30;
    w.entities[p2_idx].base.life_mut().energy = 100;
    w.entities[p2_idx].base.life_mut().water = 100;
    w.entities[p2_idx].base.life_mut().reproduction_cooldown = 0;

    // Move F1 adjacent to partner ID 2 to ensure birth space
    let p2_pos = w.entities[p2_idx].pos;
    w.entities[f1_idx].pos = curious::map::movement::Position::new(p2_pos.x, p2_pos.y + 1);

    w.push_command(f1_id, Intent::Mate { target_id: 2 });
    w.tick();

    let f2_child = w
        .entities
        .iter()
        .find(|e| e.generation == 2)
        .expect("F2 grandchild must be born");
    assert_eq!(f2_child.generation, 2);
    assert_eq!(f2_child.genome.mobility, 2);

    // Keep non-reproducing founders 3 and 4 well fed and alive
    for e in &mut w.entities {
        if e.id == 3 || e.id == 4 {
            e.base.life_mut().energy = 100;
            e.base.life_mut().water = 100;
            e.base.life_mut().health = 100;
        }
    }

    // 5. Explicitly separate survivor allele frequency from reproducing-lineage allele frequency
    let survivors: Vec<_> = w
        .entities
        .iter()
        .filter(|e| e.base.life().is_alive())
        .collect();
    assert!(survivors.len() >= 5);

    // Allele frequency among ALL survivors (includes non-reproducing founders 3 & 4)
    let survivor_mobility_2_count = survivors.iter().filter(|e| e.genome.mobility == 2).count();
    let survivor_mobility_0_count = survivors.iter().filter(|e| e.genome.mobility == 0).count();
    assert!(
        survivor_mobility_0_count >= 2,
        "Founders 3 & 4 survived without reproducing"
    );
    let survivor_freq_mobility_2 = survivor_mobility_2_count as f64 / survivors.len() as f64;
    assert!(
        survivor_freq_mobility_2 < 1.0,
        "Survivor frequency is strictly < 1.0 due to surviving non-reproducing founders"
    );

    // Reproducing lineages: founders/F1 whose offspring also reproduced
    let mut parent_to_children: std::collections::HashMap<usize, Vec<usize>> =
        std::collections::HashMap::new();
    for e in &w.entities {
        for &parent_id in &e.parents {
            parent_to_children.entry(parent_id).or_default().push(e.id);
        }
    }
    let has_reproducing_offspring = |id: usize| -> bool {
        if let Some(children) = parent_to_children.get(&id) {
            children
                .iter()
                .any(|child_id| parent_to_children.contains_key(child_id))
        } else {
            false
        }
    };

    let reproducing_lineage_founders: Vec<_> = w
        .entities
        .iter()
        .filter(|e| has_reproducing_offspring(e.id))
        .collect();
    assert!(
        !reproducing_lineage_founders.is_empty(),
        "Must have reproducing lineages"
    );

    let reproducing_mobility_2_count = reproducing_lineage_founders
        .iter()
        .filter(|e| e.genome.mobility == 2)
        .count();
    let reproducing_freq_mobility_2 =
        reproducing_mobility_2_count as f64 / reproducing_lineage_founders.len() as f64;

    assert_eq!(
        reproducing_freq_mobility_2, 1.0,
        "Reproducing lineage allele frequency is exactly 1.0 for mobility 2"
    );
    assert_ne!(
        survivor_freq_mobility_2, reproducing_freq_mobility_2,
        "Survivor allele frequency must be distinct from reproducing-lineage allele frequency: survival alone does not prove evolutionary representation"
    );
}

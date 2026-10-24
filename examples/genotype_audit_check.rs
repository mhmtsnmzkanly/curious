#[path = "support/genotype_audit.rs"]
mod genotype_audit;

#[cfg(test)]
mod tests {
    use crate::genotype_audit::{self, Audit, AuditError};
    use curious::{
        entity::{
            controller::EntityController,
            decision::{Decision, DecisionTrace},
            genetics::{Genetics, Genome},
            intent::Intent,
            lifestate::LifeState,
            perception::Perception,
            phase::EntityPhase,
            species::Species,
        },
        map::movement::Position,
        simulation::{
            SimulationEvent,
            checkpoint::{Individual, State},
        },
    };

    fn life() -> LifeState {
        LifeState {
            max_meal: 10,
            move_energy_cost: 1,
            max_stamina: 20,
            stamina: 10,
            exertion: 0,
            vision_metabolic_cost: 0,
            stamina_metabolic_cost: 0,
            death_cause: None,
            max_age: 1000,
            max_health: 100,
            max_energy: 100,
            max_water: 100,
            maturity_age: 0,
            vision_range: 6,
            age: 1,
            health: 100,
            energy: 80,
            water: 80,
            reproduction_cooldown: 0,
            reproduction_interval: 10,
            reproduction_reserve_per_mille: 500,
            speed: 3,
            moves_used: 0,
        }
    }
    fn individual(id: usize, generation: usize, parents: Vec<usize>, genome: Genome) -> Individual {
        Individual {
            genome,
            learner: Default::default(),
            hunt_learner: Default::default(),
            memory: Default::default(),
            generation,
            parents,
            corpse_energy: 0,
            id,
            pos: Position::new(id as isize, 0),
            phase: EntityPhase::Active,
            controller: EntityController::AI,
            species: Species::Herbivore,
            life: life(),
        }
    }
    fn state(entities: Vec<Individual>, mutation: u16) -> State {
        State {
            policy: Default::default(),
            genetics: Genetics {
                enabled: true,
                mutation_per_mille: mutation,
                extra_traits_enabled: true,
            },
            observation: Default::default(),
            environment: Default::default(),
            version: 18,
            tick: 0,
            rng: 1,
            next_entity_id: 100,
            bounds: [0, 10, 0, 10],
            entities,
            resources: vec![],
            commands: vec![],
        }
    }

    #[test]
    fn payment_food_and_trace_coverage_reconcile_by_genotype() {
        let genome = Genome {
            mobility: 1,
            vision: 1,
            stamina: 0,
        };
        let before = state(vec![individual(1, 0, vec![], genome.clone())], 0);
        let after = before.clone();
        let mut audit = Audit::default();
        audit.seed(&before).unwrap();
        let events = vec![
            SimulationEvent::EntityMetabolized {
                id: 1,
                vision_cost: 1,
                stamina_cost: 2,
                vision_paid: 1,
                stamina_paid: 1,
            },
            SimulationEvent::EntityAte {
                id: 1,
                corpse_id: None,
                pos: Position::new(1, 0),
                amount: 3,
            },
            SimulationEvent::EntityAte {
                id: 1,
                corpse_id: Some(9),
                pos: Position::new(1, 0),
                amount: 2,
            },
        ];
        let mut trace = DecisionTrace::new(
            1,
            1,
            Species::Herbivore,
            Position::new(0, 0),
            life(),
            1,
            Perception::empty(),
            Decision::new(
                Intent::Idle { duration: 1 },
                curious::entity::decision::Reason::Rest,
            ),
            true,
        );
        trace.perception.as_mut().unwrap().add_food(
            1,
            false,
            curious::map::movement::Steps::empty(),
        );
        trace.perception.as_mut().unwrap().add_food(
            1,
            true,
            curious::map::movement::Steps::empty(),
        );
        audit.observe(&before, &after, &events, &[trace]).unwrap();
        let g = &audit.individual(1).unwrap().totals;
        assert_eq!(
            (
                g.vision_requested,
                g.vision_paid,
                g.stamina_requested,
                g.stamina_paid
            ),
            (1, 1, 2, 1)
        );
        assert_eq!(
            (
                g.plant_food,
                g.corpse_food,
                g.observed_plant,
                g.observed_corpse
            ),
            (3, 2, 1, 1)
        );
        assert_eq!(
            audit.summary()["allele_frequencies"]["Herbivore:founders"]["individuals"],
            1
        );
        let mut missing = Audit::default();
        missing.seed(&before).unwrap();
        missing.observe(&before, &after, &[], &[]).unwrap();
        assert_eq!(missing.individual(1).unwrap().totals.missing_trace_ticks, 1);

        let mut partial = Audit::default();
        partial.seed(&before).unwrap();
        let trace_without_perception = DecisionTrace::new(
            1,
            1,
            Species::Herbivore,
            Position::new(0, 0),
            life(),
            1,
            Perception::empty(),
            Decision::new(
                Intent::Idle { duration: 1 },
                curious::entity::decision::Reason::Rest,
            ),
            false,
        );
        partial
            .observe(&before, &after, &[], &[trace_without_perception])
            .unwrap();
        assert_eq!(partial.individual(1).unwrap().totals.trace_ticks, 1);
        assert_eq!(partial.individual(1).unwrap().totals.missing_trace_ticks, 1);
    }

    #[test]
    fn same_tick_birth_joins_after_state_and_mutation_rate_classifies_transmission() {
        let parent = Genome {
            mobility: 0,
            vision: 0,
            stamina: 0,
        };
        let before = state(
            vec![
                individual(1, 0, vec![], parent.clone()),
                individual(2, 0, vec![], parent.clone()),
            ],
            0,
        );
        let child_genome = Genome {
            mobility: 0,
            vision: 1,
            stamina: 0,
        };
        let mut child = individual(3, 1, vec![1, 2], child_genome);
        let after = state(
            vec![
                individual(1, 0, vec![], parent.clone()),
                individual(2, 0, vec![], parent.clone()),
                child.clone(),
            ],
            0,
        );
        let events = vec![
            SimulationEvent::EntityMated {
                parent1: 1,
                parent2: 2,
                child: 3,
                pos: Position::new(0, 0),
            },
            SimulationEvent::EntityBorn {
                id: 3,
                species: Species::Herbivore,
                pos: Position::new(0, 0),
            },
        ];
        let mut audit = Audit::default();
        audit.seed(&before).unwrap();
        audit.observe(&before, &after, &events, &[]).unwrap();
        assert_eq!(audit.individual(3).unwrap().generation, 1);
        assert_eq!(audit.individual(3).unwrap().parents, vec![1, 2]);
        assert!(
            audit
                .issues()
                .iter()
                .any(|i| i["kind"] == "unexpected_genome_transmission")
        );
        let mut before_mutation = before.clone();
        before_mutation.genetics.mutation_per_mille = 1;
        let mut mutated = Audit::default();
        mutated.seed(&before_mutation).unwrap();
        mutated
            .observe(&before_mutation, &after, &events, &[])
            .unwrap();
        assert!(
            !mutated
                .issues()
                .iter()
                .any(|i| i["kind"] == "unexpected_genome_transmission")
        );
        assert!(
            mutated
                .issues()
                .iter()
                .any(|i| i["kind"] == "possible_mutated_genome_transmission")
        );
        child.genome = parent;
    }

    #[test]
    fn disabled_genetics_copies_event_parent_two_and_extra_trait_mutations_require_toggle() {
        let p1 = Genome {
            mobility: 1,
            vision: 1,
            stamina: 1,
        };
        let p2 = Genome {
            mobility: 0,
            vision: 2,
            stamina: 0,
        };
        let mut before = state(
            vec![
                individual(1, 0, vec![], p1.clone()),
                individual(2, 0, vec![], p2.clone()),
            ],
            1,
        );
        before.genetics.enabled = false;
        let child = individual(3, 1, vec![1, 2], p2.clone());
        let after = state(
            vec![
                individual(1, 0, vec![], p1.clone()),
                individual(2, 0, vec![], p2.clone()),
                child,
            ],
            1,
        );
        let events = vec![
            SimulationEvent::EntityMated {
                parent1: 1,
                parent2: 2,
                child: 3,
                pos: Position::new(0, 0),
            },
            SimulationEvent::EntityBorn {
                id: 3,
                species: Species::Herbivore,
                pos: Position::new(0, 0),
            },
        ];
        let mut audit = Audit::default();
        audit.seed(&before).unwrap();
        audit.observe(&before, &after, &events, &[]).unwrap();
        assert!(
            !audit
                .issues()
                .iter()
                .any(|i| i["kind"] == "unexpected_genome_transmission")
        );
        assert_eq!(audit.individual(3).unwrap().lineage_known, true);

        before.genetics.enabled = true;
        before.genetics.extra_traits_enabled = false;
        let mutated_extra = Genome {
            mobility: 1,
            vision: 1,
            stamina: 0,
        };
        let mut after_extra = after.clone();
        after_extra.entities[2].genome = mutated_extra;
        let mut no_extra_mutation = Audit::default();
        no_extra_mutation.seed(&before).unwrap();
        no_extra_mutation
            .observe(&before, &after_extra, &events, &[])
            .unwrap();
        assert!(
            no_extra_mutation
                .issues()
                .iter()
                .any(|i| i["kind"] == "unexpected_genome_transmission")
        );
    }

    #[test]
    fn bounds_reject_without_mutating_ledger() {
        let before = state(vec![individual(1, 0, vec![], Genome::default())], 0);
        let mut audit = Audit::default();
        audit.seed(&before).unwrap();
        let after = before.clone();
        let events = vec![
            SimulationEvent::EntityDied {
                id: 99,
                pos: Position::new(0, 0),
                species: Species::Herbivore,
                cause: None
            };
            genotype_audit::MAX_ISSUES + 1
        ];
        let old_count = audit.individual_count();
        assert_eq!(
            audit.observe(&before, &after, &events, &[]),
            Err(AuditError::IssueBudget)
        );
        assert_eq!(audit.individual_count(), old_count);
        assert_eq!(audit.mating_event_count(), 0);
        assert!(audit.issues().is_empty());
        let too_many = state(
            (0..=genotype_audit::MAX_IDS)
                .map(|n| individual(n, 0, vec![], Genome::default()))
                .collect(),
            0,
        );
        assert_eq!(Audit::default().seed(&too_many), Err(AuditError::IdBudget));
    }

    #[test]
    fn reseed_invalid_genome_and_unobserved_birth_fail_closed() {
        let valid = state(vec![individual(1, 0, vec![], Genome::default())], 0);
        let mut audit = Audit::default();
        assert_eq!(
            audit.observe(&valid, &valid, &[], &[]),
            Err(AuditError::NotSeeded)
        );
        audit.seed(&valid).unwrap();
        assert_eq!(audit.seed(&valid), Err(AuditError::AlreadySeeded));
        let mut invalid = state(
            vec![individual(
                8,
                0,
                vec![],
                Genome {
                    mobility: 3,
                    vision: 0,
                    stamina: 0,
                },
            )],
            0,
        );
        assert_eq!(
            Audit::default().seed(&invalid),
            Err(AuditError::InvalidGenome(8))
        );
        invalid.entities.clear();

        let unknown = individual(9, 4, vec![77, 78], Genome::default());
        let after = state(vec![valid.entities[0].clone(), unknown], 0);
        audit.observe(&valid, &after, &[], &[]).unwrap();
        assert!(
            audit
                .issues()
                .iter()
                .any(|i| i["kind"] == "new_id_without_birth_event")
        );
        assert!(!audit.individual(9).unwrap().lineage_known);
        assert_eq!(
            audit.summary()["genotypes"]["Herbivore:m0v0s0"]["descendants"],
            0
        );
    }

    #[test]
    fn death_event_retains_retired_individual_and_marks_it_dead() {
        let before = state(vec![individual(1, 0, vec![], Genome::default())], 0);
        let after = state(vec![], 0);
        let event = SimulationEvent::EntityDied {
            id: 1,
            pos: Position::new(0, 0),
            species: Species::Herbivore,
            cause: None,
        };
        let mut audit = Audit::default();
        audit.seed(&before).unwrap();
        audit.observe(&before, &after, &[event], &[]).unwrap();
        assert!(!audit.individual(1).unwrap().alive);
        assert_eq!(audit.individual(1).unwrap().totals.deaths, 1);
        assert_eq!(audit.individual_count(), 1);
    }
}

fn main() {}

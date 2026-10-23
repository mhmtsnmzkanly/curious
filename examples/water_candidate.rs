#[path = "support/water_candidate.rs"]
mod candidate;

#[cfg(test)]
mod tests {
    use super::candidate::{
        EpisodeEvidence, ScoreBoundsError, SourceResidual, actual_score, choose_index,
        record_episode_evidence, score_sources, select_index,
    };
    use curious::{
        creatures::herbivore::HerbivoreEntity,
        entity::perception::PerceivedWater,
        map::movement::{Direction, Position, Steps},
    };

    fn water(amount: usize, directions: &[Direction]) -> PerceivedWater {
        PerceivedWater {
            amount,
            steps: Steps::new(directions.to_vec()),
        }
    }

    fn life() -> curious::entity::lifestate::LifeState {
        let mut life = HerbivoreEntity::default().life_state;
        life.water = 20;
        life
    }

    #[test]
    fn current_amount_and_route_economics_choose_without_old_value() {
        let origin = Position::new(0, 0);
        let candidates = vec![
            water(10, &[Direction::Right]),
            water(1, &[Direction::Up, Direction::Up, Direction::Up]),
        ];
        let scores = score_sources(origin, &candidates, &life(), &[]);
        assert_eq!(scores.len(), 2);
        assert_eq!(choose_index(&scores), Some(0));

        // A very large but finite historical residual can still affect choice;
        // the current candidate is not rejected by a dominance filter.
        let residuals = vec![SourceResidual {
            pos: Position::new(0, -3),
            residual: 2_200,
            samples: 1,
        }];
        let adjusted = score_sources(origin, &candidates, &life(), &residuals);
        assert_eq!(adjusted.len(), 2);
        assert_eq!(choose_index(&adjusted), Some(1));
    }

    #[test]
    fn source_specific_residual_can_change_a_greedy_choice() {
        let origin = Position::new(0, 0);
        let candidates = vec![
            water(10, &[Direction::Up, Direction::Up, Direction::Up]),
            water(
                10,
                &[
                    Direction::Right,
                    Direction::Right,
                    Direction::Right,
                    Direction::Right,
                ],
            ),
        ];
        let base = score_sources(origin, &candidates, &life(), &[]);
        assert_eq!(choose_index(&base), Some(0));
        let baseline_far = base[1].baseline;

        // Outcome utility is calculated in foraging-score units; no raw legacy
        // reward is added to the current foraging score.
        let actual = actual_score(10, 1, 1).unwrap();
        assert!(actual > baseline_far);
        let mut residuals = Vec::new();
        assert!(
            record_episode_evidence(
                &mut residuals,
                EpisodeEvidence::CompletedDrink {
                    subject_id: 1,
                    event_actor_id: 1,
                    selected_target: Position::new(4, 0),
                    event_target: Position::new(4, 0),
                    water: 10,
                    movement_energy: 1,
                    elapsed_ticks: 1,
                },
                baseline_far,
            )
            .unwrap()
        );
        let adjusted = score_sources(origin, &candidates, &life(), &residuals);
        assert_eq!(choose_index(&adjusted), Some(1));
        assert_eq!(adjusted[0].residual, 0);
        assert_eq!(adjusted[1].residual, actual - baseline_far);
    }

    #[test]
    fn same_source_uses_running_mean_and_memory_stays_bounded() {
        let pos = Position::new(2, -1);
        let mut residuals = Vec::new();
        let outcome = |target, water| EpisodeEvidence::CompletedDrink {
            subject_id: 1,
            event_actor_id: 1,
            selected_target: target,
            event_target: target,
            water,
            movement_energy: 1,
            elapsed_ticks: 1,
        };
        record_episode_evidence(&mut residuals, outcome(pos, 2), 0).unwrap();
        record_episode_evidence(&mut residuals, outcome(pos, 1), 0).unwrap();
        assert_eq!(residuals.len(), 1);
        assert_eq!(residuals[0].samples, 2);
        assert_eq!(residuals[0].residual, 100);

        for x in 0..40 {
            record_episode_evidence(&mut residuals, outcome(Position::new(x, 20), x as usize), 0)
                .unwrap();
        }
        assert_eq!(residuals.len(), 32);
    }

    #[test]
    fn completed_admission_trims_oversized_tables_before_update_or_insert() {
        let oversized = || {
            (0..35)
                .map(|x| SourceResidual {
                    pos: Position::new(x, 20),
                    residual: 5,
                    samples: 1,
                })
                .collect::<Vec<_>>()
        };
        let completed = |target| EpisodeEvidence::CompletedDrink {
            subject_id: 1,
            event_actor_id: 1,
            selected_target: target,
            event_target: target,
            water: 1,
            movement_energy: 1,
            elapsed_ticks: 1,
        };

        let mut censored = oversized();
        let censored_before = censored.clone();
        assert!(!record_episode_evidence(&mut censored, EpisodeEvidence::Censored, 0,).unwrap());
        assert_eq!(censored, censored_before);

        let mut updating = oversized();
        assert!(
            record_episode_evidence(&mut updating, completed(Position::new(34, 20)), 0,).unwrap()
        );
        assert_eq!(updating.len(), 32);
        assert_eq!(updating[0].pos, Position::new(3, 20));
        assert_eq!(updating[31].pos, Position::new(34, 20));
        assert_eq!(updating[31].samples, 2);

        let mut inserting = oversized();
        let new_source = Position::new(99, 20);
        assert!(record_episode_evidence(&mut inserting, completed(new_source), 0,).unwrap());
        assert_eq!(inserting.len(), 32);
        assert_eq!(inserting[0].pos, Position::new(4, 20));
        assert_eq!(inserting[31].pos, new_source);
    }

    #[test]
    fn completed_and_censored_episode_evidence_have_distinct_admission() {
        let initial = vec![SourceResidual {
            pos: Position::new(1, 0),
            residual: 7,
            samples: 1,
        }];
        let mut censored = initial.clone();
        assert!(!record_episode_evidence(&mut censored, EpisodeEvidence::Censored, 7,).unwrap());
        assert_eq!(censored, initial);

        let mut completed = initial.clone();
        assert!(
            record_episode_evidence(
                &mut completed,
                EpisodeEvidence::CompletedDrink {
                    subject_id: 7,
                    event_actor_id: 7,
                    selected_target: Position::new(2, 0),
                    event_target: Position::new(2, 0),
                    water: 10,
                    movement_energy: 1,
                    elapsed_ticks: 1,
                },
                10,
            )
            .unwrap()
        );
        assert_eq!(completed.len(), 2);
        assert_eq!(completed[1].pos, Position::new(2, 0));

        let mut unrelated = initial.clone();
        assert!(
            !record_episode_evidence(
                &mut unrelated,
                EpisodeEvidence::CompletedDrink {
                    subject_id: 7,
                    event_actor_id: 8,
                    selected_target: Position::new(2, 0),
                    event_target: Position::new(2, 0),
                    water: 10,
                    movement_energy: 1,
                    elapsed_ticks: 1,
                },
                10,
            )
            .unwrap()
        );
        assert_eq!(unrelated, initial);

        let mut wrong_source = initial.clone();
        assert!(
            !record_episode_evidence(
                &mut wrong_source,
                EpisodeEvidence::CompletedDrink {
                    subject_id: 7,
                    event_actor_id: 7,
                    selected_target: Position::new(2, 0),
                    event_target: Position::new(3, 0),
                    water: 10,
                    movement_energy: 1,
                    elapsed_ticks: 1,
                },
                10,
            )
            .unwrap()
        );
        assert_eq!(wrong_source, initial);
    }

    #[test]
    fn out_of_contract_physical_totals_are_rejected() {
        assert_eq!(actual_score(0, 0, 0), Err(ScoreBoundsError::ZeroTicks));
        assert_eq!(actual_score(0, 0, 21), Err(ScoreBoundsError::TooManyTicks));
        assert_eq!(actual_score(101, 0, 1), Err(ScoreBoundsError::TooMuchWater));
        assert_eq!(
            actual_score(0, 1_000_001, 1),
            Err(ScoreBoundsError::MovementAccountExceeded)
        );
    }

    #[test]
    fn disabled_mode_is_exact_passthrough() {
        let scores = vec![
            super::candidate::CandidateScore {
                pos: Position::new(1, 0),
                baseline: 1,
                residual: 0,
                total: 1,
                perception_index: 0,
                path_len: 1,
            },
            super::candidate::CandidateScore {
                pos: Position::new(2, 0),
                baseline: 2,
                residual: 0,
                total: 2,
                perception_index: 1,
                path_len: 1,
            },
        ];
        // Candidate policy would select 1, but disabled dispatch returns the
        // already-computed absolute-Q legacy choice and makes no RNG call.
        assert_eq!(choose_index(&scores), Some(1));
        assert_eq!(select_index(false, Some(0), &scores), Some(0));
        assert_eq!(select_index(false, None, &scores), None);
    }
}

fn main() {
    println!("water candidate model is isolated; run with cargo test --example water_candidate");
}

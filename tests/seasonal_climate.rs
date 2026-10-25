#[path = "../examples/support/seasonal_climate_candidate.rs"]
mod seasonal_climate_candidate;

use seasonal_climate_candidate::*;

#[test]
fn test_season_cycle_deterministic_progression() {
    // 40 ticks per year => exactly 10 ticks per season
    let cycle = SeasonCycle::new(40);
    let engine = SeasonalClimateEngine::new(40);

    // Year 1
    assert_eq!(cycle.season_at(0), Season::Spring);
    assert!(!cycle.season_at(0).is_winter());
    assert_eq!(engine.current_season(0), Season::Spring);
    assert_eq!(cycle.season_at(9), Season::Spring);

    assert_eq!(cycle.season_at(10), Season::Summer);
    assert_eq!(cycle.season_at(19), Season::Summer);

    assert_eq!(cycle.season_at(20), Season::Autumn);
    assert_eq!(cycle.season_at(29), Season::Autumn);

    assert_eq!(cycle.season_at(30), Season::Winter);
    assert!(cycle.season_at(30).is_winter());
    assert_eq!(engine.current_season(30), Season::Winter);
    assert_eq!(cycle.season_at(39), Season::Winter);

    // Year 2 deterministic wrap-around
    assert_eq!(cycle.season_at(40), Season::Spring);
    assert_eq!(cycle.season_at(50), Season::Summer);
    assert_eq!(cycle.season_at(60), Season::Autumn);
    assert_eq!(cycle.season_at(70), Season::Winter);
}

#[test]
fn test_regional_latitude_temperature_and_surcharges() {
    let climate = ClimateConfig::default();

    // In Spring, Summer, Autumn: 0 surcharge regardless of latitude
    for season in [Season::Spring, Season::Summer, Season::Autumn] {
        assert_eq!(climate.thermoregulation_surcharge(season, 0), 0);
        assert_eq!(climate.thermoregulation_surcharge(season, 3), 0);
        assert_eq!(climate.thermoregulation_surcharge(season, 7), 0);
        assert_eq!(climate.thermoregulation_surcharge(season, -8), 0);
    }

    // In Winter:
    // Temperate latitude (|y| <= 4): mild surcharge = +1 energy/tick
    assert_eq!(climate.thermoregulation_surcharge(Season::Winter, 0), 1);
    assert_eq!(climate.thermoregulation_surcharge(Season::Winter, 3), 1);
    assert_eq!(climate.thermoregulation_surcharge(Season::Winter, -4), 1);

    // Frigid latitude (|y| > 4): severe surcharge = +2 energy/tick
    assert_eq!(climate.thermoregulation_surcharge(Season::Winter, 5), 2);
    assert_eq!(climate.thermoregulation_surcharge(Season::Winter, -6), 2);
    assert_eq!(climate.thermoregulation_surcharge(Season::Winter, 12), 2);
}

#[test]
fn test_winter_resource_replenishment_scaling() {
    let climate = ClimateConfig {
        winter_replenish_per_mille: 500, // 50% replenishment in Winter
        ..Default::default()
    };

    let base_food = 10;
    let base_water = 8;

    // Normal seasons retain 100% replenishment
    assert_eq!(climate.scale_replenishment(Season::Spring, base_food), 10);
    assert_eq!(climate.scale_replenishment(Season::Summer, base_water), 8);
    assert_eq!(climate.scale_replenishment(Season::Autumn, base_food), 10);

    // Winter scales replenishment down
    assert_eq!(
        climate.scale_replenishment(Season::Winter, base_food),
        5,
        "Winter replenishment must be halved (500 per mille)"
    );
    assert_eq!(
        climate.scale_replenishment(Season::Winter, base_water),
        4,
        "Winter replenishment must be halved (500 per mille)"
    );

    // More severe winter config (250 per mille = 25%)
    let severe_climate = ClimateConfig {
        winter_replenish_per_mille: 250,
        ..Default::default()
    };
    assert_eq!(
        severe_climate.scale_replenishment(Season::Winter, base_food),
        2
    );
}

#[test]
fn test_hibernation_entry_eligibility_and_rejection() {
    let engine = SeasonalClimateEngine::new(40);
    let mut state = HibernationState::Active;

    // 1. Outside burrow cannot hibernate
    let outside = engine.try_enter_hibernation(&mut state, false, Season::Winter, 80, 50);
    assert!(!outside);
    assert_eq!(state, HibernationState::Active);

    // 2. In non-winter season cannot hibernate
    let spring = engine.try_enter_hibernation(&mut state, true, Season::Spring, 80, 50);
    assert!(!spring);
    assert_eq!(state, HibernationState::Active);

    // 3. Insufficient energy (<50) cannot hibernate
    let low_energy = engine.try_enter_hibernation(&mut state, true, Season::Winter, 45, 50);
    assert!(!low_energy);
    assert_eq!(state, HibernationState::Active);

    // 4. Insufficient water (<30) cannot hibernate
    let low_water = engine.try_enter_hibernation(&mut state, true, Season::Winter, 80, 25);
    assert!(!low_water);
    assert_eq!(state, HibernationState::Active);

    // 5. Valid entry: Winter, in Burrow, energy >= 50, water >= 30
    let success = engine.try_enter_hibernation(&mut state, true, Season::Winter, 80, 50);
    assert!(success);
    assert!(state.is_hibernating());
    assert_eq!(state, HibernationState::Hibernating { ticks_hibernated: 0 });
}

#[test]
fn test_hibernation_metabolic_conservation_and_cold_immunity() {
    let engine = SeasonalClimateEngine::new(40);
    let climate = &engine.climate_config;

    // Simulate 20 ticks of severe winter at latitude y = 8 (surcharge = +2)
    let y = 8;
    let surcharge = climate.thermoregulation_surcharge(Season::Winter, y);
    assert_eq!(surcharge, 2);

    // Case A: Active individual
    // Active BMR: 1 energy + 1 water per tick, plus +2 energy surcharge = 3 energy/tick, 1 water/tick
    let mut active_energy: usize = 80;
    let mut active_water: usize = 80;
    for _ in 0..20 {
        active_energy = active_energy.saturating_sub(1 + surcharge);
        active_water = active_water.saturating_sub(1);
    }
    assert_eq!(active_energy, 20, "Active individual lost 60 energy in 20 ticks");
    assert_eq!(active_water, 60, "Active individual lost 20 water in 20 ticks");

    // Case B: Hibernating individual in Burrow
    let mut hib_state = HibernationState::Active;
    let mut hib_energy = 80;
    let mut hib_water = 80;
    assert!(engine.try_enter_hibernation(&mut hib_state, true, Season::Winter, hib_energy, hib_water));

    for _ in 0..20 {
        // Hibernating individuals face 0 cold surcharge and only drain 1 every 4 ticks
        let outcome = engine.resolve_hibernation_tick(
            &mut hib_state,
            Season::Winter,
            true, // in burrow
            &mut hib_energy,
            &mut hib_water,
            0, // no damage
        );
        assert!(matches!(outcome, HibernationOutcome::Maintained { .. }));
    }

    // Over 20 ticks with metabolic interval = 4: exactly 20 / 4 = 5 drains
    assert_eq!(
        hib_energy, 75,
        "Hibernating individual conserved energy, losing only 5 energy vs 60"
    );
    assert_eq!(
        hib_water, 75,
        "Hibernating individual conserved water, losing only 5 water vs 20"
    );
    assert_eq!(
        hib_state,
        HibernationState::Hibernating { ticks_hibernated: 20 }
    );
}

#[test]
fn test_hibernation_wake_triggers() {
    let engine = SeasonalClimateEngine::new(40);

    // Trigger 1: Attacked (takes damage > 0)
    {
        let mut state = HibernationState::Hibernating { ticks_hibernated: 5 };
        let mut energy = 60;
        let mut water = 50;
        let outcome = engine.resolve_hibernation_tick(
            &mut state,
            Season::Winter,
            true,
            &mut energy,
            &mut water,
            15, // took 15 combat damage!
        );
        assert!(matches!(outcome, HibernationOutcome::WokeUp(WakeReason::Attacked)));
        assert_eq!(state, HibernationState::Active);
    }

    // Trigger 2: Season transitions away from Winter (Spring arrives)
    {
        let mut state = HibernationState::Hibernating { ticks_hibernated: 12 };
        let mut energy = 60;
        let mut water = 50;
        let outcome = engine.resolve_hibernation_tick(
            &mut state,
            Season::Spring, // Season changed!
            true,
            &mut energy,
            &mut water,
            0,
        );
        assert!(matches!(outcome, HibernationOutcome::WokeUp(WakeReason::SeasonEnded)));
        assert_eq!(state, HibernationState::Active);
    }

    // Trigger 3: Energy drops to critical threshold (<= 15)
    {
        let mut state = HibernationState::Hibernating { ticks_hibernated: 25 };
        let mut energy = 15; // Critical energy threshold!
        let mut water = 50;
        let outcome = engine.resolve_hibernation_tick(
            &mut state,
            Season::Winter,
            true,
            &mut energy,
            &mut water,
            0,
        );
        assert!(matches!(outcome, HibernationOutcome::WokeUp(WakeReason::CriticalReserves)));
        assert_eq!(state, HibernationState::Active);
    }

    // Trigger 4: Exited burrow
    {
        let mut state = HibernationState::Hibernating { ticks_hibernated: 8 };
        let mut energy = 60;
        let mut water = 50;
        let outcome = engine.resolve_hibernation_tick(
            &mut state,
            Season::Winter,
            false, // no longer in burrow
            &mut energy,
            &mut water,
            0,
        );
        assert!(matches!(outcome, HibernationOutcome::WokeUp(WakeReason::ExitedBurrow)));
        assert_eq!(state, HibernationState::Active);
    }
}

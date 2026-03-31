use curious::{
    creatures::carnivore::CarnivoreEntity,
    creatures::herbivore::HerbivoreEntity,
    creatures::omnivore::OmnivoreEntity,
    entity::{perception::Perception, phase::EntityPhase},
    map::movement::Position,
    set_global_seed_with_time,
    world::{EntitySlot, World},
};
use std::{thread, time::Duration};

fn main() {
    // RNG için seed'i zaman damgası olarak günceller
    set_global_seed_with_time();
    let entities: Vec<EntitySlot> = vec![
        EntitySlot::new(
            1,
            (-15isize, -15isize).into(),
            EntityPhase::Active,
            curious::entity::controller::EntityController::Player(0),
            Box::new(HerbivoreEntity::default()),
        ),
        EntitySlot::new(
            2,
            (-14isize, -15isize).into(),
            EntityPhase::Active,
            curious::entity::controller::EntityController::AI,
            Box::new(HerbivoreEntity::default()),
        ),
        EntitySlot::new(
            3,
            (14isize, -15isize).into(),
            EntityPhase::Active,
            curious::entity::controller::EntityController::AI,
            Box::new(CarnivoreEntity::default()),
        ),
        EntitySlot::new(
            4,
            (15isize, -15isize).into(),
            EntityPhase::Active,
            curious::entity::controller::EntityController::AI,
            Box::new(OmnivoreEntity::default()),
        ),
        EntitySlot::new(
            5,
            (-15isize, 15isize).into(),
            EntityPhase::Active,
            curious::entity::controller::EntityController::AI,
            Box::new(CarnivoreEntity::default()),
        ),
        EntitySlot::new(
            6,
            (-14isize, 15isize).into(),
            EntityPhase::Active,
            curious::entity::controller::EntityController::AI,
            Box::new(OmnivoreEntity::default()),
        ),
    ];
    // İnteraktif dünya
    let mut engine = curious::simulation::SimulationEngine::new(World::new(-15, 14, -15, 14, entities));
    // İnteraktif dünya sayacı
    let mut tick_counter: usize = 0;
    loop {
        // 1. Simülasyon adımı işlet ve olayları topla
        let events = engine.step();

        // 2. Sayacı simülasyonun içinden al
        let current_tick = engine.world().tick_counter;

        // Örnek: 50. tick'te oyuncu kontrollü birime dışarıdan komut gönder
        if current_tick == 50 {
            use curious::entity::intent::Intent;
            use curious::map::movement::{Steps, Direction};
            engine.send_command(1, Intent::Move { steps: Steps::from(vec![Direction::Up, Direction::Up]) });
        }

        // 3. Ekranı temizle ve haritayı çiz
        print!("\x1B[2J\x1B[1;1H\n");
        print_map(engine.world(), current_tick, &events);
        
        thread::sleep(Duration::from_millis(300));
    }
}

pub fn print_map(world: &World, tick: usize, events: &[curious::simulation::SimulationEvent]) {
    let map_width = world.map.map_width();
    let map_height = world.map.map_height();

    println!(
        "\x1b[1m=== SIMULATION | Map: ({}x{}) | Tick: {} ===\x1b[0m",
        map_width, map_height, tick
    );
    println!("{:-<1$}", "", map_width * 2 + 50);

    for y in world.map.min_y..=world.map.max_y {
        // --- SOL KOLON: HARİTA ---
        for x in world.map.min_x..=world.map.max_x {
            let pos = (x, y).into();

            if let Some(slot) = world.entities.iter().find(|e| e.pos == pos) {
                match slot.phase {
                    EntityPhase::Active => {
                        let (r, g, b) = match slot.base.species() {
                            curious::entity::species::Species::Carnivore => (220, 40, 40),
                            curious::entity::species::Species::Herbivore => (40, 200, 40),
                            curious::entity::species::Species::Omnivore => (60, 120, 220),
                        };
                        print!("\x1b[38;2;{};{};{}m@ \x1b[0m", r, g, b);
                    }
                    EntityPhase::Corpse { .. } => {
                        print!("\x1b[38;2;255;140;0mX \x1b[0m");
                    }
                    _ => print!(". "),
                }
            } else if let Some(curious::map::cell::Cell::Food { .. }) = world.map.cell(pos) {
                print!("\x1b[38;2;240;220;0mf \x1b[0m");
            } else if let Some(curious::map::cell::Cell::Water { .. }) = world.map.cell(pos) {
                print!("\x1b[38;2;40;180;255mw \x1b[0m"); // Su artık mavi
            } else {
                print!(". ");
            }
        }

        // --- ORTA KOLON: CANLI DURUMLARI ---
        let entity_index = (y - world.map.min_y) as usize;
        if let Some(slot) = world.entities.get(entity_index) {
            let life = slot.entity().life();
            print!(
                "  | @{:<2} {:<10?} HP:{:<3} EN:{:<3} Ph:{:?}",
                slot.id,
                slot.base.species(),
                life.health,
                life.energy,
                slot.phase
            );
        }

        // --- SAĞ KOLON: SON OLAYLAR (EVENTS) ---
        if let Some(event) = events.get(entity_index) {
            print!("  | [EVENT] {:?}", event);
        }

        println!();
    }
    println!("{:-<1$}", "", map_width * 2 + 50);
    println!("@: Canlı | X: Ceset | f: Yemek | w: Su");
}

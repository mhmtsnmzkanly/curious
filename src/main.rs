use curious::{
    creatures::herbivore::HerbivoreEntity,
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
            (0isize, 0isize).into(),
            EntityPhase::Active,
            Box::new(HerbivoreEntity::default()),
        ),
        EntitySlot::new(
            2,
            (3isize, 3isize).into(),
            EntityPhase::Active,
            Box::new(HerbivoreEntity::default()),
        ),
    ];
    // İnteraktif dünya
    let mut world = World::new(-8, 7, -8, 7, entities);
    // İnteraktif dünya sayacı
    let mut tick_counter: usize = 0;
    loop {
        print!("\x1B[2J\x1B[1;1H\n");
        world.tick();
        tick_counter += 1;
        print_map(&world, tick_counter);
        thread::sleep(Duration::from_millis(1000));
    }
}

pub fn print_map(world: &World, tick: usize) {
    let map_width = world.map.map_width();
    let map_height = world.map.map_height();

    println!(
        "=== SIMULATION | Map: ({}x{})  | Tick: {} ===",
        map_width, map_height, tick
    );
    println!("{:-<1$}", "", map_width * 5);

    for y in world.map.min_y..=world.map.max_y {
        // --- SOL KOLON: HARİTA ---
        for x in world.map.min_x..=world.map.max_x {
            let pos = (x, y).into();

            // Hücredeki varlığı kontrol et (Öncelik: Canlı > Ceset > Yemek)
            if let Some(slot) = world.entities.iter().find(|e| e.pos == pos) {
                match slot.phase {
                    // ANSI TrueColor formatı: \x1b[38;2;R;G;Bm
                    // \x1b[0m kodu ise rengi sıfırlamak içindir
                    EntityPhase::Active => print!(
                        "\x1b[38;2;{};{};{}m@ \x1b[0m",
                        (slot.id & 0xFF) as u8,
                        ((slot.id >> 8) & 0xFF) as u8,
                        ((slot.id >> 16) & 0xFF) as u8
                    ), // Canlı
                    EntityPhase::Corpse { .. } => print!("X "), // Ceset
                    _ => print!("? "),
                }
            } else if let Some(curious::map::cell::Cell::Food { .. }) = world.map.cell(pos) {
                print!("f "); // Yemek (Food)
            } else {
                print!(". "); // Boş hücre
            }
        }

        // --- SAĞ KOLON: CANLI DURUMLARI ---
        // Sadece haritanın ilk birkaç satırında canlı bilgilerini yazdır
        let entity_index = (y - world.map.min_y) as usize;
        if let Some(slot) = world.entities.get(entity_index) {
            let life = slot.entity().life();
            print!(
                "  | @{:<2} {:?} HP:{:<3} EN:{:<3} AGE:{:<3} Ph:{:?} ",
                slot.id, slot.pos, life.health, life.energy, life.age, slot.phase
            );
        }

        println!(); // Alt satıra geç
    }
    println!("{:-<1$}", "", map_width + 5);
    println!("@: Canlı | X: Ceset | f: Yemek");
}

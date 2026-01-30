use curious::{
    creatures::herbivore::Herbivore,
    entity::Entity,
    map::{Map, cell::Cell, position::Position},
    world::World,
};
use std::{thread, time::Duration};

fn main() {
    let width = 10;
    let height = 10;
    let mut grid = vec![Cell::Empty; width * height];

    // Haritaya yemekler koy
    grid[3 * width + 3] = Cell::Food { amount: 100 };
    grid[6 * width + 8] = Cell::Food { amount: 100 };

    let mut world = World::new(
        Map {
            width,
            height,
            grid,
        },
        vec![
            Box::new(Herbivore::new(1, Position::new(0, 0))),
            Box::new(Herbivore::new(2, Position::new(9, 9))),
        ],
    );

    let mut tick_counter: usize = 0;
    loop {
        tick_counter += 1;
        print_map(&world, tick_counter);
        world.tick();
        thread::sleep(Duration::from_millis(600));
    }
}

fn print_map(world: &World, tick: usize) {
    // Terminali temizle ve imleci başa al (Daha akıcı bir görünüm sağlar)
    print!("\x1B[2J\x1B[1;1H");

    println!("====================================================");
    println!("   CURIOUS SIMULATION - TICK: {:<5}", tick);
    println!("====================================================");

    // --- CANLI DURUMLARI (DASHBOARD) ---
    println!(
        "{:<4} | {:<8} | {:<6} | {:<6} | {:<4} | {:<10}",
        "ID", "POS", "ENRG", "HLTH", "AGE", "PHASE"
    );
    println!("----------------------------------------------------");

    for e in world.entities.iter() {
        let l = e.life();
        let pos = e.position();

        // Enerji düşükse kırmızı, değilse yeşil renkle yazdırabiliriz (Opsiyonel ANSI)
        let energy_status = if l.is_energy_low() { "!" } else { " " };

        println!(
            "{:<4} | ({:>2},{:>2}) | {:>4}{} | {:>6} | {:>4} | {:?}",
            e.id(),
            pos.x,
            pos.y,
            l.energy,
            energy_status,
            l.health,
            l.age,
            e.phase()
        );
    }

    println!("----------------------------------------------------");

    // --- HARİTA ÇİZİMİ ---
    println!("\nMAP:");
    for y in 0..world.map.height {
        print!("  "); // Sol boşluk
        for x in 0..world.map.width {
            let pos = Position::new(x, y);

            // Bu hücrede bir canlı var mı? (Sadece canlı olanları göster)
            let ent = world
                .entities
                .iter()
                .find(|e| e.position() == pos && e.life().is_alive());

            if let Some(e) = ent {
                // Canlıyı ID'si ile göster (Örn: @1)
                print!("\x1B[92m@{}\x1B[0m ", e.id()); // Parlak Yeşil
            } else {
                match world.map.cell(pos) {
                    Some(Cell::Food { .. }) => print!("\x1B[93mF  \x1B[0m"), // Sarı F
                    Some(Cell::Water { .. }) => print!("\x1B[94mW  \x1B[0m"), // Mavi W
                    _ => print!(".  "),                                      // Boş hücre
                }
            }
        }
        println!();
    }
    println!("\n====================================================");
}

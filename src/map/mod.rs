pub mod cell;
pub mod movement;

use std::collections::{HashMap, VecDeque};

use crate::{
    CHUNK_SIZE, gen_range,
    map::{
        cell::Cell,
        movement::{DIRECTION_ARRAY, Direction, Position, Steps},
    },
    next_rand,
};

#[derive(Debug)]
struct Chunk {
    cells: Vec<Cell>,
}

impl Chunk {
    fn new() -> Self {
        Self {
            cells: vec![Cell::Empty; CHUNK_SIZE * CHUNK_SIZE],
        }
    }

    /// Hücre indexi oluştur
    #[inline]
    fn idx(x: usize, y: usize) -> usize {
        y * CHUNK_SIZE + x
    }

    /// Hücreyi oku
    fn cell(&self, x: usize, y: usize) -> &Cell {
        &self.cells[Self::idx(x, y)]
    }

    /// Hücreyi değiştir
    fn cell_mut(&mut self, x: usize, y: usize) -> &mut Cell {
        &mut self.cells[Self::idx(x, y)]
    }

    /// Hücre tamamen boşalmış mı?
    fn is_completely_empty(&self) -> bool {
        self.cells.iter().all(|c| matches!(c, Cell::Empty))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ChunkCoord {
    cx: isize,
    cy: isize,
}

#[derive(Debug)]
pub struct Map {
    /// Yatay eksende sağ kısım
    pub min_x: isize,
    /// Yatay eksende sol kısım
    pub max_x: isize,
    /// Dikey eksende sağ kısım
    pub min_y: isize,
    /// Dikey eksende sol kısım
    pub max_y: isize,
    /// Parçalara ayrılmış harita.
    chunks: HashMap<ChunkCoord, Chunk>,
}

impl Map {
    /// Sınırları kontrol ederek güvenli bir dünya oluşturur
    pub fn new(x1: isize, x2: isize, y1: isize, y2: isize) -> Self {
        // Kullanıcı değerleri ters girse bile (min/max) doğru eşleştirilir
        let (min_x, max_x) = if x1 <= x2 { (x1, x2) } else { (x2, x1) };
        let (min_y, max_y) = if y1 <= y2 { (y1, y2) } else { (y2, y1) };

        Self {
            min_x,
            max_x,
            min_y,
            max_y,
            chunks: HashMap::new(),
        }
    }

    /// Bir dünya pozisyonunun hangi chunk koordinatına denk geldiğini döndürür
    pub fn chunk_coord(pos: Position) -> ChunkCoord {
        ChunkCoord {
            cx: pos.x.div_euclid(CHUNK_SIZE as isize),
            cy: pos.y.div_euclid(CHUNK_SIZE as isize),
        }
    }

    fn local_coord(pos: Position) -> (usize, usize) {
        (
            pos.x.rem_euclid(CHUNK_SIZE as isize) as usize,
            pos.y.rem_euclid(CHUNK_SIZE as isize) as usize,
        )
    }

    fn index_of(&self, pos: Position) -> (ChunkCoord, usize, usize) {
        let cc = Self::chunk_coord(pos);
        let (lx, ly) = Self::local_coord(pos);
        (cc, lx, ly)
    }

    pub fn in_bounds(&self, pos: Position) -> bool {
        pos.x >= self.min_x && pos.x <= self.max_x && pos.y >= self.min_y && pos.y <= self.max_y
    }

    pub fn cell(&self, pos: Position) -> Option<&Cell> {
        if !self.in_bounds(pos) {
            return None;
        }
        let (cc, lx, ly) = self.index_of(pos);
        self.chunks.get(&cc).map(|c| c.cell(lx, ly))
    }

    pub fn is_cell(&self, pos: Position, expected: &Cell) -> bool {
        self.cell(pos).map(|c| c == expected).unwrap_or(false)
    }

    pub fn is_walkable(&self, pos: Position) -> bool {
        matches!(
            self.cell(pos),
            Some(Cell::Empty | Cell::Food { .. } | Cell::Water { .. })
        )
    }

    pub fn set_cell(&mut self, pos: Position, cell: Cell) {
        // 1. Adım: Dünya sınırları kontrolü
        if !self.in_bounds(pos) {
            return;
        }

        let (cc, lx, ly) = self.index_of(pos);

        // 2. Adım: Eğer hücre boşsa ve chunk yoksa, boş bir hücre için yeni chunk yaratma.
        if cell == Cell::Empty && !self.chunks.contains_key(&cc) {
            return;
        }

        // 3. Adım: Chunk'ı al veya oluştur, ardından hücreyi yaz
        let chunk = self.chunks.entry(cc).or_insert_with(Chunk::new);
        *chunk.cell_mut(lx, ly) = cell;
    }

    pub fn reduce_cell_amount(&mut self, pos: Position, amount: usize) -> bool {
        if !self.in_bounds(pos) {
            return false;
        }

        let (cc, lx, ly) = self.index_of(pos);

        let should_remove = {
            let chunk = match self.chunks.get_mut(&cc) {
                Some(c) => c,
                None => return false,
            };

            match chunk.cell_mut(lx, ly) {
                Cell::Food { amount: a } | Cell::Water { amount: a } => {
                    *a = a.saturating_sub(amount);
                    if *a == 0 {
                        *chunk.cell_mut(lx, ly) = Cell::Empty;
                    }
                }
                _ => return false,
            }
            // Hücre boşaldıktan sonra chunk'ın durumunu kontrol et
            chunk.is_completely_empty()
        };

        if should_remove {
            self.chunks.remove(&cc);
        }
        true
    }

    pub fn clear_cell(&mut self, pos: Position) {
        self.set_cell(pos, Cell::Empty);
    }

    /// Hücreye yiyecek ekle (varsa miktarı artır)
    pub fn add_food(&mut self, pos: Position, amount: usize) {
        if !self.in_bounds(pos) {
            return;
        }

        let new_cell = match self.cell(pos) {
            Some(Cell::Food { amount: a }) => Cell::Food { amount: a + amount },
            _ => Cell::Food { amount },
        };
        self.set_cell(pos, new_cell);
    }

    /// Bir yönde engel gelene kadar kaç adım?
    pub fn walkable_distance(&self, from: Position, dir: Direction) -> u8 {
        let mut cur = from;
        let mut steps = 0u8;

        loop {
            let next = cur + dir;
            if !self.is_walkable(next) {
                break;
            }
            steps += 1;
            cur = next;
            if steps == u8::MAX {
                break;
            }
        }
        steps
    }

    pub fn walkable_distances(&self, from: Position) -> HashMap<Direction, u8> {
        let mut map = HashMap::new();
        for d in DIRECTION_ARRAY {
            map.insert(d, self.walkable_distance(from, d));
        }
        map
    }

    /// Radius ile sınırlı BFS
    pub fn bfs_steps_to(&self, start: Position, goal: Position, radius: usize) -> Option<Steps> {
        if !self.is_walkable(goal) {
            return None;
        }

        let mut queue = VecDeque::new();
        let mut came_from: HashMap<Position, (Position, Direction)> = HashMap::new();

        queue.push_back(start);

        while let Some(current) = queue.pop_front() {
            if current == goal {
                break;
            }

            let dist = (current.x - start.x).abs() + (current.y - start.y).abs();
            if dist as usize >= radius {
                continue;
            }
            for dir in [
                Direction::Up,
                Direction::Down,
                Direction::Left,
                Direction::Right,
                Direction::UpLeft,
                Direction::UpRight,
                Direction::DownLeft,
                Direction::DownRight,
            ] {
                let next = current + dir;
                if !self.is_walkable(next) {
                    continue;
                }
                if came_from.contains_key(&next) || next == start {
                    continue;
                }

                came_from.insert(next, (current, dir));
                queue.push_back(next);
            }
        }

        // Path reconstruction
        let mut steps = Vec::new();
        let mut cur = goal;

        while cur != start {
            let (prev, dir) = *came_from.get(&cur)?;
            steps.push(dir);
            cur = prev;
        }

        steps.reverse();
        Some(Steps(steps))
    }

    pub fn scan_foods_within(
        &self,
        center: Position,
        radius: usize,
    ) -> Vec<(Position, Steps, usize)> {
        let mut result = Vec::new();

        for y in (center.y - radius as isize)..=(center.y + radius as isize) {
            for x in (center.x - radius as isize)..=(center.x + radius as isize) {
                let pos = Position { x, y };

                if !self.in_bounds(pos) {
                    continue;
                }

                let manhattan = (center.x - x).abs() + (center.y - y).abs();
                if manhattan as usize > radius {
                    continue;
                }

                if let Some(Cell::Food { amount }) = self.cell(pos) {
                    if let Some(steps) = self.bfs_steps_to(center, pos, radius) {
                        result.push((pos, steps, *amount));
                    }
                }
            }
        }

        result
    }

    pub fn scan_waters_within(
        &self,
        center: Position,
        radius: usize,
    ) -> Vec<(Position, Steps, usize)> {
        let mut result = Vec::new();

        for y in (center.y - radius as isize)..=(center.y + radius as isize) {
            for x in (center.x - radius as isize)..=(center.x + radius as isize) {
                let pos = Position { x, y };

                if !self.in_bounds(pos) {
                    continue;
                }

                let manhattan = (center.x - x).abs() + (center.y - y).abs();
                if manhattan as usize > radius {
                    continue;
                }

                if let Some(Cell::Water { amount }) = self.cell(pos) {
                    if let Some(steps) = self.bfs_steps_to(center, pos, radius) {
                        result.push((pos, steps, *amount));
                    }
                }
            }
        }

        result
    }

    /// Tüm haritayı chunk chunk doldurur (Orkestra Şefi)
    pub fn populate_resources(&mut self, density: f32) {
        // Haritanın kapsadığı chunk sınırlarını hesapla
        // Negatif koordinatlar için div_euclid kullanılmalı,
        // aksi halde değerler 0'a doğru yuvarlandığı için yanlış chunklar seçilir.
        let min_cx = self.min_x.div_euclid(CHUNK_SIZE as isize);
        let max_cx = self.max_x.div_euclid(CHUNK_SIZE as isize);
        let min_cy = self.min_y.div_euclid(CHUNK_SIZE as isize);
        let max_cy = self.max_y.div_euclid(CHUNK_SIZE as isize);

        for cx in min_cx..=max_cx {
            for cy in min_cy..=max_cy {
                self.populate_chunk(ChunkCoord { cx, cy }, density);
            }
        }
    }

    /// Sadece belirli bir chunk içine odaklanır (Uzman)
    pub fn populate_chunk(&mut self, coord: ChunkCoord, density: f32) {
        let start_x = coord.cx * CHUNK_SIZE as isize;
        let start_y = coord.cy * CHUNK_SIZE as isize;

        let spawn_threshold = (density.clamp(0.0, 1.0) * 100.0).round() as u64;

        for ly in 0..CHUNK_SIZE {
            for lx in 0..CHUNK_SIZE {
                let world_pos = Position::new(start_x + lx as isize, start_y + ly as isize);

                if !self.in_bounds(world_pos)
                    || !self
                        .cell(world_pos)
                        .map_or(true, |c| matches!(c, Cell::Empty))
                {
                    continue;
                }

                let roll = next_rand() % 100;
                if roll >= spawn_threshold {
                    continue;
                }

                let amount = (next_rand() % 7 + 5) as usize;
                let water_roll = next_rand() % 100;
                if water_roll < 20 {
                    self.set_cell(world_pos, Cell::Water { amount });
                } else {
                    self.set_cell(world_pos, Cell::Food { amount });
                }
            }
        }
    }

    pub fn map_width(&self) -> usize {
        (self.max_x - self.min_x + 1) as usize
    }

    pub fn map_height(&self) -> usize {
        (self.max_y - self.min_y + 1) as usize
    }
}

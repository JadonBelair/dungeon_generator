use macroquad::{math::I16Vec2, rand::{gen_range, ChooseRandom}};

const DIRECTIONS: [I16Vec2; 4] = [
    I16Vec2::X,
    I16Vec2::Y,
    I16Vec2::NEG_X,
    I16Vec2::NEG_Y,
];

const DUNGEON_WIDTH: usize = 64;
const DUNGEON_HEIGHT: usize = 36;

const MAX_ROOM_SIZE: usize = 11;

const ROOM_ATTEMPTS: u16 = 600;
const WINDING_CHANCE: u8 = 50;

pub struct Generator {
    pub dungeon_width: usize,
    pub dungeon_height: usize,
    pub max_room_size: usize,
    pub room_attempts: u16,
    pub winding_chance: u8,
}

impl Generator {
    pub fn new() -> Self {
        Self {
            dungeon_width: DUNGEON_WIDTH,
            dungeon_height: DUNGEON_HEIGHT,
            max_room_size: MAX_ROOM_SIZE,
            room_attempts: ROOM_ATTEMPTS,
            winding_chance: WINDING_CHANCE,
        }
    }

    pub fn generate(&self) -> Vec<Vec<u16>> {
        let mut map = vec![vec![0; self.dungeon_width]; self.dungeon_height];
        let final_room = self.fill_rooms(&mut map) - 1;
        let mut id = final_room + 1;

        for y in (1..self.dungeon_height-1).step_by(2) {
            for x in (1..self.dungeon_width-1).step_by(2) {
                if map[y][x] != 0 {
                    continue;
                }

                self.make_maze(&mut map, &I16Vec2::new(x as i16, y as i16), id);
                id += 1;
            }
        }

        self.connect_regions(&mut map, final_room);
        self.remove_dead_ends(&mut map);

        return map;
    }

    fn fill_rooms(&self, map: &mut Vec<Vec<u16>>) -> u16 {
        let mut attemps = 0;
        let mut id = 1;
        let mut rooms = Vec::new();

        while attemps < self.room_attempts {
            attemps += 1;
            let room_width = gen_range(1, (self.max_room_size-1)/2)*2+1;
            let room_height = gen_range(1, (self.max_room_size-1)/2)*2+1;
            let room_x = gen_range(1, (self.dungeon_width-room_width-1)/2)*2+1;
            let room_y = gen_range(1, (self.dungeon_height-room_height-1)/2)*2+1;

            let new_room = URect::new(room_x, room_y, room_width, room_height);

            let mut overlaps = false;
            for room in &rooms {
                if new_room.overlaps(room) {
                    overlaps = true;
                    break;
                }
            }

            if !overlaps {
                for y in room_y..(room_y+room_height) {
                    for x in room_x..(room_x+room_width) {
                        let pos = I16Vec2::new(x as i16, y as i16);
                        self.carve(map, &pos, id);
                    }
                }

                id += 1;
                rooms.push(new_room);
            }
        }
        return id;
    }

    fn make_maze(&self, map: &mut Vec<Vec<u16>>, start: &I16Vec2, id: u16) {
        let mut last_dir = I16Vec2::X;
        let mut cells = vec![*start];
        self.carve(map, start, id);

        while !cells.is_empty() {
            let cell = cells.last().unwrap();

            let mut unmade_cells: Vec<I16Vec2> = Vec::new();

            for dir in &DIRECTIONS {
                if self.can_carve(&map, cell, dir) {
                    unmade_cells.push(*dir);
                }
            }

            if !unmade_cells.is_empty() {
                let dir = if unmade_cells.contains(&last_dir) && gen_range(0, 100) > self.winding_chance {
                    last_dir
                } else {
                    *unmade_cells.choose().unwrap()
                };

                self.carve(map, &(*cell + dir), id);
                self.carve(map, &(*cell + dir*2), id);

                cells.push(*cell + dir*2);
                last_dir = dir;
            } else {
                cells.pop();
            }
        }
    }

    fn connect_regions(&self, map: &mut Vec<Vec<u16>>, final_room: u16) {
        let mut connectors = Vec::new();
        let mut main = Vec::new();

        for y in 0..self.dungeon_height{
            for x in 0..self.dungeon_width{
                let pos = I16Vec2::new(x as i16, y as i16);
                if self.is_connector(&map, &pos) {
                    connectors.push(pos);
                }
            }
        }

        main.push(gen_range(1, final_room+1));

        while !connectors.is_empty() {
            let mut current_index = -1;
            for i in 0..connectors.len() {
                let pos = &connectors[i];
                let connector_regions = self.get_connector_regions(&map, pos);
                if main.contains(&connector_regions[0]) || main.contains(&connector_regions[1]) {
                    current_index = i as i16;
                    break;
                }
            }

            let current_connector = connectors[current_index as usize];
            let connector_regions = self.get_connector_regions(&map, &current_connector);

            let new_region = if main.contains(&connector_regions[0]) {
                connector_regions[1]
            } else {
                connector_regions[0]
            };

            self.carve(map, &current_connector, new_region);
            connectors.remove(current_index as usize);
            main.push(new_region);

            for i in (0..connectors.len()).rev() {
                let test = connectors[i];
                let test_regions = self.get_connector_regions(&map, &test);

                if test_regions == connector_regions {
                    let punch = gen_range(0.0f32, 1.0f32) > 0.9;
                    if punch {
                        self.carve(map, &test, new_region);
                    }
                    connectors.remove(i);
                }
            }
        }
    }

    fn remove_dead_ends(&self, map: &mut Vec<Vec<u16>>) {
        let mut done_removing = false;
        while !done_removing {
            let mut possible_pos = Vec::new();
            for y in 1..(map.len()-1) {
                for x in 1..(map[y].len()-1) {
                    if map[y][x] == 0 {
                        continue;
                    }

                    let mut exits = 0;
                    for dir in DIRECTIONS {
                        if map[(y as i16 + dir.y) as usize][(x as i16 + dir.x) as usize] != 0 {
                            exits += 1;
                        }
                    }

                    if exits != 1 {
                        continue;
                    }

                    possible_pos.push(I16Vec2::new(x as i16, y as i16));
                }
            }

            if !possible_pos.is_empty() {
                let pos = possible_pos.choose().unwrap();
                self.carve(map, pos, 0);
            } else {
                done_removing = true;
            }
        }
    }

    fn is_connector(&self, map: &Vec<Vec<u16>>, cell: &I16Vec2) -> bool {
        let cell_x = cell.x as usize;
        let cell_y = cell.y as usize;
        if map[cell_y][cell_x] != 0 {
            return false;
        }

        let mut regions = 0;

        let mut r = Vec::new();

        if cell_x > 0 && map[cell_y][cell_x-1] != 0 {
            regions += 1;
            r.push(map[cell_y][cell_x-1]);
        }

        if cell_y > 0 && map[cell_y-1][cell_x] != 0 && !r.contains(&map[cell_y-1][cell_x]){
            regions += 1;
            r.push(map[cell_y-1][cell_x]);
        }

        if cell_x < self.dungeon_width - 1 && map[cell_y][cell_x+1] != 0 && !r.contains(&map[cell_y][cell_x+1]) {
            regions += 1;
            r.push(map[cell_y][cell_x+1]);
        }

        if cell_y < self.dungeon_height - 1 && map[cell_y+1][cell_x] != 0 && !r.contains(&map[cell_y+1][cell_x])  {
            regions += 1;
            r.push(map[cell_y+1][cell_x]);
        }

        return regions == 2;
    }

    fn get_connector_regions(&self, map: &Vec<Vec<u16>>, cell: &I16Vec2) -> [u16; 2] {
        let mut regions = [0; 2];

        if !self.is_connector(map, cell) {
            return regions;
        }

        let cell_x = cell.x as usize;
        let cell_y = cell.y as usize;

        if cell_x > 0 && cell_x < self.dungeon_width - 1 {
            if map[cell_y][cell_x - 1] != map[cell_y][cell_x + 1] {
                regions[0] = map[cell_y][cell_x - 1];
                regions[1] = map[cell_y][cell_x + 1];
            }
        }

        if cell_y > 0 && cell_y < self.dungeon_height - 1 {
            if map[cell_y - 1][cell_x] != map[cell_y + 1][cell_x] {
                regions[0] = map[cell_y - 1][cell_x];
                regions[1] = map[cell_y + 1][cell_x];
            }
        }

        return regions;
    }

    fn can_carve(&self, map: &Vec<Vec<u16>>, cell: &I16Vec2, dir: &I16Vec2) -> bool {
        let edge = *cell + *dir*3;
        if edge.x >= self.dungeon_width as i16 || edge.x < 0 || edge.y >= self.dungeon_height as i16 || edge.y < 0 {
            return false;
        }

        let pos = *cell + *dir*2;
        return map[pos.y as usize][pos.x as usize] == 0;
    }

    fn carve(&self, map: &mut Vec<Vec<u16>>, cell: &I16Vec2, value: u16) {
        map[cell.y as usize][cell.x as usize] = value;
    }
}

pub struct URect {
    pub x: usize,
    pub y: usize,
    pub w: usize,
    pub h: usize,
}

impl URect {
    pub fn new(x: usize, y: usize, w: usize, h: usize) -> Self {
        Self { x, y, w, h }
    }

    pub fn overlaps(&self, other: &Self) -> bool {
        if self.x > other.x+other.w || self.x+self.w < other.x {
            return false;
        }

        if self.y > other.y+other.h || self.y+self.h < other.y {
            return false;
        }

        return true;
    }
}


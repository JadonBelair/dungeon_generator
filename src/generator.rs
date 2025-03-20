use macroquad::{
    math::I16Vec2,
    rand::{ChooseRandom, gen_range},
};

const DIRECTIONS: [I16Vec2; 4] = [I16Vec2::X, I16Vec2::Y, I16Vec2::NEG_X, I16Vec2::NEG_Y];

const DUNGEON_WIDTH: usize = 64;
const DUNGEON_HEIGHT: usize = 36;

const MAX_ROOM_SIZE: usize = 11;

const ROOM_ATTEMPTS: u16 = 600;
const WINDING_CHANCE: u8 = 50;
const CONNECTIVITY_CHANCE: u8 = 10;

pub struct Generator {
    pub dungeon_width: usize,
    pub dungeon_height: usize,
    pub max_room_size: usize,
    pub room_attempts: u16,
    pub winding_chance: u8,
    pub connectivity_chance: u8,
}

impl Generator {
    pub fn new() -> Self {
        Self {
            dungeon_width: DUNGEON_WIDTH,
            dungeon_height: DUNGEON_HEIGHT,
            max_room_size: MAX_ROOM_SIZE,
            room_attempts: ROOM_ATTEMPTS,
            winding_chance: WINDING_CHANCE,
            connectivity_chance: CONNECTIVITY_CHANCE,
        }
    }

    pub fn generate(&self) -> Vec<Vec<u16>> {
        let mut map = vec![vec![0; self.dungeon_width]; self.dungeon_height];

        let final_room = self.fill_rooms(&mut map) - 1;
        let mut id = final_room + 1;

        // loop through every possible starting location for a maze
        // and fill in any blank spots with a new maze
        for y in (1..self.dungeon_height - 1).step_by(2) {
            for x in (1..self.dungeon_width - 1).step_by(2) {
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

        // only attempts to place rooms a certain number of times
        while attemps < self.room_attempts {
            attemps += 1;
            // ensures both room size and position are odd
            let room_width = gen_range(1, (self.max_room_size - 1) / 2) * 2 + 1;
            let room_height = gen_range(1, (self.max_room_size - 1) / 2) * 2 + 1;
            let room_x = gen_range(1, (self.dungeon_width - room_width - 1) / 2) * 2 + 1;
            let room_y = gen_range(1, (self.dungeon_height - room_height - 1) / 2) * 2 + 1;

            let new_room = URect::new(room_x, room_y, room_width, room_height);

            let mut overlaps = false;
            for room in &rooms {
                if new_room.overlaps(room) {
                    overlaps = true;
                    break;
                }
            }

            if !overlaps {
                // fill each square within the room with its id
                for y in room_y..(room_y + room_height) {
                    for x in room_x..(room_x + room_width) {
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

        // fill out the maze until the
        // backtracking burns through all cells
        while !cells.is_empty() {
            let cell = cells.last().unwrap();

            let mut unmade_cells: Vec<I16Vec2> = Vec::new();

            // check each direction to see if it is free to move to
            for dir in &DIRECTIONS {
                if self.can_carve(&map, cell, dir) {
                    unmade_cells.push(*dir);
                }
            }

            if !unmade_cells.is_empty() {
                // gives a chance for the maze to keep going in the same direction
                let dir = if unmade_cells.contains(&last_dir)
                    && gen_range(0, 100) > self.winding_chance
                {
                    last_dir
                } else {
                    *unmade_cells.choose().unwrap()
                };

                self.carve(map, &(*cell + dir), id);
                self.carve(map, &(*cell + dir * 2), id);

                cells.push(*cell + dir * 2);
                last_dir = dir;
            } else {
                // no valid movement directions were found
                // so pop the last cell and check if it can branch
                cells.pop();
            }
        }
    }

    fn connect_regions(&self, map: &mut Vec<Vec<u16>>, final_room: u16) {
        let mut connectors = Vec::new();
        let mut main = Vec::new();

        for y in 0..self.dungeon_height {
            for x in 0..self.dungeon_width {
                let pos = I16Vec2::new(x as i16, y as i16);
                if self.is_connector(&map, &pos) {
                    connectors.push(pos);
                }
            }
        }

        // make a random room the main region
        main.push(gen_range(1, final_room + 1));

        while !connectors.is_empty() {
            // find all connectors touching the main region
            // and pick one at random
            let mut valid_indexes = Vec::new();
            for i in 0..connectors.len() {
                let pos = &connectors[i];
                let connector_regions = self.get_connector_regions(&map, pos);
                if main.contains(&connector_regions[0]) || main.contains(&connector_regions[1]) {
                    valid_indexes.push(i);
                }
            }
            let current_index = *valid_indexes.choose().unwrap();
            let current_connector = connectors[current_index];

            // find the new region that this connector touches
            let connector_regions = self.get_connector_regions(&map, &current_connector);
            let new_region = if main.contains(&connector_regions[0]) {
                connector_regions[1]
            } else {
                connector_regions[0]
            };

            // remove the current connector
            self.carve(map, &current_connector, new_region);
            connectors.remove(current_index as usize);

            // loop through all connectors and remove any that are touching this new region
            // with a chance to punch through the wall when deleting the connector
            for i in (0..connectors.len()).rev() {
                let test = connectors[i];
                let test_regions = self.get_connector_regions(&map, &test);

                // ensure that the connector touches both the main region and the new region
                if (test_regions[0] == new_region || test_regions[1] == new_region)
                    && (main.contains(&test_regions[0]) || main.contains(&test_regions[1]))
                {
                    let punch = gen_range(0, 100) < self.connectivity_chance;
                    if punch {
                        self.carve(map, &test, new_region);
                    }
                    connectors.remove(i);
                }
            }

            // the new region has been connected to main
            // so add it to the list of connected regions
            main.push(new_region);
        }
    }

    fn remove_dead_ends(&self, map: &mut Vec<Vec<u16>>) {
        let mut done_removing = false;
        while !done_removing {
            let mut pos = I16Vec2::ZERO;
            let mut found = false;
            // loop through all positions on the map
            // and find the first dead-end
            for y in 1..(map.len() - 1) {
                if found {
                    break;
                }
                for x in 1..(map[y].len() - 1) {
                    if map[y][x] == 0 {
                        continue;
                    }

                    let mut exits = 0;
                    for dir in DIRECTIONS {
                        if map[(y as i16 + dir.y) as usize][(x as i16 + dir.x) as usize] != 0 {
                            exits += 1;
                        }
                    }

                    // a cell is only a dead-end if
                    // there is only 1 way out of it
                    if exits != 1 {
                        continue;
                    }

                    pos = I16Vec2::new(x as i16, y as i16);
                    found = true;
                    break;
                }
            }

            if found {
                self.carve(map, &pos, 0);
            } else {
                // if we couldnt find a dead-end
                // then they have all been removed
                done_removing = true;
            }
        }
    }

    /// connectors will never be diagonal because of how to map is constructed
    /// so we only need to check if it connects vertically or horizontally
    fn is_connector(&self, map: &Vec<Vec<u16>>, cell: &I16Vec2) -> bool {
        let cell_x = cell.x as usize;
        let cell_y = cell.y as usize;
        if map[cell_y][cell_x] != 0 {
            return false;
        }

        if (cell_x > 0 && cell_x < self.dungeon_width - 1)
            && (map[cell_y][cell_x - 1] != 0 && map[cell_y][cell_x + 1] != 0)
            && (map[cell_y][cell_x - 1] != map[cell_y][cell_x + 1])
        {
            return true;
        }

        if (cell_y > 0 && cell_y < self.dungeon_height - 1)
            && (map[cell_y - 1][cell_x] != 0 && map[cell_y + 1][cell_x] != 0)
            && (map[cell_y - 1][cell_x] != map[cell_y + 1][cell_x])
        {
            return true;
        }

        return false;
    }

    fn get_connector_regions(&self, map: &Vec<Vec<u16>>, cell: &I16Vec2) -> [u16; 2] {
        let mut regions = [0; 2];

        if !self.is_connector(map, cell) {
            return regions;
        }

        let cell_x = cell.x as usize;
        let cell_y = cell.y as usize;

        // we can skip the cell != 0 checks here because
        // theyre done in the is_connector function

        if (cell_x > 0 && cell_x < self.dungeon_width - 1)
            && (map[cell_y][cell_x - 1] != map[cell_y][cell_x + 1])
        {
            regions[0] = map[cell_y][cell_x - 1];
            regions[1] = map[cell_y][cell_x + 1];
        }

        if (cell_y > 0 && cell_y < self.dungeon_height - 1)
            && (map[cell_y - 1][cell_x] != map[cell_y + 1][cell_x])
        {
            regions[0] = map[cell_y - 1][cell_x];
            regions[1] = map[cell_y + 1][cell_x];
        }

        return regions;
    }

    fn can_carve(&self, map: &Vec<Vec<u16>>, cell: &I16Vec2, dir: &I16Vec2) -> bool {
        // this check ensures that there will
        // always be a wall beside where we carve
        let edge = *cell + *dir * 3;
        if edge.x >= self.dungeon_width as i16
            || edge.x < 0
            || edge.y >= self.dungeon_height as i16
            || edge.y < 0
        {
            return false;
        }

        let pos = *cell + *dir * 2;
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
        if self.x > other.x + other.w || self.x + self.w < other.x {
            return false;
        }

        if self.y > other.y + other.h || self.y + self.h < other.y {
            return false;
        }

        return true;
    }
}

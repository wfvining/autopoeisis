/// Implementing the example of an autopoetic system from Varela,
/// Maturana, & Urbie (1974)

extern crate rand;

use rand::Rng;

#[derive(Clone,PartialEq)]
enum Entity {
    Hole,
    Substrate,
    Catalyst,
    Link,
    BondedLink,
}

struct Universe {
    cells:  Vec<Vec<Entity>>,
    width:  usize,
    height: usize,
    decay_rate: f64,
}

impl Universe {
    pub fn new(w: usize, h: usize, decay_rate: f64, catalysts: usize)
               -> Universe {
        let mut rng = rand::thread_rng();
        let mut cells = vec![vec![Entity::Substrate;w];h];

        for i in 0..catalysts {
            let mut x = rng.gen_range(0, w);
            let mut y = rng.gen_range(0, h);
            while cells[y][x] == Entity::Catalyst {
                x = rng.gen_range(0, w);
                y = rng.gen_range(0, h);
            }
            cells[y][x] = Entity::Catalyst;
        }

        Universe { height: h, width: w, cells: cells, decay_rate: decay_rate }
    }

    fn get_neighbor(&self, n: usize, x: usize, y: usize)
                    -> (usize, usize) {
        let y = y as i64;
        let x = x as i64;
        let mut new_x: i64;
        let mut new_y: i64;
        match n {
            0 => { // go left;
                new_x = x - 1;
                new_y = y;
            }
            1 => { // go left and up
                new_x = x - 1;
                new_y = y - 1;
            }
            2 => { // go up
                new_x = x;
                new_y = y - 1;
            }
            3 => { // go up an right
                new_x = x + 1;
                new_y = y - 1;
            }
            4 => { // go right
                new_x = x + 1;
                new_y = y;
            }
            5 => { // go down and right
                new_x = x + 1;
                new_y = y + 1;
            }
            6 => { // go down
                new_x = x;
                new_y = y+1;
            }
            7 => { // go down and left
                new_x = x-1;
                new_y = y+1;
            }
            _ => { new_x = x; new_y = y }
        }
        if new_x == self.width as i64 {
            new_x = 0;
        }
        else if new_x < 0 {
            new_x = self.width as i64 - 1; 
        }

        if new_y == self.height as i64 {
            new_y = 0;
        }
        else if new_y < 0 {
            new_y = self.height as i64 - 1;
        }
        (new_x as usize, new_y as usize)
    }

    pub fn update(&mut self) {
        // select a random cell and update it This may not ber the
        // most reasonable method, since it will frequently update
        // substrate cells, leaving the actual things we care about
        // idle.
        let mut rng = rand::thread_rng();
        let x = rng.gen_range(0, self.width);
        let y = rng.gen_range(0, self.height);

        match self.cells[y][x] {
            Entity::Catalyst => {
                let (new_x, new_y) = self.get_neighbor(rng.gen_range(0,8), x, y);
            }
            Entity::Hole =>
            {}
            Entity::Substrate => {
                let (new_x, new_y) = self.get_neighbor(rng.gen_range(0, 8), x, y);
            }
            Entity::Link =>
            {}
            Entity::BondedLink =>
            {}
        }
    }
}

fn main() {
    println!("Hello, world!");
}

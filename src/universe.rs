extern crate rand;

use rand::Rng;
use std::collections::HashMap;
use std::collections::HashSet;

#[derive(Clone,PartialEq)]
pub enum Entity {
    Substrate,
    Hole,
    Catalyst,
    Link,
    BondedLink, // need to break this in to single/double bonded links
}

#[derive(Clone, PartialEq, Hash)]
pub struct Pos(i32,i32)

pub struct Universe {
    holes: HashSet<Pos>,
    catalysts: HashSet<Pos>,
    free_links: HashSet<Pos>,
    single_bonded_links: HashMap<Pos, Pos>,
    double_bonded_liks: HashMap<Pos, (Pos, Pos)>,
    decay_rate: f64,
    width: i32,
    height: i32,
}

impl Universe {
    pub fn new(w: usize, h: usize, decay_rate: f64, catalysts: usize)
               -> Universe {
        let mut rng = rand::thread_rng();
        let mut cats = HashSet::new();

        while cats.len() < catalysts {
            let p = Pos(rng.gen_range(0, w), rng.gen_range(0, h));
            cats.insert(p);
        }
        
        Universe { decay_rate: decay_rate,
                   catalysts: cats,
                   holes: HashSet::new(),
                   free_links: HashSet::new(),
                   single_bonded_links: HashMap::new(),
                   double_bonded_links: HashMap::new(),
                   width: w,
                   height: h,
        }
    }


    pub fn update(&mut self) {
        // select a random cell and update it This may not ber the
        // most reasonable method, since it will frequently update
        // substrate cells, leaving the actual things we care about
        // idle.
        let mut rng = rand::thread_rng();
        let p = Pos(rng.gen_range(0, self.width), rng.gen_range(0, self.height));

        if(self.catalysts.contains(p)) {
            // react with the substrate and move a random direction.
            // TODO: React with the substrate
            // select a random direction to move
            let p1 = neighbor(p, rng.gen_range(0,8));
            if !self.double_bonded_links.contains(p1) {
                // then we can displace whatever is in p1.
                // self.catalysts.remove(p)
                // self.catalysts.insert(p1)
                // if p1 is in any other set then replace it with p
            }
        }
    }
}

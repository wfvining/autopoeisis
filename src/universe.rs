extern crate rand;

use rand::Rng;
use std::collections::HashSet;
use std::cmp::{PartialEq,Eq};

#[derive(Clone, PartialEq, Hash, Eq, Copy)]
pub struct Pos {
    pub x: i32,
    pub y: i32
}

pub fn pos(x: i32, y: i32) -> Pos {
    Pos { x: x, y: y }
}

#[derive(Hash)]
struct Bond {
    a: Pos,
    b: Pos,
}

impl Bond {
    pub fn new(p1: Pos, p2: Pos) -> Bond {
        Bond { a: p1, b: p2 }
    }
}

impl PartialEq for Bond {
    fn eq(&self, other: &Bond) -> bool {
        (self.a == other.a && self.b == other.b)
            || (self.a == other.b && self.b == other.a)
    }
}
impl Eq for Bond {}

pub struct Universe {
    holes: HashSet<Pos>,
    catalysts: HashSet<Pos>,
    links: HashSet<Pos>,
    bonds: HashSet<Bond>,
    decay_rate: f64,
    upper_left: Pos,
    lower_right: Pos,
}

fn neighbor(p: &Pos, n: i32) -> Pos {
    match n {
        0 => pos(p.x, p.y-1),
        1 => pos(p.x+1,   p.y),
        2 => pos(p.x, p.y+1),
        3 => pos(p.x-1, p.y),
        _ => p.clone() // this is an ugly hack
    }
}

fn distance(p1: &Pos, p2: &Pos) -> f64 {
    (((p1.x - p2.x) as f64).powi(2) + ((p1.y - p2.y) as f64).powi(2)).sqrt()
}

fn new_bounds(p: &Pos, upper_left: &Pos, lower_right: &Pos) -> (Pos, Pos) {
    let ux = if p.x < upper_left.x {
        p.x
    }
    else {
        upper_left.x
    };

    let lx = if p.x >= lower_right.x {
        p.x + 1
    }
    else {
        lower_right.x
    };
    
    let uy = if p.y < upper_left.y {
        p.y
    }
    else {
        upper_left.y
    };
    
    let ly = if p.y >= lower_right.y {
        p.y + 1
    }
    else {
        lower_right.y
    };

    (pos(ux, uy), pos(lx, ly))
}

impl Universe {
    pub fn new(w: i32, h: i32 , decay_rate: f64, catalysts: usize)
               -> Universe {
        let mut rng = rand::thread_rng();
        let mut cats = HashSet::new();

        while cats.len() < catalysts {
            let p = pos(rng.gen_range::<i32>(0, w), rng.gen_range::<i32>(0, h));
            cats.insert(p);
        }
        
        Universe { decay_rate: decay_rate,
                   catalysts: cats,
                   holes: HashSet::new(),
                   links: HashSet::new(),
                   bonds: HashSet::new(),
                   upper_left: pos(0, 0),
                   lower_right: pos(w, h),
        }
    }

    fn is_substrate(&self, p: &Pos) -> bool {
        !(self.holes.contains(p)
          || self.catalysts.contains(p)
          || self.links.contains(p))
    }

    fn is_link(&self, p: &Pos) -> bool {
        self.links.contains(p)
    }

    fn is_catalyst(&self, p: &Pos) -> bool {
        self.catalysts.contains(p)
    }

    fn is_hole(&self, p: &Pos) -> bool {
        self.holes.contains(p)
    }

    fn num_bonds(&self, p: &Pos) -> i32 {
        self.bonds.iter()
            .fold(0, |acc, ref b|
                  if b.a == *p || b.b == *p
                  {
                      acc + 1
                  } else
                  {
                      acc
                  })
    }

    fn bonded(&self, p1: &Pos, p2: &Pos) -> bool {
        for ref bond in self.bonds.iter() {
            if **bond == Bond::new(*p1, *p2) {
                return true;
            }
        }
        return false;
    }
    
    fn is_bonded(&self, p: &Pos) -> bool {
        self.bonds.iter()
            .fold(false, |acc, ref b| b.a == *p || b.b == *p || acc)
    }

    fn num_links(&self) -> usize {
        self.links.len()
    }

    fn num_holes(&self) -> usize {
        self.holes.len()
    }

    fn num_catalysts(&self) -> usize {
        self.catalysts.len()
    }

    fn bond(&mut self, p: &Pos) {
        assert!(self.num_links() == self.num_holes(), "bond() - pre");
        let neighboring_links: Vec<Pos> = (0..4).into_iter()
            .map(|i| neighbor(&p, i))
            .filter(|&n| self.is_link(&n)
                    && self.num_bonds(&n) < 2
                     // don't include links that are already bonded to p
                    && !self.bonded(&n, &p))
            .collect();
        if !neighboring_links.is_empty() {
            let mut rng = rand::thread_rng();
            let b = neighboring_links[rng.gen_range(0, neighboring_links.len())];
            self.bonds.insert(Bond::new(b, *p));
        }
        assert!(self.num_links() == self.num_holes(), "bond() - post");
    }

    /// Eliminate any bonds that are no longer valid since the link at
    /// `dead_link` has decayed.
    fn fix_bonds(&mut self, dead_link: &Pos) {
        self.bonds
            .retain(|ref bond| bond.a != *dead_link && bond.b != *dead_link);
    }

    fn update_link(&mut self, p: &Pos) {
        let mut rng = rand::thread_rng();
        let d = rng.gen_range(0.0, 1.0);
        assert!(self.is_link(&p));
        assert!(!self.is_hole(&p));
        if d < self.decay_rate {
            // find a hole to put the second substrate element into.
            let neighboring_holes: Vec<Pos> = (0..4).into_iter()
                .map(|i| neighbor(&p, i))
                .filter(|pos| self.holes.contains(pos))
                .collect();
            if neighboring_holes.is_empty() {
                let h = self.holes.iter()
                    .fold(None, |closest, ph| match closest {
                        Some(c) => {
                            if distance(&ph, &p) < distance(&c, &p)
                            {
                                Some(*ph)
                            } else
                            {
                                closest
                            }
                        },
                        None => Some(*ph)
                    }).unwrap();
                self.holes.remove(&h);
            }
            else {
                let i = rng.gen_range(0, neighboring_holes.len());
                let h = &neighboring_holes[i];
                self.holes.remove(h);
            }
            // now remove the link
            self.links.remove(p);
            self.fix_bonds(p);
        }
        else {
            assert!(self.num_links() == self.num_holes(), "move link - pre");
            // only free links move
            let p = if !self.is_bonded(&p) {
                let p1 = neighbor(&p, rng.gen_range(0,4));
                let (ul, lr) = new_bounds(&p1, &self.upper_left, &self.lower_right);
                self.upper_left = ul;
                self.lower_right = lr;
                if self.is_substrate(&p1) {
                    self.links.remove(&p);
                    self.links.insert(p1.clone());
                }
                else if self.is_hole(&p1) {
                    assert!(self.holes.remove(&p1));
                    self.links.remove(&p);
                    assert!(self.holes.insert(p.clone()));
                    self.links.insert(p1.clone());
                    assert!(self.num_links() == self.num_holes(), "move link - hole");
                }
                assert!(self.num_links() == self.num_holes(), "move link");
                p1
            } else { *p };

            // if there are less than two bonds
            if self.num_bonds(&p) < 2 {
                self.bond(&p);
            }
        }
    }
    
    /// Select a random location and update it.
    pub fn update(&mut self) {
        let mut rng = rand::thread_rng();
        let i = rng.gen_range(0, self.num_holes() + self.num_links() + self.num_catalysts());
        let p = if i < self.num_holes() {
            self.holes.clone().iter().nth(i).unwrap().clone()
        } else if i - self.num_holes() < self.num_links() {
            self.links.clone().iter().nth(i - self.num_holes()).unwrap().clone()
        } else {
            self.catalysts.clone().iter().nth(i - (self.num_holes() + self.num_links())).unwrap().clone()
        };

        if self.is_catalyst(&p) {
            assert!(self.num_links() == self.num_holes(), "catalyst - pre");
            // react with the substrate and move a random direction.
            // select a random direction to move
            let neighbors: Vec<Pos> = (0..4).into_iter()
                .map(|i| neighbor(&p, i))
                .filter(|&pos| !self.is_bonded(&pos)).collect();
            if neighbors.is_empty() {
                return; // can't move.
            }
            let p1 = neighbors[rng.gen_range(0, neighbors.len())];
            let neighbors2: Vec<Pos> = (0..4).into_iter()
                .map(|i| neighbor(&p, i))
                .filter(|&pos| self.is_substrate(&pos) && pos != p1).collect();
            
            let (ul, lr) = new_bounds(&p1, &self.upper_left, &self.lower_right);
            self.upper_left = ul;
            self.lower_right = lr;
                        
            // move the catalyst and create a new link and a new hole
            if self.is_substrate(&p1) && !neighbors2.is_empty() {
                let p2 = neighbors2[rng.gen_range(0, neighbors2.len())];
                let (ul, lr) = new_bounds(&p2, &ul, &lr);
                self.upper_left = ul;
                self.lower_right = lr;
                
                self.catalysts.remove(&p);
                self.holes.insert(p2);
                self.links.insert(p.clone());
                self.catalysts.insert(p1);
            }
            else if self.is_hole(&p1) {
                self.catalysts.remove(&p);
                self.holes.remove(&p1);
                self.holes.insert(p.clone());
                self.catalysts.insert(p1);
            }
            else if self.is_link(&p1) {
                self.catalysts.remove(&p);
                self.catalysts.insert(p1.clone());
                self.links.remove(&p1);
                self.links.insert(p.clone());
            }
            else if !self.is_catalyst(&p1) {
                self.catalysts.remove(&p);
                self.catalysts.insert(p1.clone());
            }
            assert!(self.num_links() == self.num_holes(), "catalyst - post");
        }
        else if self.is_link(&p) {
            self.update_link(&p)
        }
        else if self.is_hole(&p) {
            assert!(self.num_links() == self.num_holes(), "update hole - pre");
            let p1 = neighbor(&p, rng.gen_range(0, 4));
            let (ul, lr) = new_bounds(&p1, &self.upper_left, &self.lower_right);
            self.upper_left = ul;
            self.lower_right = lr;

            if self.is_link(&p1) && !self.is_bonded(&p1) {
                // displace the free link.
                self.links.remove(&p1);
                self.links.insert(p.clone());
                self.holes.remove(&p);
                self.holes.insert(p1.clone());
            }
            else if self.is_catalyst(&p1) {
                self.catalysts.remove(&p1);
                self.catalysts.insert(p.clone());
                self.holes.remove(&p);
                self.holes.insert(p1.clone());
            }
            else if self.is_substrate(&p1) {
                self.holes.remove(&p);
                self.holes.insert(p1.clone());
            }
            // TODO: holes can pass through bonded links.
  
            assert!(self.num_links() == self.num_holes(), "update hole - post");
        }
    }

    pub fn get_catalysts_in(&self, top_left: &Pos, size: &Pos) -> Vec<Pos> {
        let bottom_right = pos(top_left.x + size.x, top_left.y + size.y);
        self.catalysts.iter().filter(|&pos| {
            top_left.x <= pos.x && pos.x <= bottom_right.x &&
                top_left.y <= pos.y && pos.y <= bottom_right.y
        }).cloned().collect()
    }

    pub fn get_holes_in(&self, top_left: &Pos, size: &Pos) -> Vec<Pos> {
        let bottom_right = pos(top_left.x + size.x, top_left.y + size.y);
        self.holes.iter().filter(|&pos| {
            top_left.x <= pos.x && pos.x <= bottom_right.x &&
                top_left.y <= pos.y && pos.y <= bottom_right.y
        }).cloned().collect()
    }

    pub fn get_free_links_in(&self, top_left: &Pos, size: &Pos) -> Vec<Pos> {
        let bottom_right = pos(top_left.x + size.x, top_left.y + size.y);
        self.links.iter().filter(|&pos| { self.num_bonds(pos) == 0
                                 && top_left.x <= pos.x && pos.x <= bottom_right.x &&
                                 top_left.y <= pos.y && pos.y <= bottom_right.y
        }).cloned().collect()
    }

    pub fn get_single_bonded_links_in(&self, top_left: &Pos, size: &Pos) -> Vec<Pos> {
        let bottom_right = pos(top_left.x + size.x, top_left.y + size.y);
        self.links.iter().filter(|&pos| {
            self.num_bonds(&pos) == 1
                && top_left.x <= pos.x && pos.x <= bottom_right.x &&
                top_left.y <= pos.y && pos.y <= bottom_right.y
        }).cloned().collect()
    }

    pub fn get_double_bonded_links_in(&self, top_left: &Pos, size: &Pos) -> Vec<Pos> {
        let bottom_right = pos(top_left.x + size.x, top_left.y + size.y);
        self.links.iter().filter(|&pos| {
            self.num_bonds(&pos) == 2
                && top_left.x <= pos.x && pos.x <= bottom_right.x &&
                top_left.y <= pos.y && pos.y <= bottom_right.y
        }).cloned().collect()
    }
}


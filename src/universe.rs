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

#[derive(Clone, PartialEq, Hash, Eq, Copy)]
pub struct Pos {
    pub x: i32,
    pub y: i32
}

pub fn pos(x: i32, y: i32) -> Pos {
    Pos { x: x, y: y }
}

pub struct Universe {
    holes: HashSet<Pos>,
    catalysts: HashSet<Pos>,
    free_links: HashSet<Pos>,
    single_bonded_links: HashMap<Pos, Pos>,
    double_bonded_links: HashMap<Pos, (Pos, Pos)>,
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

fn different(p: &Pos, ps: (Pos, Pos)) -> Pos {
    if p == &ps.0 { ps.1 } else { ps.0 }
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
                   free_links: HashSet::new(),
                   single_bonded_links: HashMap::new(),
                   double_bonded_links: HashMap::new(),
                   upper_left: pos(0, 0),
                   lower_right: pos(w, h),
        }
    }

    fn is_substrate(&self, p: &Pos) -> bool {
        !(self.holes.contains(p)
          || self.catalysts.contains(p)
          || self.free_links.contains(p)
          || self.single_bonded_links.contains_key(p)
          || self.double_bonded_links.contains_key(p))
    }

    fn is_link(&self, p: &Pos) -> bool {
        self.free_links.contains(p)
            || self.single_bonded_links.contains_key(p)
            || self.double_bonded_links.contains_key(p)
    }

    fn num_links(&self) -> usize {
        self.free_links.len() + self.single_bonded_links.len() + self.double_bonded_links.len()
    }

    fn num_holes(&self) -> usize {
        self.holes.len()
    }

    fn form_single_bond(&mut self, p: &Pos, b: &Pos) {
        
    }

    fn form_double_bond(&mut self, p: &Pos, b1: &Pos, b2: &Pos) {
        
    }

    fn bond(&mut self, p: &Pos) {
        assert!(self.num_links() == self.num_holes(), "bond() - pre");
        let neighboring_links: Vec<Pos> = (0..4).into_iter()
            .map(|i| neighbor(&p, i))
            .filter(|&n| self.free_links.contains(&n) || (self.single_bonded_links.contains_key(&n) && self.single_bonded_links.get(&n).unwrap() != p)).collect();
        if !neighboring_links.is_empty() {
            let mut rng = rand::thread_rng();
            let b = &neighboring_links[rng.gen_range(0, neighboring_links.len())];

            if self.free_links.contains(&b) {
                self.free_links.remove(&b);
                self.single_bonded_links.insert(b.clone(), p.clone());
                assert!(self.num_links() == self.num_holes(), "bond to free link");
            }
            else {
                let bond = self.single_bonded_links.remove(&b).unwrap();
                self.double_bonded_links.insert(b.clone(), (p.clone(), bond));
                assert!(self.num_links() == self.num_holes(), "bond to sb link");
            }

            if self.free_links.contains(&p) {
                self.free_links.remove(&p);
                self.single_bonded_links.insert(p.clone(), b.clone());
                assert!(self.num_links() == self.num_holes(), "bond free link");
            }
            else {
                let bond = self.single_bonded_links.remove(&p).unwrap();
                self.double_bonded_links.insert(p.clone(), (b.clone(), bond));
                assert!(bond != *p && bond != *b && p != b, "all bonds distinct");
                assert!(self.num_links() == self.num_holes(), "bond sb link");
            }
        }
    }

    fn fix_bonds(&mut self, dead_link: &Pos, broken_bonds: (Option<Pos>, Option<Pos>)) {
        // dead link is gone, it breviously had bonds in broken_bonds
        match broken_bonds {
            (None, None) => {}, // don't need to do anything
            (Some(l), None) => {
                if self.single_bonded_links.contains_key(&l) {
                    self.single_bonded_links.remove(&l);
                    self.free_links.insert(l.clone());
                }
                else { // l had two bonds, not is has only one.
                    let b = different(dead_link, self.double_bonded_links.remove(&l).unwrap());
                    self.single_bonded_links.insert(l.clone(), b.clone());
                }
            },
            (None, Some(l)) => {
                if self.single_bonded_links.contains_key(&l) {
                    self.single_bonded_links.remove(&l);
                    self.free_links.insert(l.clone());
                }
                else { // l had two bonds, not is has only one.
                    let b = different(dead_link, self.double_bonded_links.remove(&l).unwrap());
                    self.single_bonded_links.insert(l.clone(), b.clone());
                }
            },
            (Some(l1), Some(l2)) => {
                if self.single_bonded_links.contains_key(&l1) {
                    self.single_bonded_links.remove(&l1);
                    self.free_links.insert(l1.clone());
                }
                else { // l had two bonds, not is has only one.
                    let b = different(dead_link, self.double_bonded_links.remove(&l1).unwrap());
                    self.single_bonded_links.insert(l1.clone(), b.clone());
                }

                if self.single_bonded_links.contains_key(&l2) {
                    self.single_bonded_links.remove(&l2);
                    self.free_links.insert(l2.clone());
                }
                else { // l had two bonds, not is has only one.
                    let b = different(dead_link, self.double_bonded_links.remove(&l2).unwrap());
                    self.single_bonded_links.insert(l2.clone(), b.clone());
                }
            },
        }
    }

    fn update_link(&mut self, p: &Pos) {
        let mut rng = rand::thread_rng();
        let d = rng.gen_range(0.0, 1.0);
        if d < self.decay_rate {
            // find a hole to put the second substrate element into.
            let neighboring_holes: Vec<Pos> = (0..4).into_iter()
                .map(|i| neighbor(&p, i))
                .filter(|pos| self.holes.contains(pos))
                .collect();
            if neighboring_holes.is_empty() {
                let hs = self.holes.clone();
                let h = hs.into_iter().fold(None, |closest, ph| match closest {
                    Some(c) => {
                        if distance(&ph, &p) < distance(&c, &p)
                        {
                            Some(ph)
                        } else
                        {
                            closest
                        }
                    },
                    None => Some(ph)
                }).unwrap();
                self.holes.remove(&h);
            }
            else {
                let i = rng.gen_range(0, neighboring_holes.len());
                let h = &neighboring_holes[i];
                self.holes.remove(h);
            }
            assert!(self.free_links.len() + self.single_bonded_links.len() + self.double_bonded_links.len() == self.holes.len()+1, "after decay 1");
            // now remove the link
            let links = if self.free_links.contains(p) {
                self.free_links.remove(p);
                (None, None)
            }
            else if self.single_bonded_links.contains_key(p) {
                let l = self.single_bonded_links.remove(p).unwrap();
                (Some(l), None)
            }
            else { // then p was double bonded
                let (l1, l2) = self.double_bonded_links.remove(p).unwrap();
                (Some(l1), Some(l2))
            };
            self.fix_bonds(p, links);
            assert!(self.free_links.len() + self.single_bonded_links.len() + self.double_bonded_links.len() == self.holes.len(), "after decay");
        }
        else {
            // only free links move
            if self.free_links.contains(&p) {
                let last_free_len = self.free_links.len();
                let last_hole_len = self.holes.len();
                let p1 = neighbor(&p, rng.gen_range(0,4));
                let (ul, lr) = new_bounds(&p1, &self.upper_left, &self.lower_right);
                self.upper_left = ul;
                self.lower_right = lr;
                if self.is_substrate(&p1) {
                    self.free_links.remove(&p);
                    self.free_links.insert(p1.clone());
                }
                else if self.holes.contains(&p1) {
                    self.holes.remove(&p1);
                    self.free_links.remove(&p);
                    self.holes.insert(p.clone());
                    self.free_links.insert(p1.clone());
                }
                assert!(self.free_links.len() == last_free_len, "after movement - free");
                assert!(self.holes.len() == last_hole_len, "after movement - holes");
            }
            // only free or singly bound links can bond
            if self.free_links.contains(&p) || self.single_bonded_links.contains_key(&p) {
                self.bond(&p);
                assert!(self.free_links.len() + self.single_bonded_links.len() + self.double_bonded_links.len() == self.holes.len(), "after bonding");
            }
        }
    }
    
    /// Select a random location and update it.
    pub fn update(&mut self) {
        let mut rng = rand::thread_rng();
        let p = pos(rng.gen_range(self.upper_left.x, self.lower_right.x),
                    rng.gen_range(self.upper_left.y, self.lower_right.y));

        if self.catalysts.contains(&p) {
            // react with the substrate and move a random direction.
            // select a random direction to move
            let p1 = neighbor(&p, rng.gen_range(0,4));
            // this is a departure from the paper, which will select
            // any available substrate with which to interact, here we
            // select two random neighbors and interact _only_ with
            // those neighbors.
            let mut p2 = neighbor(&p, rng.gen_range(0,4));
            while p1 == p2 {
                p2 = neighbor(&p, rng.gen_range(0,4));
            }
            
            let (ul, lr) = new_bounds(&p1, &self.upper_left, &self.lower_right);
            let (ul, lr) = new_bounds(&p2, &ul, &lr);
            self.upper_left = ul;
            self.lower_right = lr;
            
            // catalysts can't displace bonded links
            if !self.double_bonded_links.contains_key(&p1) && !self.single_bonded_links.contains_key(&p1) {
                // react with the substrate to create a new free link
                if self.is_substrate(&p1) && self.is_substrate(&p2) {
                    // this is more deterministic than it should be...
                    self.holes.insert(p2.clone());
                    self.free_links.insert(p1.clone());
                }

                // displace a hole or a free link if it was in p1
                // if p1 is in any other set then replace it with p
                if self.holes.contains(&p1) {
                    self.holes.remove(&p1);
                    self.holes.insert(p.clone());
                }
                else if self.free_links.contains(&p1) {
                    self.free_links.remove(&p1);
                    self.free_links.insert(p.clone());
                }

                // move the catalyst, 
                if !self.catalysts.contains(&p1) {
                    // catalysts can displace other catalysts, but
                    // that is equivalent to the catalyst not moving at all.
                    self.catalysts.remove(&p);
                    self.catalysts.insert(p1.clone());
                }
            }
        }
        else if self.is_link(&p) {
            self.update_link(&p)
        }
        if self.holes.contains(&p) {
            let p1 = neighbor(&p, rng.gen_range(0, 4));
            let (ul, lr) = new_bounds(&p1, &self.upper_left, &self.lower_right);
            self.upper_left = ul;
            self.lower_right = lr;
            
            // TODO: holes can pass through bonded links.
            if self.single_bonded_links.contains_key(&p1)
                || self.double_bonded_links.contains_key(&p1) {
                    // look at the cell just beyond the link and see if it is substrate
                }
            else if self.free_links.contains(&p1) {
                self.free_links.remove(&p1);
                self.free_links.insert(p.clone());
            }
            else if self.catalysts.contains(&p1) {
                self.catalysts.remove(&p1);
                self.catalysts.insert(p.clone());
            }

            if !self.holes.contains(&p1) {
                self.holes.remove(&p);
                self.holes.insert(p1.clone());
            }
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
        self.free_links.iter().filter(|&pos| {
            top_left.x <= pos.x && pos.x <= bottom_right.x &&
                top_left.y <= pos.y && pos.y <= bottom_right.y
        }).cloned().collect()
    }

    pub fn get_single_bonded_links_in(&self, top_left: &Pos, size: &Pos)
                                      -> Vec<(Pos, Option<Pos>, Option<Pos>)> {
        let bottom_right = pos(top_left.x + size.x, top_left.y + size.y);
        // This is unfortunate, I could not get the types to work out
        // with filter_map so we are left with this loop.
        let mut sbs = vec![];
        for (&p, &b) in self.single_bonded_links.iter() {
            if top_left.x <= p.x && p.x <= bottom_right.x &&
                top_left.y <= p.y && p.y <= bottom_right.y
            {
                sbs.push((p.clone(), Some(b.clone()), None));
            }
        }
        sbs
    }

    pub fn get_double_bonded_links_in(&self, top_left: &Pos, size: &Pos)
                                      -> Vec<(Pos, Option<Pos>, Option<Pos>)> {
        let bottom_right = pos(top_left.x + size.x, top_left.y + size.y);
        // This is unfortunate, I could not get the types to work out
        // with filter_map so we are left with this loop.
        let mut dbs = vec![];
        for (&p, &bonds) in self.double_bonded_links.iter() {
            if top_left.x <= p.x && p.x <= bottom_right.x &&
                top_left.y <= p.y && p.y <= bottom_right.y
            {
                dbs.push((p.clone(), Some(bonds.0.clone()), Some(bonds.1.clone())));
            }
        }
        dbs

    }
}


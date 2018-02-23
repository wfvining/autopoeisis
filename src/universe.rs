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
        0 => pos(p.x,   p.y-1),
        1 => pos(p.x+1, p.y),
        2 => pos(p.x,   p.y+1),
        3 => pos(p.x-1, p.y),
        4 => pos(p.x-1, p.y-1),
        5 => pos(p.x-1, p.y+1),
        6 => pos(p.x+1, p.y+1),
        7 => pos(p.x+1, p.y-1),
        _ => panic!("neighbor(): index out of range!"),
    }
}

fn adjacent(p: &Pos, q: &Pos) -> bool {
    (p.x == q.x && (p.y == q.y+1 || p.y == q.y-1))
        || (p.y == q.y && (p.x == q.x+1 || p.x == q.x-1))
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
        let b = Bond::new(*p1, *p2);
        for ref bond in self.bonds.iter() {
            if **bond == b {
                return true;
            }
        }
        false
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

    /// check that all bonds are valid (ie. have a corresponding
    /// adjacent link)
    fn validate_bonds(&self) -> bool {
        self.bonds.iter()
            .fold(true, |acc, ref bond| {
                self.is_link(&bond.a) && self.is_link(&bond.b) && acc
            })
    }

    fn bond_angle_obtuse(&self, p: &Pos, b: &Pos) -> bool {
        if self.num_bonds(&p) == 0 {
            true
        }
        else {
            (0..8).into_iter()
                .map(|i| neighbor(&p, i))
                .fold(true, |acc, n| acc && !(self.bonded(&n, &p) && adjacent(&n, b)))
        }
    }

    fn bond(&mut self, p: &Pos) {
        // TODO: prefferentially bond to already bound links
        // let neighboring_free_links: Vec<Pos> = (0..8).into_iter()
        //     .map(|i| neighbor(&p, i))
        //     .filter(|&n| self.is_link(&n)
        //             && self.num_bonds(&n) == 0
        //             && !self.bonded(&n, &p))
        //     .collect();
        let neighboring_links: Vec<Pos> = (0..8).into_iter()
            .map(|i| neighbor(&p, i))
            .filter(|&n| self.is_link(&n)
                    && self.num_bonds(&n) < 2
                    // exclude links that would result in an acute bond angle
                    && self.bond_angle_obtuse(p, &n)
                    // don't include links that are already bonded to p
                    && !self.bonded(&n, &p))
            .collect();
        if !neighboring_links.is_empty() {
            let mut rng = rand::thread_rng();
            let b = neighboring_links[rng.gen_range(0, neighboring_links.len())];
            self.bonds.insert(Bond::new(b, *p));
        }
    }

    /// Eliminate any bonds that are no longer valid since the link at
    /// `dead_link` has decayed.
    fn fix_bonds(&mut self, dead_link: &Pos) {
        self.bonds
            .retain(|ref bond| bond.a != *dead_link && bond.b != *dead_link);
    }

    /// returns a list of all holes adjacent to position p.
    fn get_adjacent_holes(&self, p: &Pos) -> Vec<Pos> {
        (0..8).into_iter()
            .map(|i| neighbor(p, i))
            .filter(|&p| self.is_hole(&p)).collect()
    }

    /// return a list of holes that are separated from p by a bonded link
    fn get_displaced_holes(&self, p: &Pos) -> Vec<Pos> {
        (0..4).into_iter()
            .map(|i| (neighbor(p, i), neighbor(&neighbor(p, i), i)))
            .filter(|&p| self.is_link(&p.0) && self.is_bonded(&p.0) && self.is_hole(&p.1))
            .map(|p| p.1).collect()
    }

    fn update_link(&mut self, p: &Pos) {
        let mut rng = rand::thread_rng();
        let d = rng.gen_range(0.0, 1.0);
        if d < self.decay_rate {
            // now remove the link
            self.links.remove(p);
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
            self.fix_bonds(p);
            assert!(self.validate_bonds(), "update link - post:decay");
            assert!(self.num_bonds(p) <= (0..4).into_iter()
                    .map(|i| neighbor(&p, i))
                    .filter(|&n| self.is_link(&n) && self.is_link(&n))
                    .fold(0, |s, _| s + 1), "decay - post");
        }
        else {
            // only free links move
            let p = if !self.is_bonded(&p) {
                let p1 = neighbor(&p, rng.gen_range(0,4));
                let (ul, lr) = new_bounds(&p1, &self.upper_left, &self.lower_right);
                self.upper_left = ul;
                self.lower_right = lr;
                if self.is_substrate(&p1) {
                    let holes = self.get_adjacent_holes(&p1);
                    let displaced_holes = self.get_displaced_holes(&p1);
                    if !holes.is_empty() {
                        let i = rng.gen_range(0, holes.len());
                        self.holes.remove(&holes[i]);
                        self.holes.insert(p.clone());
                        self.links.remove(&p);
                        self.links.insert(p1.clone());
                    }
                    else if !displaced_holes.is_empty() {
                        let i = rng.gen_range(0, displaced_holes.len());
                        self.holes.remove(&displaced_holes[i]);
                        self.holes.insert(p.clone());
                        self.links.remove(&p);
                        self.links.insert(p1.clone());
                    }
                    else {
                        self.links.remove(&p);
                        self.links.insert(p1.clone());
                    }
                    p1
                }
                else if self.is_hole(&p1) {
                    self.holes.remove(&p1);
                    self.links.remove(&p);
                    self.holes.insert(p.clone());
                    self.links.insert(p1.clone());
                    p1
                }
                else {
                    *p
                }
            } else { *p };
            // if there are less than two bonds
            assert!(self.validate_bonds(), "update link - move:bond");
            if self.num_bonds(&p) < 2 {
                self.bond(&p);
            }
            assert!(self.validate_bonds(), "update link - post:move");
        }
    }

    fn move_catalyst(&mut self, p: &Pos) -> Pos {
        let mut rng = rand::thread_rng();
        let p1 = neighbor(&p, rng.gen_range(0, 4));
        let (ul, lr) = new_bounds(&p1, &self.upper_left, &self.lower_right);
        self.upper_left = ul;
        self.lower_right = lr;

        if self.is_hole(&p1) {
            self.catalysts.remove(&p);
            self.holes.remove(&p1);
            self.holes.insert(p.clone());
            self.catalysts.insert(p1);
        }
        else if self.is_link(&p1) && !self.is_bonded(&p1) {
            let adjacent_holes = self.get_adjacent_holes(&p1);
            let displaced_holes = self.get_displaced_holes(&p1);
            if !adjacent_holes.is_empty() {
                let h = adjacent_holes[rng.gen_range(0, adjacent_holes.len())];
                self.links.remove(&p1);
                self.holes.remove(&h);
                self.links.insert(h.clone());
                self.holes.insert(p.clone());
                self.catalysts.remove(&p);
                self.catalysts.insert(p1.clone());
                self.bond(&h);
            }
            else if !displaced_holes.is_empty() {
                let h = displaced_holes[rng.gen_range(0, displaced_holes.len())];
                self.links.remove(&p1);
                self.holes.remove(&h);
                self.links.insert(h.clone());
                self.holes.insert(p.clone());
                self.catalysts.remove(&p);
                self.catalysts.insert(p1.clone());
                self.bond(&h);
            } else {
                self.catalysts.remove(&p);
                self.catalysts.insert(p1.clone());
                self.links.remove(&p1);
                self.links.insert(p.clone());
                self.bond(&p);
            }
            assert!(self.catalysts.is_disjoint(&self.links), "displace links: 0");
            assert!(self.catalysts.is_disjoint(&self.holes), "displace links: 0");
            assert!(self.links.is_disjoint(&self.holes), "displace links: 0");
        }
        else if !self.is_catalyst(&p1) && self.is_substrate(&p1) {
            let adjacent_holes = self.get_adjacent_holes(&p1);
            let displaced_holes = self.get_displaced_holes(&p1);
            if !adjacent_holes.is_empty() {
                let h = adjacent_holes[rng.gen_range(0, adjacent_holes.len())];
                self.holes.remove(&h);
                self.holes.insert(p.clone());
                self.catalysts.remove(&p);
                self.catalysts.insert(p1.clone());
            }
            else if !displaced_holes.is_empty() {
                let h = displaced_holes[rng.gen_range(0, displaced_holes.len())];
                self.holes.remove(&h);
                self.holes.insert(p.clone());
                self.catalysts.remove(&p);
                self.catalysts.insert(p1.clone());
            } else {
                self.catalysts.remove(&p);
                self.catalysts.insert(p1.clone());
            }
        }
        else {
            return *p;
        }

        assert!(self.catalysts.is_disjoint(&self.links));
        assert!(self.catalysts.is_disjoint(&self.holes));
        assert!(self.links.is_disjoint(&self.holes));
        
        p1
    }

    fn select_neighbor(&self, p: &Pos, i: i32) -> Pos {
        let n = neighbor(p, i);
        assert!(self.is_catalyst(p));
        assert!(self.is_substrate(&n));
        if i == 1 || i == 3 {
            if self.is_substrate(&neighbor(&n, 0))
                && !self.is_substrate(&neighbor(&n, 2))
            {
                neighbor(&n, 0)
            }
            else if !self.is_substrate(&neighbor(&n, 0))
                && self.is_substrate(&neighbor(&n, 2))
            {
                neighbor(&n, 2)
            }
            else
            {
                assert!(self.is_substrate(&neighbor(&n, 0)) || self.is_substrate(&neighbor(&n, 2)));
                assert!(self.is_substrate(&neighbor(&n, 0)));
                assert!(self.is_substrate(&neighbor(&n, 2)));
                if rand::random() {
                    neighbor(&n, 0)
                } else {
                    neighbor(&n, 2)
                }
            }
        }
        else {
            if self.is_substrate(&neighbor(&n, 1))
                && !self.is_substrate(&neighbor(&n, 3))
            {
                neighbor(&n, 1)
            }
            else if !self.is_substrate(&neighbor(&n, 1))
                && self.is_substrate(&neighbor(&n, 3))
            {
                neighbor(&n, 3)
            }
            else
            {
                assert!(self.is_substrate(&neighbor(&n, 1)) || self.is_substrate(&neighbor(&n, 3)));
                assert!(self.is_substrate(&neighbor(&n, 1)));
                assert!(self.is_substrate(&neighbor(&n, 3)));
                if rand::random() {
                    neighbor(&n, 1)
                } else {
                    neighbor(&n, 3)
                }
            }
        }
    }

    fn produce(&mut self, p: &Pos) {
        let mut rng = rand::thread_rng();
        let mut neighbors = vec![];
        for i in 0..4 {
            let n = neighbor(p, i);
            if i == 1 || i == 3 {
                // look above and below for a neighbor
                if self.is_substrate(&n) &&
                    (self.is_substrate(&neighbor(&n, 0))
                     || self.is_substrate(&neighbor(&n, 2)))
                {
                    assert!(self.is_substrate(&neighbor(&n, 0)) || self.is_substrate(&neighbor(&n, 2)));
                    neighbors.push(i);
                }
            }
            else {
                // look left and right for a neighbor
                if self.is_substrate(&n) &&
                    (self.is_substrate(&neighbor(&n, 1))
                     || self.is_substrate(&neighbor(&n, 3)))
                {
                    assert!(self.is_substrate(&neighbor(&n, 1)) || self.is_substrate(&neighbor(&n, 3)));
                    neighbors.push(i);
                }
            }
        }
        if neighbors.is_empty() {
            return;
        }

        let i = neighbors[rng.gen_range(0, neighbors.len())];
        let n = neighbor(p, i);
        let n1 = self.select_neighbor(p, i);

        assert!(self.is_substrate(&n));
        assert!(self.is_substrate(&n1));
        assert!(n != n1);
        
        self.holes.insert(n1);
        self.links.insert(n);
        self.bond(&n);

        assert!(self.catalysts.is_disjoint(&self.links), "produce - catalysts not disjoint from links");
        assert!(self.catalysts.is_disjoint(&self.holes), "produce - catalysts not disjoint from holes");
        assert!(self.links.is_disjoint(&self.holes), "produce - links not disjoint from holes");

    }

    /// Select a random location and update it.
    pub fn update(&mut self) {
        assert!(self.validate_bonds(), "update pre");

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
            assert!(self.num_bonds(&p) <= (0..4).into_iter()
                    .map(|i| neighbor(&p, i))
                    .filter(|&n| self.is_link(&n) && self.is_link(&n)).fold(0, |s, _| s + 1),
                    "catalyst - pre 1");

            let p = self.move_catalyst(&p);
            self.produce(&p);

            // move the catalyst and create a new link and a new hole
            assert!(self.validate_bonds(), "update catalyst - post");
            assert!(self.num_bonds(&p) <= (0..4).into_iter()
                    .map(|i| neighbor(&p, i))
                    .filter(|&n| self.is_link(&n) && self.is_link(&n)).fold(0, |s, _| s + 1), "catalyst - post 1");
            assert!(self.num_links() == self.num_holes(), "catalyst - post");
        }
        else if self.is_link(&p) {
            self.update_link(&p);
            assert!(self.validate_bonds(), "update link - post");
        }
        else if self.is_hole(&p) {
            assert!(self.num_bonds(&p) <= (0..4).into_iter()
                    .map(|i| neighbor(&p, i))
                    .filter(|&n| self.is_link(&n) && self.is_link(&n)).fold(0, |s, _| s + 1), "hole - pre 1");
            assert!(self.num_links() == self.num_holes(), "update hole - pre");
            let i = rng.gen_range(0, 4);
            let p1 = neighbor(&p, i);
            let (ul, lr) = new_bounds(&p1, &self.upper_left, &self.lower_right);
            self.upper_left = ul;
            self.lower_right = lr;

            if self.is_link(&p1) && !self.is_bonded(&p1) {
                // displace the free link.
                self.links.remove(&p1);
                self.links.insert(p.clone());
                self.holes.remove(&p);
                self.holes.insert(p1.clone());
                self.bond(&p);
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
            else if self.is_link(&p1) && self.is_bonded(&p1) {
                let n = neighbor(&p1, i);
                // if self.is_link(&n) && !self.is_bonded(&n) {
                //     self.holes.remove(&p);
                //     self.holes.insert(n.clone());
                //     self.links.remove(&n);
                //     self.links.insert(p.clone());
                // }
                // else if self.is_catalyst(&n) {
                //     self.holes.remove(&p);
                //     self.holes.insert(n.clone());
                //     self.catalysts.remove(&n);
                //     self.catalysts.insert(p.clone());
                // }
                /* else */ if self.is_substrate(&n) {
                    self.holes.remove(&p);
                    self.holes.insert(n.clone());
                }
            }
            // TODO: holes can pass through bonded links.
            assert!(self.validate_bonds(), "update hole - post 0");
            assert!(self.num_bonds(&p) <= (0..8).into_iter()
                    .map(|i| neighbor(&p, i))
                    .filter(|&n| self.is_link(&n) && self.is_link(&n)).fold(0, |s, _| s + 1), "hole - post 1");
            assert!(self.num_links() == self.num_holes(), "update hole - post 2");
        }

        assert!(self.validate_bonds(), "update post");
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


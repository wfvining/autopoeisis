/// view based on relm game of life example at github.com/juchiast/gameoflife

use ::*;
use relm::{Relm, Widget,Update}
use gtk::prelude::*;
use gtk::{Window,WindowType,DrawingArea,Button};
use std::time::Duration;
use futures_glib::Interval;

use universe::*;

struct Pos {
    x: i32,
    y: i32,
}

#[derive(Clone)]
pub struct MyModel {
    universe: Universe,
    size: Pos,
    center: Pos,
    scale: i32,
    mouse: Option<Pos>,
}

impl MyModel {
    fn new() -> Self {
        MyModel {
            universe: Universe::new(250, 250, 0.01, 2),
            size: Pos { x: 250, y: 250 },
            center: Pos { x: 0, y: 0 },
            scale: 2,
            mouse: None,
        }
    }
}

#[derive(Msg)]
pub enum MyMsg {
    Motion(((f64, f64), gdk::ModifierType)),
    Tick(()),
    Quit,
}

#[derive(Clone)]
pub struct Win {
    hbox: gdk::Box,
    area: DrawingArea,
    window: Window,
    model: MyModel,
}

impl Win {
    fn draw(&mut self, cells: &[Pos], top_left: &Pos) {
        use gdk::prelude::ContextExt;
        let cr = cairo::Context::create_from_window(&self.area.get_window().unwrap());
        cr.set_source_rgb(1.,1.,1.);
        cr.paint();
        cr.scale(self.model.scale as f64, self.model.scale as f64);
        cr.set_source_rgb(0., 0., 0.); // black?
        for pos in cells {
            cr.rectangle()
        }
    }
}

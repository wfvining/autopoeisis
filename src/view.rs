/// view based on relm game of life example at github.com/juchiast/gameoflife

use ::*;
use relm::{Relm, Widget,Update};
use gtk::prelude::*;
use gtk::{Window,WindowType,DrawingArea};
use gdk::{ModifierType, EventMask};
use std::time::Duration;
use futures_glib::Interval;

use universe::*;

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
            universe: Universe::new(10, 10, 0.05, 10),
            size: pos(250,250),
            center: pos(62,62),
            scale: 4,
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

pub struct Win {
    hbox: gtk::Box,
    area: DrawingArea,
    window: Window,
    model: MyModel,
}

impl Win {
    fn draw(&mut self, catalysts: &[Pos], holes: &[Pos],
            free_links: &[Pos], single_links: &[Pos],
            double_links: &[Pos],
            top_left: &Pos) {
        use gdk::prelude::ContextExt;
        let cr = cairo::Context::create_from_window(&self.area.get_window().unwrap());
        cr.set_source_rgb(0.3, 0.3, 0.3);
        cr.paint();
        cr.scale(self.model.scale as f64, self.model.scale as f64);

        cr.set_source_rgb(1.0, 0.0, 0.0); // red
        for pos in catalysts {
            cr.rectangle((pos.x - top_left.x) as f64, (pos.y - top_left.y) as f64, 1.0, 1.0);
        }
        cr.fill();

        cr.set_source_rgb(0.05, 0.05, 0.05); // black
        for pos in holes {
            cr.rectangle((pos.x - top_left.x) as f64, (pos.y - top_left.y) as f64, 1.0, 1.0);
        }
        cr.fill();

        cr.set_source_rgb(0.0, 1.0, 0.0); // green
        for pos in free_links {
            cr.rectangle((pos.x - top_left.x) as f64, (pos.y - top_left.y) as f64, 1.0, 1.0);
        }
        cr.fill();

        cr.set_source_rgb(0.0, 0.0, 1.0);
        for pos in single_links {
            cr.rectangle((pos.x - top_left.x) as f64, (pos.y - top_left.y) as f64, 1.0, 1.0);
        }
        cr.fill();

        cr.set_source_rgb(1.0, 1.0, 1.0);
        for pos in double_links {
            cr.rectangle((pos.x - top_left.x) as f64, (pos.y - top_left.y) as f64, 1.0, 1.0);
        }
        cr.fill();
    }
}

impl Update for Win {
    type Model = MyModel;
    type ModelParam = ();
    type Msg = MyMsg;

    fn model(_: &Relm<Self>, _: ()) -> MyModel {
        MyModel::new()
    }

    fn subscriptions(&mut self, relm: &Relm<Self>) {
        let stream = Interval::new(Duration::from_millis(1));
        relm.connect_exec_ignore_err(stream, MyMsg::Tick);
    }

    fn update(&mut self, event: MyMsg) {
        use self::MyMsg::*;
        match event {
            Tick(()) => {
                for _i in 0..100 {
                    self.model.universe.update();
                }
                let top_left = pos(self.model.center.x - self.model.size.x / 2, self.model.center.y - self.model.size.y / 2);
                let catalysts = self.model.universe.get_catalysts_in(&top_left, &self.model.size);
                let holes = self.model.universe.get_holes_in(&top_left, &self.model.size);
                let free_links = self.model.universe.get_free_links_in(&top_left, &self.model.size);
                let single_links = self.model.universe.get_single_bonded_links_in(&top_left, &self.model.size);
                let double_links = self.model.universe.get_double_bonded_links_in(&top_left, &self.model.size);
                
                self.draw(&catalysts, &holes, &free_links, &single_links, &double_links, &top_left);
            },
            Motion(((x, y), t)) => {
                let p = pos(x as i32, y as i32);
                if (t & ModifierType::BUTTON1_MASK).bits() != 0 {
                    self.model.mouse = match self.model.mouse {
                        None => Some(p),
                        Some(ref old_pos) => {
                            let new_center = pos(
                                self.model.center.x + (old_pos.x - p.x) / self.model.scale,
                                self.model.center.y + (old_pos.y - p.y) / self.model.scale
                            );
                            let x = if new_center.x != self.model.center.x {
                                p.x
                            }
                            else {
                                old_pos.x
                            };
                            
                            let y = if new_center.y != self.model.center.y {
                                p.y
                            }
                            else {
                                old_pos.y
                            };
                            
                            self.model.center = new_center;
                            Some(pos(x, y))
                        }
                    }
                } else {
                    self.model.mouse = None;
                }
            },
            Quit => gtk::main_quit(),
        }
    }
}

impl Widget for Win {
    type Root = Window;

    // Return the root widget.
    fn root(&self) -> Self::Root {
        self.window.clone()
    }

    fn view(relm: &Relm<Self>, model: Self::Model) -> Self {
        let window = Window::new(WindowType::Toplevel);
        let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        let area = DrawingArea::new();

        area.set_size_request(model.size.x * model.scale, model.size.y * model.scale);
        area.set_events(area.get_events() | EventMask::POINTER_MOTION_MASK.bits() as i32);
        area.set_events(area.get_events() | EventMask::BUTTON_PRESS_MASK.bits() as i32);

        hbox.pack_start(&area, false, false, 0);
        window.add(&hbox);
        window.set_title("Autopoeisis");
        window.show_all();

        use self::MyMsg::*;
        connect!(relm, window, connect_delete_event(_,_), return (Some(Quit), Inhibit(false)));
        connect!(relm, area, connect_motion_notify_event(_, ev), return (Some(Motion((ev.get_position(), ev.get_state()))), Inhibit(false)));

        Win {
            hbox: hbox,
            area: area,
            window: window,
            model: model,
        }
    }
}

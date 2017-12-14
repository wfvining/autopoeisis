/// Implementing the example of an autopoetic system from Varela,
/// Maturana, & Urbie (1974)

extern crate rand;
extern crate gtk;
extern crate gdk;
extern crate cairo;
#[macro_use]
extern crate relm;
#[macro_use]
extern crate relm_derive;
extern crate futures_glib;

mod universe;
mod view;

fn main() {
    std::env::set_var("GDK_BACKEND", "x11");
    relm::run::<view::Win>(()).unwrap();
}

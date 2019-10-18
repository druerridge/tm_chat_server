#![deny(warnings)]
#![deny(clippy::all)]
#![allow(clippy::needless_return)]

mod app;
mod settings;
mod chat_server;

use crate::app::App;

extern crate structopt;
#[macro_use]
extern crate structopt_derive;

fn main() {
    let app = App {};
    app.run();
}

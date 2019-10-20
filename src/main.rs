#![deny(warnings)]
#![deny(clippy::all)]
#![allow(clippy::needless_return)]

extern crate structopt;
#[macro_use]
extern crate structopt_derive;

use crate::app::App;

mod app;
mod settings;
mod chat_server;
mod commands;
mod tm_tcp_stream;

fn main() {
    let app = App {};
    app.run();
}

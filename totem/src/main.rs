#![feature(ptr_to_from_bits)]

mod app;
mod engine;
mod util;
mod widgets;
mod bytecode;

use app::App;

pub fn main() {
    pollster::block_on(App::create_and_run());
}

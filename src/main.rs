#![feature(ptr_to_from_bits)]

mod app;
mod renderer;
mod theme;
mod visuals;
mod engine;
mod util;

use app::App;

pub fn main() {
    pollster::block_on(App::create_and_run());
}

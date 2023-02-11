#![feature(slice_as_chunks)]
#![feature(ptr_to_from_bits)]

mod app;
mod engine;
mod util;
mod widgets;

use app::App;

pub fn main() {
    pollster::block_on(App::create_and_run());
}

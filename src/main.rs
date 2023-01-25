mod app;
mod renderer;
mod theme;
mod visuals;

use app::App;

pub fn main() {
    pollster::block_on(App::create_and_run());
}

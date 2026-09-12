use rust_fluid::app;

fn main() {
    pollster::block_on(app::run());
}
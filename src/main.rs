use crate::game::Game;

mod game;
mod util;
mod texture;
mod world;
mod buffer_builder;

fn main() {
    let mut game = Game::new();
    game.run();
}

use bevy::prelude::*;
use kurulus_app::plugins::register_plugins;

fn main() {
    let mut app = App::new();
    register_plugins(&mut app);
    app.run();
}

use bevy::prelude::*;

mod demo;

extern crate nalgebra as na;

fn main() {
    App::build()
        .add_default_plugins()
        .add_plugin(demo::DemoPlugin)
        .run();
}

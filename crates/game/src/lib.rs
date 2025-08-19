#![feature(duration_millis_float)]

use asset_loading::*;
use audio::*;
use avian3d::prelude::*;
use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;
use bevy_seedling::prelude::*;
use bevy_tnua::prelude::*;
use models::*;
use scene::*;
use camera::*;
use event::*;

pub mod camera;
#[cfg(feature = "dev_native")]
pub mod dev_tools;
pub mod player;
pub mod enemy;
pub mod sound;
pub mod combat;
pub mod utils;
pub mod event;

pub fn plugin(app: &mut App) {
    app.add_plugins((
        models::plugin,
        camera::plugin,
        scene::plugin,
        player::plugin,
        enemy::plugin,
        combat::plugin,
        sound::plugin,
        #[cfg(feature = "dev_native")]
        dev_tools::plugin,
        event::plugin,

        MeshPickingPlugin
    ));
}
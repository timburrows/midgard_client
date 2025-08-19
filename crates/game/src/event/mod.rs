use super::*;

pub mod types;
pub mod emitters;
pub mod triggers;

pub fn plugin(app: &mut App) {
    app.add_plugins(
        (
            emitters::plugin,
            triggers::plugin,
            types::plugin
        )
    );
}
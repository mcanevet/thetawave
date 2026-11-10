use bevy::prelude::*;
use leafwing_input_manager::prelude::InputMap;
use serde::Deserialize;
use thetawave_interface::input::{InputsResource, MainMenuExplorer, MenuAction, PlayerAction};

/// Spawns entity to track navigation over menus
pub fn spawn_menu_explorer_system(mut commands: Commands, inputs_res: Res<InputsResource>) {
    // Bevy 0.20 port: `InputManagerBundle` was removed; `InputMap` now
    // requires its own `ActionState` via required components.
    commands
        .spawn(inputs_res.menu.clone())
        .insert(MainMenuExplorer);
}

#[derive(Deserialize)]
pub struct InputBindings {
    pub menu_keyboard: Vec<(MenuAction, KeyCode)>,
    pub menu_gamepad: Vec<(MenuAction, GamepadButton)>,
    pub player_keyboard: Vec<(PlayerAction, KeyCode)>,
    pub player_gamepad: Vec<(PlayerAction, GamepadButton)>,
    pub player_mouse: Vec<(PlayerAction, MouseButton)>,
}

impl From<InputBindings> for InputsResource {
    fn from(bindings: InputBindings) -> Self {
        InputsResource {
            menu: InputMap::new(bindings.menu_keyboard)
                .insert_multiple(bindings.menu_gamepad)
                .to_owned(),
            player_keyboard: InputMap::new(bindings.player_keyboard)
                .insert_multiple(bindings.player_mouse)
                .to_owned(),
            player_gamepad: InputMap::new(bindings.player_gamepad),
        }
    }
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "playtest")))]
pub(super) fn get_input_bindings() -> InputBindings {
    use ron::from_str;
    use std::{env::current_dir, fs::read_to_string};

    let config_path = current_dir().unwrap().join("config");

    from_str::<InputBindings>(&read_to_string(config_path.join("input.ron")).unwrap()).unwrap()
}

// Battery/robot runs are driven from arbitrary cwds (and must not depend
// on `generate_config_files()` having written to disk), so bake the
// committed default in — same approach the wasm path already takes.
#[cfg(any(target_arch = "wasm32", feature = "playtest"))]
pub(super) fn get_input_bindings() -> InputBindings {
    use ron::de::from_bytes;

    from_bytes::<InputBindings>(include_bytes!("input.ron")).unwrap()
}

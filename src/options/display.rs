//! Systems to configure minor display settings.
use crate::game::GameParametersResource;
use bevy::{
    camera::Projection,
    input::ButtonInput,
    prelude::{Camera2d, Entity, KeyCode, NonSend, Query, Res, With},
    window::{MonitorSelection, PrimaryWindow, Window, WindowMode},
    winit::WinitWindows,
};
use serde::Deserialize;
use winit::window::Icon;

/// Display settings of the window
#[derive(Deserialize)]
pub struct DisplayConfig {
    /// Width of window
    pub width: f32,
    /// Height of window
    pub height: f32,
    /// True of fullsceen, false if windowed
    pub fullscreen: bool,
}

impl From<DisplayConfig> for Window {
    fn from(display_config: DisplayConfig) -> Self {
        Window {
            title: "Thetawave".to_string(),
            resolution: (display_config.width as u32, display_config.height as u32).into(),
            resizable: true,
            mode: if display_config.fullscreen {
                WindowMode::BorderlessFullscreen(MonitorSelection::Current)
            } else {
                WindowMode::Windowed
            },
            ..Default::default()
        }
    }
}

// TODO: fix this function, doesn't toggle back to windowed correctly
/// Toggles the window between full screen and windowed on key press
pub(super) fn toggle_fullscreen_system(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut window_query: Query<&mut Window, With<PrimaryWindow>>,
) {
    // get primary window
    //let window = windows.get_primary_mut().unwrap();
    let mut primary_window = window_query.single_mut().unwrap();
    // get input for toggling full screen
    let fullscreen_input = keyboard_input.just_released(KeyCode::KeyF);

    // set window mode to the mode it's not in
    if fullscreen_input {
        let new_mode = match primary_window.mode {
            WindowMode::BorderlessFullscreen(_) => {
                primary_window.set_maximized(false);
                WindowMode::Windowed
            }
            WindowMode::Windowed => {
                primary_window.set_maximized(true);
                WindowMode::BorderlessFullscreen(MonitorSelection::Current)
            }
            _ => primary_window.mode,
        };

        primary_window.mode = new_mode;
    }
}

/// Toggles a zoomed out camera perspective on key press
pub(super) fn toggle_zoom_system(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    // Bevy 0.20 port: `OrthographicProjection` is no longer a component; the
    // `Projection` enum component holds it as `Projection::Orthographic`.
    mut camera_query: Query<&mut Projection, With<Camera2d>>,
    game_parameters: Res<GameParametersResource>,
) {
    // get input for toggling zoom
    let zoom_input = keyboard_input.just_released(KeyCode::KeyV);

    // toggle zoom to opposite scale
    if zoom_input {
        for mut proj in camera_query.iter_mut() {
            if let Projection::Orthographic(orthographic) = proj.as_mut() {
                if orthographic.scale == 1.0 {
                    orthographic.scale = game_parameters.camera_zoom_out_scale;
                } else {
                    orthographic.scale = 1.0;
                }
            }
        }
    }
}

/// set the window icon. This needs to be run once, near app start up.
/// Runs as a no-op when there is no window backend (headless playtest runs).
pub(super) fn set_window_icon(
    windows: Option<NonSend<WinitWindows>>,
    window_query: Query<Entity, With<PrimaryWindow>>,
) {
    let Some(windows) = windows else {
        return;
    };
    let Ok(window_entity) = window_query.single() else {
        return;
    };
    let Some(window) = windows.get_window(window_entity) else {
        return;
    };

    // set icon using image crate
    let (icon_rgba, icon_width, icon_height) = {
        let image = image::open("assets/texture/window_icon.png")
            .expect("Failed to open icon path")
            .into_rgba8();
        let (width, height) = image.dimensions();
        let rgba = image.into_raw();
        (rgba, width, height)
    };

    let icon = Icon::from_rgba(icon_rgba, icon_width, icon_height).unwrap();

    window.set_window_icon(Some(icon));
}

use swayipc::{ Connection as SwayConnection};
use std::error::Error;
use cosmic_comp_config::input::{ ClickMethod, ScrollMethod};
use tracing::info;
pub type SwayResult<T = ()> = Result<T, Box<dyn Error>>;

// Macro to handle Sway command execution with logging
macro_rules! execute_sway_cmd {
    ($page:expr, $cmd:expr, $success_msg:expr) => {
        match $page.sway_connection().run_command($cmd.clone()) {
            Ok(_) => info!("{} via: swaymsg {}", $success_msg, &$cmd),
            Err(e) => {
                tracing::error!("Failed to execute command: {}", e);
                return Err(e.into());
            }
        }
    };
}

// pub enum Message {
//     DisableWhileTyping(bool, bool),
//     PrimaryButtonSelected(bool, bool), // Assuming first bool is left_handed, second is touchpad
//     SetAcceleration(bool, bool),
//     SetMouseSpeed(f64, bool),
//     SetNaturalScroll(bool, bool),
//     SetSecondaryClickBehavior(Option<ClickMethod>, bool),
//     SetScrollFactor(f64, bool),
//     SetScrollMethod(Option<ScrollMethod>, bool),
//     TapToClick(bool),
// }
use super::Page;

pub trait PointerMethods {
    /// Get a mutable reference to the sway connection
    fn sway_connection(&mut self) -> &mut SwayConnection;
}

impl PointerMethods for Page {
    fn sway_connection(&mut self) -> &mut SwayConnection {
        &mut self.connection
    }
}

use super::Message;

pub fn execute_sway_pointer_commands(message: &Message, page: &mut Page) -> SwayResult {
    match message {
        Message::DisableWhileTyping(disabled, touchpad) => {
            let device_type = if *touchpad { "touchpad" } else { "pointer" };
            let decision = if *disabled { "disabled" } else { "enabled" };
            let cmd = format!("input type:{} dwt {}", device_type, decision);
            execute_sway_cmd!(page, cmd, format!("Set disable-while-typing to {} for {}", decision, device_type));
        }

        Message::PrimaryButtonSelected(entity, touchpad) => {
            let device_type = if *touchpad { "touchpad" } else { "pointer" };
            let select_model = if *touchpad {
                &mut page.touchpad_primary_button
            } else {
                &mut page.primary_button
            };
            select_model.activate(*entity);
            let Some(left_entity) = select_model.entity_at(1) else {
                return Err("Could not get left entity".into());
            };
            let left_handedness = if select_model.active() == left_entity { "enabled" } else { "disabled" };
            let cmd = format!("input type:{} left_handed {}", device_type, left_handedness);
            execute_sway_cmd!(page, cmd, format!("Set left-handed mode to {} for {}", left_handedness, device_type));
        }

        Message::SetAcceleration(enabled, touchpad) => {
            let device_type = if *touchpad { "touchpad" } else { "pointer" };
            let profile = if *enabled { "adaptive" } else { "flat" };
            let cmd = format!("input type:{} accel_profile {}", device_type, profile);
            execute_sway_cmd!(page, cmd, format!("Set acceleration profile to {} for {}", profile, device_type));
        }

        Message::SetMouseSpeed(speed, touchpad) => {
            let device_type = if *touchpad { "touchpad" } else { "pointer" };
            let cmd = format!("input type:{} pointer_accel {}", device_type, speed);
            execute_sway_cmd!(page, cmd, format!("Set pointer acceleration to {} for {}", speed, device_type));
        }

        Message::SetNaturalScroll(enabled, touchpad) => {
            let device_type = if *touchpad { "touchpad" } else { "pointer" };
            let setting = if *enabled { "enabled" } else { "disabled" };
            let cmd = format!("input type:{} natural_scroll {}", device_type, setting);
            execute_sway_cmd!(page, cmd, format!("Set natural scroll to {} for {}", setting, device_type));
        }

        Message::SetSecondaryClickBehavior(method, touchpad) => {
            let device_type = if *touchpad { "touchpad" } else { "pointer" };
            let click_method = match method {
                Some(ClickMethod::ButtonAreas) => "button_areas",
                Some(ClickMethod::Clickfinger) => "clickfinger",
                _ => "none",
            };
            let cmd = format!("input type:{} click_method {}", device_type, click_method);
            execute_sway_cmd!(page, cmd, format!("Set click method to {} for {}", click_method, device_type));
        }

        Message::SetScrollFactor(factor, touchpad) => {
            let device_type = if *touchpad { "touchpad" } else { "pointer" };
            let cmd = format!("input type:{} scroll_factor {}", device_type, factor);
            execute_sway_cmd!(page, cmd, format!("Set scroll factor to {} for {}", factor, device_type));
        }

        Message::SetScrollMethod(method, touchpad) => {
            let device_type = if *touchpad { "touchpad" } else { "pointer" };
            let scroll_method = match method {
                Some(ScrollMethod::TwoFinger) => "two_finger",
                Some(ScrollMethod::Edge) => "edge",
                Some(ScrollMethod::OnButtonDown) => "on_button_down",
                Some(ScrollMethod::NoScroll) => "none",
                _ => "none",
            };
            let cmd = format!("input type:{} scroll_method {}", device_type, scroll_method);
            execute_sway_cmd!(page, cmd, format!("Set scroll method to {} for {}", scroll_method, device_type));
        }

        Message::TapToClick(enabled) => {
            let setting = if *enabled { "enabled" } else { "disabled" };
            let cmd = format!("input type:touchpad tap {}", setting);
            execute_sway_cmd!(page, cmd, format!("Set tap-to-click to {} for touchpad", setting));
        }
    }
    Ok(())
}
   
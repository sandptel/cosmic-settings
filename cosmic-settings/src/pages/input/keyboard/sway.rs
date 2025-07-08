use swayipc::Connection as SwayConnection;
use std::error::Error;
use cosmic_comp_config::NumlockState;
use crate::pages::input::keyboard::Context;
pub type SwayResult<T = ()> = Result<T, Box<dyn Error>>;
use tracing::info;
use super::{Message, Page, SpecialKey};

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

pub trait KeyboardMethods {
    /// Gets a mutable reference to the sway connection
    fn sway_connection(&mut self) -> &mut SwayConnection;
}

impl KeyboardMethods for Page {
    fn sway_connection(&mut self) -> &mut SwayConnection {
        &mut self.connection
    }
}

pub fn execute_sway_keyboard_commands(message: &Message, page: &mut Page) -> SwayResult {
    match message {
        Message::SetRepeatKeysDelay(delay) => {
            let cmd = format!("input type:keyboard repeat_delay {}", delay);
            execute_sway_cmd!(page, cmd, format!("Set keyboard repeat delay to {}ms", delay));
        }

        Message::SetRepeatKeysRate(rate) => {
            let cmd = format!("input type:keyboard repeat_rate {}", rate);
            execute_sway_cmd!(page, cmd, format!("Set keyboard repeat rate to {} keys/second", rate));
        }

        Message::SetNumlockState(_numlock_state) => {
            // Note: xkb_numlock can only be set in config file, not at runtime
            // This is a limitation of Sway - numlock settings require restart
            // We'll log this but can't apply it immediately via swaymsg
        }

        Message::SpecialCharacterSelect(option) => {
            if let Some(Context::SpecialCharacter(special_key)) = &page.context {
                let special_key_clone = *special_key; // Clone the value to avoid borrowing issues
                // Build new options string
                let current_options = page.xkb.options.as_deref().unwrap_or_default();
                let prefix = special_key.prefix();
                
                // Remove existing options with same prefix and add new one if specified
                let new_options: Vec<&str> = current_options
                    .split(',')
                    .filter(|opt| !opt.starts_with(prefix))
                    .chain(option.iter().copied())
                    .filter(|opt| !opt.is_empty())
                    .collect();
                let options_string = new_options.join(",");
                let cmd = if options_string.is_empty() {
                    "input type:keyboard xkb_options \"\"".to_string()
                } else {
                    format!("input type:keyboard xkb_options \"{}\"", options_string)
                };
                execute_sway_cmd!(page, cmd, format!("Updated XKB options for {:?}", special_key_clone));
            }
        }

        Message::SourceAdd(_id) => {
            // Build complete layout string from all active layouts
            update_sway_layouts(page)?;
        }

        Message::SourceContext(source_context) => {
            use super::SourceContext;
            match source_context {
                SourceContext::Remove(_id) | 
                SourceContext::MoveUp(_id) | 
                SourceContext::MoveDown(_id) => {
                    // Update Sway with current layout configuration after UI changes
                    update_sway_layouts(page)?;
                }
                SourceContext::Settings(_id) | 
                SourceContext::ViewLayout(_id) => {
                    // These don't need Sway commands
                }
            }
        }

        // These messages don't have direct Sway equivalents, so we'll skip them
        Message::ExpandInputSourcePopover(_) |
        Message::InputSourceSearch(_) |
        Message::OpenSpecialCharacterContext(_) |
        Message::OpenNumlockContext |
        Message::ShowInputSourcesContext |
        Message::SetShowExtendedInputSources(_) => {
            // These are UI-only messages that don't need Sway commands
        }
    }
    Ok(())
}

fn update_sway_layouts(page: &mut Page) -> SwayResult {
    // Build layout and variant strings for all active layouts
    let mut layouts = Vec::new();
    let mut variants = Vec::new();
    
    // Collect layout data first to avoid borrowing conflicts
    for layout_id in &page.active_layouts {
        if let Some((locale, variant, _, _)) = page.keyboard_layouts.get(*layout_id) {
            layouts.push(locale.clone());
            variants.push(variant.clone());
        }
    }
    
    if !layouts.is_empty() {
        let layout_string = layouts.join(",");
        let variant_string = variants.join(",");
        
        // Set layouts
        let layout_cmd = format!("input type:keyboard xkb_layout \"{}\"", layout_string);
        execute_sway_cmd!(page, layout_cmd, format!("Updated keyboard layouts to [{}]", layout_string));
        
        // Set variants if any non-empty variants exist
        if variants.iter().any(|v| !v.is_empty()) {
            let variant_cmd = format!("input type:keyboard xkb_variant \"{}\"", variant_string);
            execute_sway_cmd!(page, variant_cmd, format!("Updated keyboard variants to [{}]", variant_string));
        }
    }
    
    Ok(())
}



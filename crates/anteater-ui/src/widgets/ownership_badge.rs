//! Ownership state badge widget
//!
//! Displays ownership state with consistent visual language

use anteater_ui_types::*;
use egui::{Color32, RichText};

/// Render an ownership state badge
pub fn ownership_badge(ui: &mut egui::Ui, state: &OwnershipState) {
    let (bg_color, text_color, label) = ownership_colors(state);

    let text = RichText::new(label)
        .color(text_color)
        .small();

    ui.label(text);
}

/// Get colors for an ownership state (following VISUAL_LANGUAGE.md)
fn ownership_colors(state: &OwnershipState) -> (Color32, Color32, &'static str) {
    match state {
        OwnershipState::Owned => {
            (Color32::TRANSPARENT, Color32::DARK_GREEN, "")
        }
        OwnershipState::MovedFrom { .. } => {
            (Color32::from_gray(250), Color32::from_gray(158), "moved")
        }
        OwnershipState::Borrowed { .. } => {
            (Color32::from_rgb(227, 242, 253), Color32::from_rgb(21, 101, 192), "&")
        }
        OwnershipState::MutablyBorrowed { .. } => {
            (Color32::from_rgb(255, 243, 224), Color32::from_rgb(230, 81, 0), "&mut")
        }
        OwnershipState::Dropped => {
            (Color32::from_gray(250), Color32::from_gray(117), "dropped")
        }
        OwnershipState::Uninitialized => {
            (Color32::from_gray(250), Color32::from_gray(189), "—")
        }
        OwnershipState::PartiallyMoved { .. } => {
            (Color32::from_rgb(255, 248, 225), Color32::from_rgb(255, 143, 0), "partial")
        }
        OwnershipState::Unknown { .. } => {
            (Color32::from_gray(250), Color32::from_gray(158), "?")
        }
    }
}

/// Whether to use strikethrough for this ownership state
pub fn should_strikethrough(state: &OwnershipState) -> bool {
    matches!(
        state,
        OwnershipState::MovedFrom { .. } | OwnershipState::Dropped
    )
}

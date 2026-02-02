//! Type display widget
//!
//! Renders Rust type names with appropriate formatting

use anteater_ui_types::TypeDisplay;
use egui::RichText;

/// Render a type name
pub fn type_display(ui: &mut egui::Ui, ty: &TypeDisplay) {
    let text = RichText::new(ty.display())
        .family(egui::FontFamily::Monospace)
        .color(ui.style().visuals.weak_text_color());

    ui.label(text);
}

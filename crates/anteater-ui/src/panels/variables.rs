//! Variable inspector panel - displays variables with ownership state

use super::Panel;
use crate::mock::MockDebugSession;
use crate::widgets::{ownership_badge, type_display};
use anteater_ui_types::*;
use egui::{Color32, RichText};

pub struct VariablesPanel {
    expanded: std::collections::HashSet<String>,
}

impl VariablesPanel {
    pub fn new() -> Self {
        Self {
            expanded: std::collections::HashSet::new(),
        }
    }
}

impl Default for VariablesPanel {
    fn default() -> Self {
        Self::new()
    }
}

impl Panel for VariablesPanel {
    fn id(&self) -> &'static str {
        "variables"
    }

    fn title(&self) -> &str {
        "Variables"
    }

    fn ui(&mut self, ui: &mut egui::Ui, session: &MockDebugSession) {
        // For now, show variables for frame 0
        let variables = session.variables(FrameId(0));

        egui::ScrollArea::vertical().show(ui, |ui| {
            for var in variables {
                self.render_variable(ui, var, 0, String::new());
            }
        });
    }
}

impl VariablesPanel {
    fn render_variable(&mut self, ui: &mut egui::Ui, var: &Variable, depth: usize, path: String) {
        let full_path = if path.is_empty() {
            var.name.clone()
        } else {
            format!("{}.{}", path, var.name)
        };

        ui.horizontal(|ui| {
            // Indentation for nested variables
            ui.add_space(depth as f32 * 16.0);

            // Expand/collapse button for variables with children
            if !var.children.is_empty() {
                let is_expanded = self.expanded.contains(&full_path);
                let button_text = if is_expanded { "▼" } else { "▶" };
                if ui.small_button(button_text).clicked() {
                    if is_expanded {
                        self.expanded.remove(&full_path);
                    } else {
                        self.expanded.insert(full_path.clone());
                    }
                }
            } else {
                ui.add_space(20.0); // Spacing for alignment
            }

            // Variable name (with strikethrough if moved/dropped)
            let name_text = if should_strikethrough(&var.ownership) {
                RichText::new(&var.name)
                    .strikethrough()
                    .color(Color32::GRAY)
            } else if var.is_optimized_out {
                RichText::new(&var.name)
                    .strikethrough()
                    .color(Color32::GRAY)
            } else {
                RichText::new(&var.name)
            };
            ui.label(name_text);

            // Ownership badge
            if !matches!(var.ownership, OwnershipState::Owned) || var.is_optimized_out {
                ownership_badge(ui, &var.ownership);
            }

            // Type
            type_display(ui, &var.rust_type);

            // Value
            let value_text = if var.is_optimized_out {
                RichText::new("(optimized out)")
                    .italics()
                    .color(Color32::GRAY)
            } else {
                RichText::new(var.value.display()).family(egui::FontFamily::Monospace)
            };
            ui.label(value_text);

            // Address (if available)
            if let Some(addr) = var.address {
                ui.label(
                    RichText::new(format!("@ 0x{:x}", addr))
                        .small()
                        .weak(),
                );
            }
        });

        // Render children if expanded
        if !var.children.is_empty() && self.expanded.contains(&full_path) {
            for child in &var.children {
                self.render_variable(ui, child, depth + 1, full_path.clone());
            }
        }
    }
}

fn should_strikethrough(state: &OwnershipState) -> bool {
    matches!(
        state,
        OwnershipState::MovedFrom { .. } | OwnershipState::Dropped
    )
}

//! Breakpoint management panel

use super::Panel;
use crate::mock::MockDebugSession;
use anteater_ui_types::*;
use egui::RichText;

pub struct BreakpointsPanel;

impl BreakpointsPanel {
    pub fn new() -> Self {
        Self
    }
}

impl Default for BreakpointsPanel {
    fn default() -> Self {
        Self::new()
    }
}

impl Panel for BreakpointsPanel {
    fn id(&self) -> &'static str {
        "breakpoints"
    }

    fn title(&self) -> &str {
        "Breakpoints"
    }

    fn ui(&mut self, ui: &mut egui::Ui, session: &MockDebugSession) {
        let breakpoints = session.breakpoints();

        egui::ScrollArea::vertical().show(ui, |ui| {
            for bp in breakpoints {
                ui.horizontal(|ui| {
                    // Enable/disable checkbox
                    let mut enabled = bp.enabled;
                    let checkbox_response = ui.checkbox(&mut enabled, "");

                    if checkbox_response.changed() {
                        println!("Breakpoint {} {} (mock - no backend connected)",
                            bp.id.0,
                            if enabled { "enabled" } else { "disabled" }
                        );
                    }

                    checkbox_response.on_hover_text("Toggle breakpoint (will work when backend is connected)");

                    // Location
                    let location_text = match &bp.location {
                        BreakpointLocation::Source { location } => {
                            format!("{}:{}", location.path.0.display(), location.line)
                        }
                        BreakpointLocation::Function { name } => {
                            format!("fn {}", name)
                        }
                        BreakpointLocation::Address { address } => {
                            format!("0x{:x}", address)
                        }
                    };
                    ui.label(RichText::new(location_text).family(egui::FontFamily::Monospace));

                    // Condition if present
                    if let Some(condition) = &bp.condition {
                        ui.label(
                            RichText::new(format!("if {}", condition))
                                .small()
                                .weak(),
                        );
                    }

                    // Hit count
                    if bp.hit_count > 0 {
                        ui.label(
                            RichText::new(format!("hits: {}", bp.hit_count))
                                .small()
                                .weak(),
                        );
                    }
                });
                ui.separator();
            }
        });
    }
}

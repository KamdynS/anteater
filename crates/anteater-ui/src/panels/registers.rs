//! Register panel - displays CPU registers

use super::Panel;
use crate::mock::MockDebugSession;
use egui::{Color32, RichText};

pub struct RegistersPanel;

impl RegistersPanel {
    pub fn new() -> Self {
        Self
    }
}

impl Default for RegistersPanel {
    fn default() -> Self {
        Self::new()
    }
}

impl Panel for RegistersPanel {
    fn id(&self) -> &'static str {
        "registers"
    }

    fn title(&self) -> &str {
        "Registers"
    }

    fn ui(&mut self, ui: &mut egui::Ui, session: &MockDebugSession) {
        let registers = session.registers();

        // General purpose registers
        if !registers.general.is_empty() {
            ui.heading("General");
            ui.separator();

            egui::Grid::new("registers_general")
                .striped(true)
                .num_columns(3)
                .show(ui, |ui| {
                    ui.label(RichText::new("Name").strong());
                    ui.label(RichText::new("Hex").strong());
                    ui.label(RichText::new("Decimal").strong());
                    ui.end_row();

                    for reg in &registers.general {
                        // Highlight changed registers
                        let name_text = if reg.changed {
                            RichText::new(&reg.name)
                                .color(Color32::from_rgb(230, 126, 34))
                                .strong()
                        } else {
                            RichText::new(&reg.name)
                        };

                        ui.label(name_text);
                        ui.label(
                            RichText::new(format!("0x{:016x}", reg.value))
                                .family(egui::FontFamily::Monospace),
                        );
                        ui.label(
                            RichText::new(format!("{}", reg.value))
                                .family(egui::FontFamily::Monospace),
                        );
                        ui.end_row();
                    }
                });

            ui.add_space(8.0);
        }

        // Special registers (like flags)
        if !registers.special.is_empty() {
            ui.heading("Special");
            ui.separator();

            for reg in &registers.special {
                ui.horizontal(|ui| {
                    ui.label(RichText::new(&reg.name).strong());
                    ui.label(
                        RichText::new(format!("0x{:08x}", reg.value))
                            .family(egui::FontFamily::Monospace),
                    );
                });

                // Show flags breakdown if available
                if let Some(flags) = &reg.flags {
                    ui.indent("flags", |ui| {
                        for (flag_name, flag_value) in flags {
                            ui.horizontal(|ui| {
                                ui.label(flag_name);
                                ui.label(if *flag_value { "1" } else { "0" });
                            });
                        }
                    });
                }
            }
        }
    }
}

//! Disassembly panel - shows machine code

use super::Panel;
use crate::mock::MockDebugSession;
use egui::RichText;

pub struct DisassemblyPanel;

impl DisassemblyPanel {
    pub fn new() -> Self {
        Self
    }
}

impl Default for DisassemblyPanel {
    fn default() -> Self {
        Self::new()
    }
}

impl Panel for DisassemblyPanel {
    fn id(&self) -> &'static str {
        "disassembly"
    }

    fn title(&self) -> &str {
        "Disassembly"
    }

    fn ui(&mut self, ui: &mut egui::Ui, session: &MockDebugSession) {
        let disasm = session.disassembly(0x401230..0x40123c);

        egui::ScrollArea::vertical().show(ui, |ui| {
            egui::Grid::new("disassembly_grid")
                .striped(true)
                .num_columns(3)
                .show(ui, |ui| {
                    for line in &disasm {
                        // Address
                        let addr_text = RichText::new(format!("0x{:08x}", line.address))
                            .family(egui::FontFamily::Monospace);
                        ui.label(addr_text);

                        // Bytes
                        let bytes_str = line
                            .bytes
                            .iter()
                            .map(|b| format!("{:02x}", b))
                            .collect::<Vec<_>>()
                            .join(" ");
                        ui.label(
                            RichText::new(bytes_str)
                                .family(egui::FontFamily::Monospace)
                                .weak(),
                        );

                        // Instruction (highlight current)
                        let instr_text = if line.is_current {
                            RichText::new(&line.instruction)
                                .family(egui::FontFamily::Monospace)
                                .strong()
                        } else {
                            RichText::new(&line.instruction).family(egui::FontFamily::Monospace)
                        };
                        ui.label(instr_text);

                        ui.end_row();
                    }
                });
        });
    }
}

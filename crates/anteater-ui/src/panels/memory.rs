//! Memory view panel - hex dump with ASCII sidebar

use super::Panel;
use crate::mock::MockDebugSession;
use egui::RichText;

pub struct MemoryPanel {
    /// Starting address for the view
    address: u64,
    /// Bytes per row (typically 16)
    bytes_per_row: usize,
    /// Number of rows to display
    visible_rows: usize,
}

impl MemoryPanel {
    pub fn new() -> Self {
        Self {
            address: 0x0, // Start at beginning of mock memory
            bytes_per_row: 16,
            visible_rows: 32,
        }
    }

    fn render_hex_dump(&self, ui: &mut egui::Ui, session: &MockDebugSession) {
        let total_bytes = self.bytes_per_row * self.visible_rows;
        let end_addr = self.address + total_bytes as u64;

        if let Some(data) = session.memory(self.address..end_addr) {
            egui::Grid::new("memory_hex_grid")
                .num_columns(4)
                .striped(true)
                .spacing([8.0, 2.0])
                .show(ui, |ui| {
                    // Header row
                    ui.label(RichText::new("Address").strong());
                    ui.label(RichText::new("Hex").strong());
                    ui.label(RichText::new("").strong()); // Separator column
                    ui.label(RichText::new("ASCII").strong());
                    ui.end_row();

                    // Data rows
                    for (row_idx, chunk) in data.chunks(self.bytes_per_row).enumerate() {
                        let row_addr = self.address + (row_idx * self.bytes_per_row) as u64;

                        // Address column
                        ui.label(
                            RichText::new(format!("0x{:08x}", row_addr))
                                .family(egui::FontFamily::Monospace)
                                .color(ui.style().visuals.weak_text_color()),
                        );

                        // Hex column - split into groups of 8
                        ui.horizontal(|ui| {
                            for (i, byte) in chunk.iter().enumerate() {
                                if i == 8 {
                                    ui.add_space(8.0); // Extra space in the middle
                                }
                                ui.label(
                                    RichText::new(format!("{:02x}", byte))
                                        .family(egui::FontFamily::Monospace),
                                );
                            }
                            // Pad if less than 16 bytes
                            for i in chunk.len()..self.bytes_per_row {
                                if i == 8 {
                                    ui.add_space(8.0);
                                }
                                ui.label(
                                    RichText::new("  ")
                                        .family(egui::FontFamily::Monospace),
                                );
                            }
                        });

                        // Separator
                        ui.label("|");

                        // ASCII column
                        let ascii_str: String = chunk
                            .iter()
                            .map(|&b| {
                                if b.is_ascii_graphic() || b == b' ' {
                                    b as char
                                } else {
                                    '.'
                                }
                            })
                            .collect();

                        ui.label(
                            RichText::new(ascii_str)
                                .family(egui::FontFamily::Monospace)
                                .color(ui.style().visuals.text_color()),
                        );

                        ui.end_row();
                    }
                });
        } else {
            ui.label("Memory read failed or address out of range");
        }
    }
}

impl Default for MemoryPanel {
    fn default() -> Self {
        Self::new()
    }
}

impl Panel for MemoryPanel {
    fn id(&self) -> &'static str {
        "memory"
    }

    fn title(&self) -> &str {
        "Memory"
    }

    fn ui(&mut self, ui: &mut egui::Ui, session: &MockDebugSession) {
        // Address input
        ui.horizontal(|ui| {
            ui.label("Address:");
            let mut addr_str = format!("0x{:x}", self.address);
            if ui.text_edit_singleline(&mut addr_str).changed() {
                if let Ok(addr) = u64::from_str_radix(addr_str.trim_start_matches("0x"), 16) {
                    self.address = addr;
                }
            }

            ui.separator();

            if ui.button("⬆").clicked() && self.address >= self.bytes_per_row as u64 {
                self.address -= self.bytes_per_row as u64;
            }
            if ui.button("⬇").clicked() {
                self.address += self.bytes_per_row as u64;
            }
        });

        ui.separator();

        // Hex dump view with scrolling
        egui::ScrollArea::vertical()
            .id_salt("memory_scroll")
            .show(ui, |ui| {
                self.render_hex_dump(ui, session);
            });
    }
}

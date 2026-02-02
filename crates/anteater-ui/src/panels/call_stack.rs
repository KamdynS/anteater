//! Call stack panel - displays stack frames

use super::Panel;
use crate::mock::MockDebugSession;
use anteater_ui_types::FrameId;
use egui::RichText;

pub struct CallStackPanel {
    selected_frame: Option<FrameId>,
}

impl CallStackPanel {
    pub fn new() -> Self {
        Self {
            selected_frame: Some(FrameId(0)), // Default to innermost frame
        }
    }

    pub fn selected_frame(&self) -> Option<FrameId> {
        self.selected_frame
    }
}

impl Default for CallStackPanel {
    fn default() -> Self {
        Self::new()
    }
}

impl Panel for CallStackPanel {
    fn id(&self) -> &'static str {
        "call_stack"
    }

    fn title(&self) -> &str {
        "Call Stack"
    }

    fn ui(&mut self, ui: &mut egui::Ui, session: &MockDebugSession) {
        let frames = session.stack_frames();

        egui::ScrollArea::vertical().show(ui, |ui| {
            for (idx, frame) in frames.iter().enumerate() {
                let is_selected = self.selected_frame == Some(frame.id);

                let response = ui.selectable_label(is_selected, "");

                if response.clicked() {
                    self.selected_frame = Some(frame.id);
                }

                // Draw frame content next to selection
                ui.horizontal(|ui| {
                    // Frame number
                    ui.label(RichText::new(format!("#{}", idx)).weak());

                    // Function name
                    let func_name = if frame.is_inlined {
                        RichText::new(&frame.function_name).italics()
                    } else {
                        RichText::new(&frame.function_name)
                    };
                    ui.label(func_name);

                    // Source location if available
                    if let Some(loc) = &frame.source_location {
                        ui.label(
                            RichText::new(format!(
                                "{}:{}",
                                loc.path.0.display(),
                                loc.line
                            ))
                            .weak()
                            .small(),
                        );
                    }

                    // Module badge for non-user code
                    if let Some(module) = &frame.module {
                        if module == "std" || module.starts_with("std::") {
                            ui.label(
                                RichText::new("[std]")
                                    .small()
                                    .color(ui.style().visuals.weak_text_color()),
                            );
                        }
                    }
                });

                ui.separator();
            }
        });
    }
}

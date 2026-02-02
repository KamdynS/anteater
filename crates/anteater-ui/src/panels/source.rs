//! Source code panel - displays source with syntax highlighting

use super::Panel;
use crate::mock::MockDebugSession;
use anteater_ui_types::*;
use egui::{Color32, RichText};
use syntect::easy::HighlightLines;
use syntect::highlighting::{Style, ThemeSet};
use syntect::parsing::SyntaxSet;

pub struct SourcePanel {
    syntax_set: SyntaxSet,
    theme_set: ThemeSet,
    current_file: Option<SourcePath>,

    /// Cached syntax highlighting results
    /// Key: (file_path, file_content_hash)
    /// Value: Vec of highlighted lines
    highlight_cache: std::collections::HashMap<(SourcePath, u64), Vec<Vec<(Style, String)>>>,
}

impl SourcePanel {
    pub fn new() -> Self {
        Self {
            syntax_set: SyntaxSet::load_defaults_newlines(),
            theme_set: ThemeSet::load_defaults(),
            current_file: None,
            highlight_cache: std::collections::HashMap::new(),
        }
    }

    fn render_source(&mut self, ui: &mut egui::Ui, session: &MockDebugSession) {
        let current_location = session.current_source_location();

        // Determine which file to show
        let file_path = current_location.as_ref().map(|l| &l.path);
        if let Some(path) = file_path {
            if let Some(source_file) = session.source_file(path) {
                self.current_file = Some(path.clone());
                self.render_source_lines(ui, &source_file, current_location.as_ref());
            } else {
                ui.label("Source file not available");
            }
        } else {
            ui.label("No source location");
        }
    }

    fn render_source_lines(
        &mut self,
        ui: &mut egui::Ui,
        source_file: &SourceFile,
        current_location: Option<&SourceLocation>,
    ) {
        // Compute hash of file content for cache key
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        for line in &source_file.lines {
            line.hash(&mut hasher);
        }
        let content_hash = hasher.finish();

        let cache_key = (source_file.path.clone(), content_hash);

        // Get or compute highlighted lines
        let highlighted_lines = self.highlight_cache.entry(cache_key).or_insert_with(|| {
            // Cache miss - compute syntax highlighting
            let syntax = self
                .syntax_set
                .find_syntax_by_extension("rs")
                .unwrap_or_else(|| self.syntax_set.find_syntax_plain_text());

            let theme = &self.theme_set.themes["base16-ocean.dark"];
            let mut highlighter = HighlightLines::new(syntax, theme);

            source_file
                .lines
                .iter()
                .map(|line| {
                    highlighter
                        .highlight_line(line, &self.syntax_set)
                        .unwrap_or_default()
                        .into_iter()
                        .map(|(style, text)| (style, text.to_string()))
                        .collect()
                })
                .collect()
        });

        let current_line = current_location.map(|l| l.line);
        let breakpoint_lines: std::collections::HashSet<_> =
            source_file.breakpoints.iter().copied().collect();

        egui::Grid::new("source_grid")
            .num_columns(3)
            .spacing([4.0, 0.0])
            .striped(false)
            .show(ui, |ui| {
                for (line_num, highlighted) in highlighted_lines.iter().enumerate() {
                    let line_num_1based = (line_num + 1) as u32;
                    let is_current_line = current_line == Some(line_num_1based);
                    let has_breakpoint = breakpoint_lines.contains(&line_num_1based);

                    // Background for current line
                    if is_current_line {
                        let rect = ui.available_rect_before_wrap();
                        ui.painter().rect_filled(
                            rect,
                            0.0,
                            Color32::from_rgba_premultiplied(100, 100, 50, 40),
                        );
                    }

                    // Gutter (line number + breakpoint indicator)
                    ui.horizontal(|ui| {
                        ui.set_min_width(60.0);

                        // Breakpoint indicator
                        if has_breakpoint {
                            ui.label(RichText::new("●").color(Color32::from_rgb(200, 50, 50)));
                        } else {
                            ui.label(" ");
                        }

                        // Line number
                        let line_num_text = if is_current_line {
                            RichText::new(format!("{:4}", line_num_1based))
                                .color(Color32::from_rgb(255, 200, 100))
                                .strong()
                        } else {
                            RichText::new(format!("{:4}", line_num_1based))
                                .color(ui.style().visuals.weak_text_color())
                        };
                        ui.label(line_num_text.family(egui::FontFamily::Monospace));
                    });

                    // Current line indicator
                    if is_current_line {
                        ui.label(RichText::new("→").color(Color32::from_rgb(255, 200, 100)));
                    } else {
                        ui.label(" ");
                    }

                    // Source code with syntax highlighting (from cache)
                    ui.horizontal(|ui| {
                        for (style, text) in highlighted {
                            let color = style_to_color32(*style);
                            ui.label(
                                RichText::new(text)
                                    .family(egui::FontFamily::Monospace)
                                    .color(color),
                            );
                        }
                    });

                    ui.end_row();
                }
            });
    }
}

impl Default for SourcePanel {
    fn default() -> Self {
        Self::new()
    }
}

impl Panel for SourcePanel {
    fn id(&self) -> &'static str {
        "source"
    }

    fn title(&self) -> &str {
        "Source"
    }

    fn ui(&mut self, ui: &mut egui::Ui, session: &MockDebugSession) {
        // File info header
        if let Some(location) = session.current_source_location() {
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(format!("{}", location.path.0.display()))
                        .strong(),
                );
                ui.label(format!("Line {}", location.line));
            });
            ui.separator();
        }

        // Source code view with scrolling
        egui::ScrollArea::vertical()
            .id_salt("source_scroll")
            .show(ui, |ui| {
                self.render_source(ui, session);
            });
    }
}

/// Convert syntect Style to egui Color32
fn style_to_color32(style: Style) -> Color32 {
    Color32::from_rgba_premultiplied(
        style.foreground.r,
        style.foreground.g,
        style.foreground.b,
        style.foreground.a,
    )
}

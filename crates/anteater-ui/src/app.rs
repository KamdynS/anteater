//! Main application structure with docking panel system

use crate::mock::MockDebugSession;
use crate::panels::*;
use crate::ITerm2Theme;
use egui_dock::{DockArea, DockState, Style};

/// Main Anteater application
pub struct AnteaterApp {
    /// Mock debug session for UI development
    session: MockDebugSession,

    /// Docking state - manages panel layout
    dock_state: DockState<PanelType>,

    /// Panel instances (persistent across frames)
    panels: PanelInstances,

    /// Current theme
    current_theme: ITerm2Theme,

    /// Available themes
    themes: Vec<ITerm2Theme>,

    /// Show theme selector dialog
    show_theme_selector: bool,

    /// Performance overlay (FPS counter, frame time)
    perf_overlay: PerformanceOverlay,
}

/// Performance monitoring overlay
struct PerformanceOverlay {
    /// Whether the overlay is visible
    visible: bool,

    /// Frame times for averaging (rolling window)
    frame_times: std::collections::VecDeque<f32>,

    /// Maximum number of samples to keep
    max_samples: usize,

    /// Last frame time
    last_frame_time: std::time::Instant,
}

impl PerformanceOverlay {
    fn new() -> Self {
        Self {
            visible: false,
            frame_times: std::collections::VecDeque::with_capacity(120),
            max_samples: 120, // 2 seconds at 60fps
            last_frame_time: std::time::Instant::now(),
        }
    }

    fn toggle(&mut self) {
        self.visible = !self.visible;
    }

    fn update(&mut self) {
        let now = std::time::Instant::now();
        let frame_time = now.duration_since(self.last_frame_time).as_secs_f32();
        self.last_frame_time = now;

        self.frame_times.push_back(frame_time);
        if self.frame_times.len() > self.max_samples {
            self.frame_times.pop_front();
        }
    }

    fn fps(&self) -> f32 {
        if self.frame_times.is_empty() {
            return 0.0;
        }
        let avg_frame_time: f32 = self.frame_times.iter().sum::<f32>() / self.frame_times.len() as f32;
        if avg_frame_time > 0.0 {
            1.0 / avg_frame_time
        } else {
            0.0
        }
    }

    fn avg_frame_time_ms(&self) -> f32 {
        if self.frame_times.is_empty() {
            return 0.0;
        }
        let avg = self.frame_times.iter().sum::<f32>() / self.frame_times.len() as f32;
        avg * 1000.0 // Convert to milliseconds
    }

    fn min_frame_time_ms(&self) -> f32 {
        self.frame_times.iter().copied().min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap_or(0.0) * 1000.0
    }

    fn max_frame_time_ms(&self) -> f32 {
        self.frame_times.iter().copied().max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap_or(0.0) * 1000.0
    }

    fn render(&self, ctx: &egui::Context) {
        if !self.visible {
            return;
        }

        egui::Window::new("Performance")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::RIGHT_TOP, [-10.0, 10.0])
            .show(ctx, |ui| {
                ui.heading("Performance Metrics");
                ui.add_space(4.0);

                ui.horizontal(|ui| {
                    ui.label("FPS:");
                    let fps = self.fps();
                    let color = if fps >= 55.0 {
                        egui::Color32::GREEN
                    } else if fps >= 30.0 {
                        egui::Color32::YELLOW
                    } else {
                        egui::Color32::RED
                    };
                    ui.colored_label(color, format!("{:.1}", fps));
                });

                ui.horizontal(|ui| {
                    ui.label("Frame Time (avg):");
                    ui.label(format!("{:.2} ms", self.avg_frame_time_ms()));
                });

                ui.horizontal(|ui| {
                    ui.label("Frame Time (min):");
                    ui.label(format!("{:.2} ms", self.min_frame_time_ms()));
                });

                ui.horizontal(|ui| {
                    ui.label("Frame Time (max):");
                    ui.label(format!("{:.2} ms", self.max_frame_time_ms()));
                });

                ui.add_space(4.0);
                ui.separator();
                ui.add_space(4.0);

                // Memory usage (approximate)
                ui.horizontal(|ui| {
                    ui.label("Memory:");
                    ui.label(format!("{:.1} MB", ctx.memory(|mem| mem.data.len()) as f32 / 1024.0 / 1024.0));
                });

                ui.add_space(4.0);
                ui.separator();
                ui.add_space(4.0);

                // Frame time graph
                ui.label("Frame Time Graph:");
                let frame_times: Vec<f64> = self.frame_times.iter().map(|&t| (t * 1000.0) as f64).collect();

                use egui_plot::{Line, Plot, PlotPoints};
                let points: PlotPoints = PlotPoints::from_iter(
                    frame_times.iter().enumerate().map(|(i, &y)| [i as f64, y])
                );
                let line = Line::new(points);

                Plot::new("frame_time_plot")
                    .height(80.0)
                    .include_y(0.0)
                    .include_y(20.0)
                    .allow_zoom(false)
                    .allow_drag(false)
                    .allow_scroll(false)
                    .show_x(false)
                    .show_y(true)
                    .show(ui, |plot_ui| {
                        plot_ui.line(line);
                        // Add target frame time line (16.67ms for 60fps)
                        let target_line = Line::new(PlotPoints::from_iter(
                            vec![[0.0, 16.67], [frame_times.len() as f64, 16.67]]
                        )).color(egui::Color32::from_rgb(100, 200, 100));
                        plot_ui.line(target_line);
                    });

                ui.add_space(4.0);
                ui.label("💡 Press F12 to toggle this overlay");
            });
    }
}

/// Stores instances of all panels to maintain state across frames
struct PanelInstances {
    variables: variables::VariablesPanel,
    call_stack: call_stack::CallStackPanel,
    registers: registers::RegistersPanel,
    memory: memory::MemoryPanel,
    disassembly: disassembly::DisassemblyPanel,
    breakpoints: breakpoints::BreakpointsPanel,
    source: source::SourcePanel,
}

impl PanelInstances {
    fn new() -> Self {
        Self {
            variables: variables::VariablesPanel::new(),
            call_stack: call_stack::CallStackPanel::new(),
            registers: registers::RegistersPanel::new(),
            memory: memory::MemoryPanel::new(),
            disassembly: disassembly::DisassemblyPanel::new(),
            breakpoints: breakpoints::BreakpointsPanel::new(),
            source: source::SourcePanel::new(),
        }
    }
}

/// Panel types that can be docked
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PanelType {
    Variables,
    CallStack,
    Registers,
    Memory,
    Disassembly,
    Breakpoints,
    Source,
}

impl PanelType {
    fn title(&self) -> &'static str {
        match self {
            PanelType::Variables => "Variables",
            PanelType::CallStack => "Call Stack",
            PanelType::Registers => "Registers",
            PanelType::Memory => "Memory",
            PanelType::Disassembly => "Disassembly",
            PanelType::Breakpoints => "Breakpoints",
            PanelType::Source => "Source",
        }
    }

    /// Get all panel types
    fn all() -> Vec<PanelType> {
        vec![
            PanelType::Source,
            PanelType::Variables,
            PanelType::CallStack,
            PanelType::Registers,
            PanelType::Disassembly,
            PanelType::Memory,
            PanelType::Breakpoints,
        ]
    }
}

impl AnteaterApp {
    /// Create a new Anteater application
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Create available themes
        let themes = vec![
            ITerm2Theme::default_dark(),
            ITerm2Theme::dracula(),
            ITerm2Theme::nord(),
            ITerm2Theme::one_dark(),
            ITerm2Theme::gruvbox_dark(),
        ];

        // Apply default theme
        let current_theme = themes[0].clone();
        let mut visuals = egui::Visuals::dark();
        current_theme.apply_to_egui(&mut visuals);
        cc.egui_ctx.set_visuals(visuals);

        Self {
            session: MockDebugSession::new(),
            dock_state: Self::create_default_layout(),
            panels: PanelInstances::new(),
            current_theme,
            themes,
            show_theme_selector: false,
            perf_overlay: PerformanceOverlay::new(),
        }
    }

    /// Create the default panel layout
    fn create_default_layout() -> DockState<PanelType> {
        // Create a simple default layout with all panels as tabs
        // Users can rearrange as they wish
        DockState::new(vec![
            PanelType::Source,
            PanelType::Variables,
            PanelType::CallStack,
            PanelType::Registers,
            PanelType::Disassembly,
            PanelType::Memory,
            PanelType::Breakpoints,
        ])
    }

    /// Handle global keyboard shortcuts
    fn handle_keyboard_shortcuts(&mut self, ctx: &egui::Context) {
        ctx.input(|i| {
            // F5 - Continue
            if i.key_pressed(egui::Key::F5) {
                println!("Continue (F5)");
                // TODO: Send continue command to debug engine
            }

            // F10 - Step Over
            if i.key_pressed(egui::Key::F10) {
                println!("Step Over (F10)");
                // TODO: Send step over command
            }

            // F11 - Step Into
            if i.key_pressed(egui::Key::F11) {
                println!("Step Into (F11)");
                // TODO: Send step into command
            }

            // Shift+F11 - Step Out
            if i.key_pressed(egui::Key::F11) && i.modifiers.shift {
                println!("Step Out (Shift+F11)");
                // TODO: Send step out command
            }

            // F9 - Toggle breakpoint at current line
            if i.key_pressed(egui::Key::F9) {
                println!("Toggle Breakpoint (F9)");
                // TODO: Toggle breakpoint
            }

            // Ctrl/Cmd+P - Command palette (TODO)
            if i.modifiers.command && i.key_pressed(egui::Key::P) {
                println!("Command Palette (Ctrl/Cmd+P)");
                // TODO: Open command palette
            }

            // F12 - Toggle performance overlay
            if i.key_pressed(egui::Key::F12) {
                self.perf_overlay.toggle();
            }
        });
    }

    /// Check if a panel is currently open in the dock
    fn is_panel_open(&self, panel_type: PanelType) -> bool {
        self.dock_state
            .main_surface()
            .iter()
            .any(|node| {
                if let Some(tabs) = node.tabs() {
                    tabs.iter().any(|tab| *tab == panel_type)
                } else {
                    false
                }
            })
    }

    /// Open a panel (add it as a new tab in the main surface)
    fn open_panel(&mut self, panel_type: PanelType) {
        if !self.is_panel_open(panel_type) {
            self.dock_state.main_surface_mut().push_to_focused_leaf(panel_type);
        }
    }

    /// Apply a theme to the application
    fn apply_theme(&mut self, ctx: &egui::Context, theme: ITerm2Theme) {
        let mut visuals = egui::Visuals::dark();
        theme.apply_to_egui(&mut visuals);
        ctx.set_visuals(visuals);
        self.current_theme = theme;
    }
}

impl eframe::App for AnteaterApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Update performance metrics
        self.perf_overlay.update();

        // Handle global keyboard shortcuts
        self.handle_keyboard_shortcuts(ctx);

        // Top menu bar
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Open Executable...").clicked() {
                        // TODO: File picker
                    }
                    if ui.button("Quit").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });

                ui.menu_button("Debug", |ui| {
                    if ui.button("Continue (F5)").clicked() {
                        // TODO: Send continue command
                    }
                    if ui.button("Step Over (F10)").clicked() {
                        // TODO: Send step over command
                    }
                    if ui.button("Step Into (F11)").clicked() {
                        // TODO: Send step into command
                    }
                });

                ui.menu_button("View", |ui| {
                    ui.menu_button("Panels", |ui| {
                        for panel_type in PanelType::all() {
                            let is_open = self.is_panel_open(panel_type);
                            if ui.checkbox(&mut is_open.clone(), panel_type.title()).changed() {
                                if is_open {
                                    // Panel is already open, do nothing (checkbox can't close panels)
                                } else {
                                    // Panel is closed, open it
                                    self.open_panel(panel_type);
                                }
                            }
                        }
                    });

                    ui.separator();

                    if ui.button("Change Theme...").clicked() {
                        self.show_theme_selector = true;
                    }

                    ui.separator();

                    if ui.button("Reset Layout").clicked() {
                        self.dock_state = Self::create_default_layout();
                    }
                });
            });
        });

        // Status bar
        egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Status:");
                match self.session.status() {
                    anteater_ui_types::ExecutionStatus::Stopped { reason } => {
                        ui.label(format!("Stopped ({:?})", reason));
                    }
                    anteater_ui_types::ExecutionStatus::Running => {
                        ui.label("Running");
                    }
                    status => {
                        ui.label(format!("{:?}", status));
                    }
                }

                ui.separator();

                if let Some(loc) = self.session.current_source_location() {
                    ui.label(format!("{}:{}", loc.path.0.display(), loc.line));
                }
            });
        });

        // Main docking area
        egui::CentralPanel::default().show(ctx, |ui| {
            let mut tab_viewer = TabViewer {
                session: &self.session,
                panels: &mut self.panels,
            };

            DockArea::new(&mut self.dock_state)
                .style(Style::from_egui(ctx.style().as_ref()))
                .show_inside(ui, &mut tab_viewer);

            // Show docking help on first load
            if ui.ctx().memory(|mem| mem.data.get_temp("show_docking_help".into()).unwrap_or(true)) {
                egui::Window::new("💡 Docking Help")
                    .collapsible(false)
                    .resizable(false)
                    .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                    .show(ui.ctx(), |ui| {
                        ui.label("Welcome to Anteater!");
                        ui.add_space(8.0);
                        ui.label("🔹 Drag tabs to rearrange panels");
                        ui.label("🔹 Drag tabs to window edges to create splits");
                        ui.label("🔹 Close tabs with the × button");
                        ui.label("🔹 Reopen panels via View → Panels");
                        ui.label("🔹 Change themes via View → Change Theme");
                        ui.add_space(8.0);

                        if ui.button("Got it!").clicked() {
                            ui.ctx().memory_mut(|mem| {
                                mem.data.insert_temp("show_docking_help".into(), false);
                            });
                        }
                    });
            }
        });

        // Theme selector dialog
        if self.show_theme_selector {
            let mut selected_theme: Option<ITerm2Theme> = None;
            let mut close_dialog = false;

            egui::Window::new("Select Theme")
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.heading("Choose a theme:");
                    ui.add_space(8.0);

                    for theme in &self.themes {
                        let is_current = theme.name == self.current_theme.name;
                        if ui.selectable_label(is_current, &theme.name).clicked() {
                            selected_theme = Some(theme.clone());
                            close_dialog = true;
                        }
                    }

                    ui.add_space(8.0);
                    ui.separator();

                    if ui.button("Close").clicked() {
                        close_dialog = true;
                    }

                    ui.add_space(4.0);
                    ui.label("💡 Tip: You can also load custom iTerm2 themes!");
                });

            if let Some(theme) = selected_theme {
                self.apply_theme(ctx, theme);
            }

            if close_dialog {
                self.show_theme_selector = false;
            }
        }

        // Render performance overlay (if enabled)
        self.perf_overlay.render(ctx);
    }
}

/// Tab viewer for rendering panel contents
struct TabViewer<'a> {
    session: &'a MockDebugSession,
    panels: &'a mut PanelInstances,
}

impl<'a> egui_dock::TabViewer for TabViewer<'a> {
    type Tab = PanelType;

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        tab.title().into()
    }

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        // Render the appropriate panel based on type using persistent instances
        match tab {
            PanelType::Variables => {
                self.panels.variables.ui(ui, self.session);
            }
            PanelType::CallStack => {
                self.panels.call_stack.ui(ui, self.session);
            }
            PanelType::Registers => {
                self.panels.registers.ui(ui, self.session);
            }
            PanelType::Memory => {
                self.panels.memory.ui(ui, self.session);
            }
            PanelType::Disassembly => {
                self.panels.disassembly.ui(ui, self.session);
            }
            PanelType::Breakpoints => {
                self.panels.breakpoints.ui(ui, self.session);
            }
            PanelType::Source => {
                self.panels.source.ui(ui, self.session);
            }
        }
    }
}

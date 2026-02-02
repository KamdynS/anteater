//! UI panels for the debugger
//!
//! Each panel is a self-contained component that can be placed in the docking layout.

pub mod registers;
pub mod call_stack;
pub mod variables;
pub mod memory;
pub mod disassembly;
pub mod breakpoints;
pub mod source;

/// Trait implemented by all debugger panels
pub trait Panel {
    /// Unique identifier for this panel type
    fn id(&self) -> &'static str;

    /// Display name shown in tabs
    fn title(&self) -> &str;

    /// Render the panel UI
    fn ui(&mut self, ui: &mut egui::Ui, session: &crate::mock::MockDebugSession);
}

//! # Anteater UI Types (ViewModel Layer)
//!
//! This module defines the data structures that the UI layer consumes. It serves as
//! the **contract** between the semantic layer (which you're building) and the UI
//! layer (which AI agents can help generate).
//!
//! ## Design Philosophy
//!
//! The UI should be "dumb" — it receives fully-interpreted, display-ready data and
//! renders it. All the hard work (DWARF parsing, MIR correlation, ownership tracking,
//! type interpretation) happens in the semantic layer. By the time data reaches these
//! types, it's already Rust-meaningful.
//!
//! ## Stability Notes
//!
//! These types are **provisional**. The core developer is building the semantic layer
//! in parallel, and these types will evolve. Each type has a `// STABILITY:` comment
//! indicating how likely it is to change:
//!
//! - `STABLE`: Core concept unlikely to change, though fields may be added
//! - `PROVISIONAL`: Shape is reasonable but may shift significantly  
//! - `SPECULATIVE`: Best guess based on design docs; expect changes
//!
//! AI agents generating UI code should:
//! 1. Build against these types as given
//! 2. Keep rendering logic decoupled from type internals where possible
//! 3. Expect to update code when types change
//!
//! ## Architecture Context
//!
//! ```
//! ┌─────────────────────────────────────────────────────┐
//! │                    UI Layer (egui)                  │
//! │         Consumes these ViewModel types              │  ← YOU ARE HERE
//! └─────────────────────┬───────────────────────────────┘
//!                       │
//! ┌─────────────────────▼───────────────────────────────┐
//! │                 Semantic Layer                      │
//! │    Produces these ViewModel types from raw state    │  ← Core developer builds
//! │    MIR correlation, ownership tracking, traits      │
//! └─────────────────────┬───────────────────────────────┘
//!                       │
//! ┌─────────────────────▼───────────────────────────────┐
//! │                  Debug Core                         │
//! │      ptrace control, DWARF parsing, memory read     │  ← Core developer builds
//! └─────────────────────────────────────────────────────┘
//! ```

use std::ops::Range;
use std::path::PathBuf;

// =============================================================================
// CORE DEBUG SESSION
// =============================================================================

/// The top-level handle to a debugging session.
///
/// The UI holds a reference to this and queries it each frame. The semantic layer
/// updates it as the debuggee state changes.
///
/// STABILITY: STABLE (interface will exist; methods may be added/changed)
///
/// OPEN QUESTION: Is this a trait or a concrete struct? Trait would allow mocking
/// for UI testing. For now, assume concrete struct that the semantic layer provides.
pub struct DebugSession {
    // Internal state managed by semantic layer — UI doesn't access directly
    // ...
}

impl DebugSession {
    /// Current execution state of the debuggee.
    pub fn status(&self) -> ExecutionStatus { todo!() }
    
    /// Stack frames for the current thread. Index 0 is the innermost frame.
    pub fn stack_frames(&self) -> &[StackFrame] { todo!() }
    
    /// Variables visible in a given stack frame.
    /// Includes locals, parameters, and captured variables (for closures).
    pub fn variables(&self, frame: FrameId) -> &[Variable] { todo!() }
    
    /// Read memory from the debuggee's address space.
    /// Returns None if the range is not readable (unmapped, protected, etc.)
    pub fn memory(&self, range: Range<u64>) -> Option<&[u8]> { todo!() }
    
    /// Current register values.
    pub fn registers(&self) -> &RegisterSet { todo!() }
    
    /// Disassembly for an address range.
    pub fn disassembly(&self, range: Range<u64>) -> Vec<DisassemblyLine> { todo!() }
    
    /// All breakpoints in the session.
    pub fn breakpoints(&self) -> &[Breakpoint] { todo!() }
    
    /// Source file contents (for display in source panel).
    /// The semantic layer handles finding/loading source files.
    pub fn source_file(&self, path: &SourcePath) -> Option<&SourceFile> { todo!() }
    
    /// Current source location (file + line), if determinable.
    pub fn current_source_location(&self) -> Option<SourceLocation> { todo!() }
}

/// Execution status of the debuggee.
///
/// STABILITY: STABLE
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExecutionStatus {
    /// Not yet started or loading
    NotStarted,
    /// Running (not stopped at breakpoint)
    Running,
    /// Stopped at a breakpoint or after stepping
    Stopped { reason: StopReason },
    /// Process has exited
    Exited { code: i32 },
    /// Process was killed by signal
    Signaled { signal: i32 },
}

/// Why the debuggee stopped.
///
/// STABILITY: PROVISIONAL (may add more reasons)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StopReason {
    Breakpoint { id: BreakpointId },
    Step,
    Signal { signal: i32 },
    /// Stopped at an ownership decision point (sparse tracking)
    /// The UI probably doesn't need to distinguish this from Step, but
    /// the semantic layer might surface it for transparency.
    OwnershipCheckpoint,
}

// =============================================================================
// STACK FRAMES
// =============================================================================

/// Unique identifier for a stack frame within a debug session.
///
/// STABILITY: STABLE
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FrameId(pub u32);

/// A single stack frame.
///
/// STABILITY: STABLE (fields may be added)
#[derive(Debug, Clone)]
pub struct StackFrame {
    pub id: FrameId,
    
    /// Function name, demangled and formatted for display.
    /// Includes generic parameters if applicable, e.g., `process::<Vec<u8>>`.
    pub function_name: String,
    
    /// Source location, if debug info is available.
    pub source_location: Option<SourceLocation>,
    
    /// Instruction pointer for this frame.
    pub instruction_pointer: u64,
    
    /// Whether this frame is inlined (not a real stack frame).
    /// Inlined frames are synthesized from debug info for better UX.
    pub is_inlined: bool,
    
    /// Module/crate this function belongs to, if known.
    /// Useful for distinguishing std library frames from user code.
    pub module: Option<String>,
}

// =============================================================================
// VARIABLES AND OWNERSHIP
// =============================================================================

/// A variable visible in the debugger.
///
/// This is the **core type** for the variable inspector panel. It includes
/// both the standard debugger information (name, type, value) and Rust-specific
/// semantic information (ownership state).
///
/// STABILITY: PROVISIONAL
///
/// The `ownership` field is the key Anteater innovation. It comes from the
/// MIR+DWARF correlation layer. See `OwnershipState` for details.
#[derive(Debug, Clone)]
pub struct Variable {
    /// Variable name as it appears in source.
    pub name: String,
    
    /// Rust type, formatted for display.
    /// Examples: `Vec<u8>`, `&mut HashMap<String, i32>`, `Option<Box<dyn Error>>`
    pub rust_type: TypeDisplay,
    
    /// Current value, formatted for display.
    /// For compound types, this is a summary (e.g., `Vec(len=3)`).
    /// Detailed contents are in `children`.
    pub value: ValueDisplay,
    
    /// Ownership/borrow state of this variable.
    /// This is what makes Anteater special.
    pub ownership: OwnershipState,
    
    /// For compound types (structs, enums, arrays, etc.), the nested values.
    /// Empty for primitives.
    pub children: Vec<Variable>,
    
    /// Memory address where this variable lives, if applicable.
    /// None for optimized-out variables or registers.
    pub address: Option<u64>,
    
    /// Whether this variable has been optimized out by the compiler.
    /// The UI should display this distinctly (grayed out, with explanation).
    pub is_optimized_out: bool,
    
    /// For enum variants: which variant is active.
    /// None if not an enum or if variant can't be determined.
    pub enum_variant: Option<String>,
}

/// Rust type formatted for display.
///
/// STABILITY: SPECULATIVE
///
/// The semantic layer will do the work of producing nice type strings from
/// DWARF info. This wrapper exists in case we need structured type info
/// later (e.g., for syntax highlighting within type names, or for type-aware
/// interactions like "show all variables of this type").
///
/// For now, AI agents can treat this as effectively a String.
#[derive(Debug, Clone)]
pub struct TypeDisplay {
    /// The full type as a string, e.g., `HashMap<String, Vec<u8>>`
    pub full: String,
    
    /// Optional short form for constrained UI space, e.g., `HashMap<...>`
    pub short: Option<String>,
}

impl TypeDisplay {
    pub fn new(full: impl Into<String>) -> Self {
        Self { full: full.into(), short: None }
    }
    
    pub fn display(&self) -> &str {
        &self.full
    }
}

/// Value formatted for display.
///
/// STABILITY: SPECULATIVE
///
/// Similar rationale to TypeDisplay — wrapping in case we need structure later.
#[derive(Debug, Clone)]
pub struct ValueDisplay {
    /// The value as a string.
    /// For primitives: the literal value, e.g., `42`, `"hello"`, `true`
    /// For compounds: a summary, e.g., `Vec(len=3, cap=8)`, `Some(...)`
    pub display: String,
    
    /// Raw bytes if available (for hex view integration).
    pub raw_bytes: Option<Vec<u8>>,
}

impl ValueDisplay {
    pub fn new(display: impl Into<String>) -> Self {
        Self { display: display.into(), raw_bytes: None }
    }
    
    pub fn display(&self) -> &str {
        &self.display
    }
}

/// Ownership state of a variable.
///
/// STABILITY: PROVISIONAL — this is central to Anteater's value proposition.
/// The states are derived from MIR analysis + runtime tracking.
///
/// From the design doc:
/// > Ownership status is inherently historical. We need execution history.
///
/// The semantic layer computes this by replaying MIR semantics against the
/// recorded branch history. By the time it reaches the UI, it's a simple enum.
///
/// ## Display Guidance for UI
///
/// The UI should have a consistent visual language for these states:
/// - `Owned`: Normal display (no special indicator, or subtle "owned" badge)
/// - `MovedFrom`: Grayed out / strikethrough, "moved" indicator
/// - `Borrowed`: Show what's borrowing it, maybe a visual link
/// - `MutablyBorrowed`: Same as Borrowed but more prominent (mutable is significant)
/// - `Dropped`: Grayed out, "dropped" indicator
/// - `Uninitialized`: Grayed out, "uninitialized" indicator
/// - `PartiallyMoved`: Complex case for structs — show which fields moved
///
/// The exact visual language should be defined in a theme/style guide.
#[derive(Debug, Clone, PartialEq)]
pub enum OwnershipState {
    /// Variable owns its value. Normal state.
    Owned,
    
    /// Value has been moved out of this variable.
    /// The memory may still contain the old bytes, but semantically it's gone.
    MovedFrom {
        /// Where the value moved to, if known.
        moved_to: Option<String>,
    },
    
    /// Value is currently borrowed (shared reference).
    Borrowed {
        /// What variable(s) hold the borrow.
        by: Vec<String>,
    },
    
    /// Value is currently mutably borrowed.
    MutablyBorrowed {
        /// What variable holds the mutable borrow.
        by: String,
    },
    
    /// Value has been dropped (destructor ran).
    Dropped,
    
    /// Variable declared but not yet initialized.
    /// `let x: i32;` before assignment.
    Uninitialized,
    
    /// For structs: some fields have been moved out.
    /// Rust allows this with `#[feature(move_fields)]` or in certain patterns.
    PartiallyMoved {
        /// Which fields have been moved.
        moved_fields: Vec<String>,
    },
    
    /// Ownership state can't be determined.
    /// Possible reasons: optimized out, complex control flow, MIR correlation failed.
    Unknown {
        reason: Option<String>,
    },
}

impl OwnershipState {
    /// Whether this variable's value is "accessible" — can be meaningfully read.
    pub fn is_accessible(&self) -> bool {
        matches!(self, 
            OwnershipState::Owned | 
            OwnershipState::Borrowed { .. } |
            OwnershipState::MutablyBorrowed { .. }
        )
    }
    
    /// Whether to display this variable as "grayed out" or diminished.
    pub fn should_dim(&self) -> bool {
        matches!(self,
            OwnershipState::MovedFrom { .. } |
            OwnershipState::Dropped |
            OwnershipState::Uninitialized
        )
    }
}

// =============================================================================
// SOURCE CODE
// =============================================================================

/// Path to a source file.
///
/// STABILITY: STABLE
///
/// This might be absolute or relative depending on how the binary was compiled.
/// The semantic layer handles resolving paths to actual files.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SourcePath(pub PathBuf);

/// Source location (file + line + optional column).
///
/// STABILITY: STABLE
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceLocation {
    pub path: SourcePath,
    pub line: u32,       // 1-indexed
    pub column: Option<u32>, // 1-indexed, if available
}

/// Contents of a source file, with metadata for display.
///
/// STABILITY: PROVISIONAL
#[derive(Debug, Clone)]
pub struct SourceFile {
    pub path: SourcePath,
    
    /// Lines of the file. Index 0 = line 1.
    pub lines: Vec<String>,
    
    /// Breakpoints set in this file, by line number.
    pub breakpoints: Vec<u32>,
    
    /// Lines that have executable code (for breakpoint validity indication).
    /// Not all lines can have breakpoints (comments, blank lines, etc.)
    pub executable_lines: Vec<u32>,
}

// =============================================================================
// BREAKPOINTS
// =============================================================================

/// Unique identifier for a breakpoint.
///
/// STABILITY: STABLE
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BreakpointId(pub u32);

/// A breakpoint set in the debuggee.
///
/// STABILITY: PROVISIONAL
#[derive(Debug, Clone)]
pub struct Breakpoint {
    pub id: BreakpointId,
    pub enabled: bool,
    pub location: BreakpointLocation,
    
    /// Condition expression (if conditional breakpoint).
    /// This is a Rust expression that must evaluate to bool.
    pub condition: Option<String>,
    
    /// Number of times this breakpoint has been hit.
    pub hit_count: u32,
    
    /// If set, only break after this many hits.
    pub ignore_count: u32,
}

/// Where a breakpoint is set.
///
/// STABILITY: PROVISIONAL
#[derive(Debug, Clone)]
pub enum BreakpointLocation {
    /// Breakpoint at a source location.
    Source { location: SourceLocation },
    
    /// Breakpoint at a function entry.
    Function { name: String },
    
    /// Breakpoint at a raw address.
    Address { address: u64 },
}

// =============================================================================
// REGISTERS
// =============================================================================

/// Register values for the current state.
///
/// STABILITY: PROVISIONAL
///
/// This is x86-64 specific for now (per the project scope).
#[derive(Debug, Clone)]
pub struct RegisterSet {
    /// General-purpose registers
    pub general: Vec<Register>,
    
    /// Floating-point / SSE registers  
    pub floating: Vec<Register>,
    
    /// Special registers (flags, segment, etc.)
    pub special: Vec<Register>,
}

/// A single register.
///
/// STABILITY: PROVISIONAL
#[derive(Debug, Clone)]
pub struct Register {
    pub name: String,
    pub value: u64,
    
    /// For flags registers, breakdown of individual flags.
    pub flags: Option<Vec<(String, bool)>>,
    
    /// Whether this register changed since the last stop.
    pub changed: bool,
}

// =============================================================================
// DISASSEMBLY
// =============================================================================

/// A line of disassembly.
///
/// STABILITY: PROVISIONAL
#[derive(Debug, Clone)]
pub struct DisassemblyLine {
    pub address: u64,
    
    /// Raw bytes of the instruction.
    pub bytes: Vec<u8>,
    
    /// Disassembled instruction (mnemonic + operands).
    pub instruction: String,
    
    /// Source location this instruction corresponds to, if known.
    pub source_location: Option<SourceLocation>,
    
    /// Whether this is the current instruction pointer.
    pub is_current: bool,
    
    /// Whether there's a breakpoint at this address.
    pub has_breakpoint: bool,
    
    /// For call instructions: the name of the function being called, if known.
    pub call_target: Option<String>,
}

// =============================================================================
// MEMORY VIEW
// =============================================================================

/// Configuration for memory view display.
///
/// STABILITY: SPECULATIVE
///
/// This isn't really a ViewModel type — it's UI state. But including it here
/// as a suggestion for how the memory panel might be configured.
#[derive(Debug, Clone)]
pub struct MemoryViewConfig {
    /// Number of bytes per row (typically 16).
    pub bytes_per_row: usize,
    
    /// How to interpret bytes beyond raw hex.
    pub interpretation: MemoryInterpretation,
}

/// How to interpret memory bytes.
///
/// STABILITY: SPECULATIVE
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MemoryInterpretation {
    /// Just hex + ASCII
    None,
    /// Interpret as array of i32, f64, etc.
    AsType(String),
}

// =============================================================================
// RUST-SPECIFIC: TRAITS
// =============================================================================

/// Trait information for a type.
///
/// STABILITY: SPECULATIVE
///
/// From the project description:
/// > For any value, see what traits its type implements.
///
/// This is one of Anteater's differentiating features. The semantic layer
/// extracts trait impls from MIR/DWARF. The UI displays them.
#[derive(Debug, Clone)]
pub struct TraitInfo {
    /// Trait name, e.g., `Debug`, `Clone`, `Iterator`
    pub name: String,
    
    /// Full path if not in prelude, e.g., `std::iter::Iterator`
    pub path: Option<String>,
    
    /// Methods provided by this trait impl.
    pub methods: Vec<TraitMethod>,
    
    /// Whether this is an auto trait (Send, Sync, etc.)
    pub is_auto: bool,
}

/// A method from a trait implementation.
///
/// STABILITY: SPECULATIVE
#[derive(Debug, Clone)]
pub struct TraitMethod {
    pub name: String,
    
    /// Signature for display, e.g., `fn next(&mut self) -> Option<Self::Item>`
    pub signature: String,
    
    /// Source location of the impl, if available (for "jump to impl" feature).
    pub impl_location: Option<SourceLocation>,
}

// =============================================================================
// RUST-SPECIFIC: BORROW SCOPE VISUALIZATION
// =============================================================================

/// Information about active borrows in a scope.
///
/// STABILITY: SPECULATIVE
///
/// This is for the "borrow scope display" feature — showing what's borrowing what
/// and for how long. The exact shape of this is unclear and will evolve.
///
/// From the design doc:
/// > Understand the lifetime of borrows in the current scope. See what's borrowing what.
#[derive(Debug, Clone)]
pub struct BorrowInfo {
    /// The variable being borrowed.
    pub borrowed_variable: String,
    
    /// The variable holding the borrow.
    pub borrower: String,
    
    /// Whether it's a mutable borrow.
    pub is_mutable: bool,
    
    /// Source range where this borrow is active.
    /// Used for visual highlighting in the source view.
    pub active_range: Option<SourceRange>,
}

/// A range in source code (for highlighting).
///
/// STABILITY: SPECULATIVE
#[derive(Debug, Clone)]
pub struct SourceRange {
    pub start: SourceLocation,
    pub end: SourceLocation,
}

// =============================================================================
// RUST-SPECIFIC: PATTERN MATCH STATE
// =============================================================================

/// State of a pattern match when stopped inside a `match` expression.
///
/// STABILITY: SPECULATIVE
///
/// From the project description:
/// > When stopped inside a match, see which arm matched and why.
#[derive(Debug, Clone)]
pub struct PatternMatchState {
    /// The expression being matched, as source text.
    pub scrutinee: String,
    
    /// Value of the scrutinee.
    pub scrutinee_value: ValueDisplay,
    
    /// All arms of the match.
    pub arms: Vec<MatchArm>,
    
    /// Which arm matched (index into `arms`).
    pub matched_arm: usize,
}

/// A single arm of a match expression.
///
/// STABILITY: SPECULATIVE  
#[derive(Debug, Clone)]
pub struct MatchArm {
    /// Pattern as source text, e.g., `Some(x) if x > 0`
    pub pattern: String,
    
    /// Whether this arm matched.
    pub matched: bool,
    
    /// If didn't match, why not (for UX explanation).
    pub non_match_reason: Option<String>,
    
    /// Bindings introduced by this pattern if it matched.
    pub bindings: Vec<(String, ValueDisplay)>,
}

// =============================================================================
// UI COMMANDS (for command pattern / action dispatch)
// =============================================================================

/// Commands the UI can send to the debug session.
///
/// STABILITY: PROVISIONAL
///
/// Using a command enum allows for undo/redo, logging, and clean separation
/// between UI interaction and debug operations.
#[derive(Debug, Clone)]
pub enum DebugCommand {
    // Execution control
    Continue,
    StepOver,
    StepInto,
    StepOut,
    StepInstruction,
    Pause,
    Stop,
    Restart,
    
    // Breakpoints
    AddBreakpoint { location: BreakpointLocation },
    RemoveBreakpoint { id: BreakpointId },
    EnableBreakpoint { id: BreakpointId, enabled: bool },
    SetBreakpointCondition { id: BreakpointId, condition: Option<String> },
    
    // Navigation
    SelectFrame { id: FrameId },
    GoToSource { location: SourceLocation },
    GoToAddress { address: u64 },
    
    // Variable inspection
    ExpandVariable { path: Vec<String> }, // Path through nested structure
    CollapseVariable { path: Vec<String> },
}

// =============================================================================
// THEME / VISUAL LANGUAGE (for consistency across AI-generated panels)
// =============================================================================

/// Color/style definitions for consistent UI appearance.
///
/// STABILITY: SPECULATIVE
///
/// AI agents generating UI code should reference these rather than hardcoding
/// colors. This allows the visual language to be tuned in one place.
///
/// Note: This is a sketch. The actual implementation might use egui's style
/// system more directly.
pub mod theme {
    use super::OwnershipState;
    
    /// Colors for ownership state indicators.
    /// Returns (background, foreground, badge_text)
    pub fn ownership_colors(state: &OwnershipState) -> (&'static str, &'static str, &'static str) {
        match state {
            OwnershipState::Owned => ("#e8f5e9", "#2e7d32", "owned"),
            OwnershipState::MovedFrom { .. } => ("#fafafa", "#9e9e9e", "moved"),
            OwnershipState::Borrowed { .. } => ("#e3f2fd", "#1565c0", "borrowed"),
            OwnershipState::MutablyBorrowed { .. } => ("#fff3e0", "#e65100", "mut borrowed"),
            OwnershipState::Dropped => ("#fafafa", "#757575", "dropped"),
            OwnershipState::Uninitialized => ("#fafafa", "#bdbdbd", "uninitialized"),
            OwnershipState::PartiallyMoved { .. } => ("#fff8e1", "#ff8f00", "partial"),
            OwnershipState::Unknown { .. } => ("#fafafa", "#9e9e9e", "?"),
        }
    }
    
    /// Whether to use strikethrough for this ownership state.
    pub fn use_strikethrough(state: &OwnershipState) -> bool {
        matches!(state, OwnershipState::MovedFrom { .. } | OwnershipState::Dropped)
    }
}

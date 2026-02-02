# How Anteater Works: Complete Deep Dive

This document explains every detail of how Anteater works, from workspace structure to event loops to rendering. This is the comprehensive technical reference.

---

## Table of Contents

1. [Workspace Architecture](#workspace-architecture)
2. [Data Flow: From ptrace to Pixels](#data-flow)
3. [The ViewModel Pattern](#the-viewmodel-pattern)
4. [egui Immediate Mode Rendering](#egui-immediate-mode)
5. [Panel System Deep Dive](#panel-system)
6. [Event Loop and State Management](#event-loop)
7. [Mock Data System](#mock-data)
8. [Keyboard Shortcuts and Commands](#keyboard-shortcuts)
9. [Theme System](#theme-system)
10. [Memory Layout Examples](#memory-layout)

---

## Workspace Architecture

Anteater is a **Rust workspace** with 5 separate crates. This modular design allows parallel development and clean separation of concerns.

```
anteater/                              # Workspace root
│
├── Cargo.toml                         # Workspace manifest
│   └── [workspace]
│       ├── members = ["crates/*"]
│       └── dependencies = {...}       # Shared dependency versions
│
├── docs/                              # All documentation lives here
│   ├── ARCHITECTURE.md
│   ├── HOW_IT_WORKS.md               # This file
│   └── ...
│
└── crates/                            # All code lives in workspace members
    │
    ├── anteater-ui-types/             # ViewModel contract (85 lines)
    │   ├── Cargo.toml
    │   └── src/
    │       └── lib.rs                 # The entire UI contract in one file
    │           ├── pub struct Variable
    │           ├── pub enum OwnershipState
    │           ├── pub struct StackFrame
    │           ├── pub trait DebugSession  # <-- THE KEY INTERFACE
    │           └── ...
    │
    ├── anteater-ui/                   # UI implementation (~2000 lines)
    │   ├── Cargo.toml
    │   │   └── dependencies:
    │   │       ├── anteater-ui-types  # Only depends on the contract
    │   │       ├── egui
    │   │       ├── eframe
    │   │       ├── egui_dock
    │   │       └── syntect
    │   └── src/
    │       ├── lib.rs                 # Re-exports + theme module
    │       ├── app.rs                 # Main app + docking system
    │       ├── mock.rs                # MockDebugSession (for development)
    │       ├── panels/                # All 7 debugger panels
    │       │   ├── mod.rs
    │       │   ├── variables.rs
    │       │   ├── call_stack.rs
    │       │   ├── registers.rs
    │       │   ├── memory.rs
    │       │   ├── disassembly.rs
    │       │   ├── breakpoints.rs
    │       │   └── source.rs
    │       └── widgets/               # Reusable components
    │           ├── mod.rs
    │           ├── ownership_badge.rs
    │           └── type_display.rs
    │
    ├── anteater-engine/               # Semantic layer (YOU BUILD THIS)
    │   ├── Cargo.toml
    │   │   └── dependencies:
    │   │       ├── anteater-ui-types  # Implements DebugSession trait
    │   │       └── anteater-core      # Uses ptrace/DWARF from core
    │   └── src/
    │       └── lib.rs                 # Future: MIR correlation, ownership tracking
    │
    ├── anteater-core/                 # Debug core (YOU BUILD THIS)
    │   ├── Cargo.toml
    │   └── src/
    │       └── lib.rs                 # Future: ptrace wrapper, DWARF parser
    │
    └── anteater/                      # Binary crate (main entry point)
        ├── Cargo.toml
        │   └── dependencies:
        │       ├── anteater-ui        # Provides the UI
        │       ├── anteater-engine    # Provides the debug logic
        │       └── eframe             # For main() setup
        └── src/
            └── main.rs                # Just 20 lines: setup + run
```

### Why This Structure?

**Dependency Flow:**
```
anteater (binary)
  │
  ├──> anteater-ui ──────────┐
  │                          ├──> anteater-ui-types (contract)
  └──> anteater-engine ──────┤
            │                │
            └──> anteater-core
```

**Key insight:** `anteater-ui` NEVER depends on `anteater-core` or `anteater-engine`. It only knows about the **ViewModel types** in `anteater-ui-types`. This means:

- UI can be built/tested with mock data
- Core can change internals without breaking UI
- Clear contract = parallel development

---

## Data Flow: From ptrace to Pixels

Here's the **complete journey** of data from the debugged process to your screen:

```
┌─────────────────────────────────────────────────────────────────────┐
│                         USER'S PROGRAM                               │
│   (being debugged, stopped at breakpoint)                           │
│                                                                      │
│   let mut data = vec![1, 2, 3];                                     │
│   let borrowed = &data;  // <-- stopped here                        │
│                                                                      │
│   Process memory: 0x7fff_1234_5678 = [01 00 00 00 02 00 00 00 ...]  │
└─────────────────────────────────┬───────────────────────────────────┘
                                  │
                                  │ ptrace(PTRACE_PEEKDATA, ...)
                                  │ reads memory
                                  ▼
┌─────────────────────────────────────────────────────────────────────┐
│                    ANTEATER-CORE (Debug Core)                       │
│                                                                      │
│  ┌──────────────────┐      ┌──────────────────┐                    │
│  │  Ptrace Wrapper  │      │  DWARF Parser    │                    │
│  │                  │      │                  │                    │
│  │ - attach()       │      │ - parse .debug_* │                    │
│  │ - read_memory()  │      │ - find vars      │                    │
│  │ - get_regs()     │      │ - get locations  │                    │
│  │ - continue()     │      │ - unwind stack   │                    │
│  └──────────────────┘      └──────────────────┘                    │
│                                                                      │
│  Raw data: "Variable 'data' at RBP-24, type 'Vec<i32>'"            │
└─────────────────────────────────┬───────────────────────────────────┘
                                  │
                                  │ Feeds into semantic analysis
                                  ▼
┌─────────────────────────────────────────────────────────────────────┐
│                  ANTEATER-ENGINE (Semantic Layer)                   │
│                                                                      │
│  ┌──────────────────┐      ┌──────────────────┐                    │
│  │   MIR Parser     │      │  MIR+DWARF       │                    │
│  │                  │      │  Correlator      │                    │
│  │ - parse MIR from │◄─────┤                  │                    │
│  │   .mir files     │      │ - match MIR vars │                    │
│  │ - track moves    │      │   to DWARF locs  │                    │
│  │ - track borrows  │      │ - infer ownership│                    │
│  └──────────────────┘      └──────────────────┘                    │
│                                     │                                │
│                                     │ produces                      │
│                                     ▼                                │
│                          ┌──────────────────────┐                   │
│                          │ OWNERSHIP TRACKING   │                   │
│                          │                      │                   │
│                          │ Tracks per-variable: │                   │
│                          │ - Owned / Moved      │                   │
│                          │ - Borrowed / &mut    │                   │
│                          │ - Dropped / Uninit   │                   │
│                          └──────────────────────┘                   │
│                                     │                                │
│                                     │ Implements DebugSession trait │
│                                     ▼                                │
│  impl DebugSession for RealDebugSession {                           │
│      fn variables(&self, frame) -> &[Variable] {                    │
│          // Returns Variables with OwnershipState filled in         │
│      }                                                               │
│  }                                                                   │
└─────────────────────────────────────┬───────────────────────────────┘
                                      │
                                      │ Exposes ViewModel types
                                      ▼
┌─────────────────────────────────────────────────────────────────────┐
│              ANTEATER-UI-TYPES (The Contract / ViewModel)           │
│                                                                      │
│  pub struct Variable {                                              │
│      pub name: String,              // "data"                       │
│      pub rust_type: TypeDisplay,    // "Vec<i32>"                   │
│      pub value: ValueDisplay,       // "Vec(len=3, cap=4)"          │
│      pub ownership: OwnershipState, // Borrowed { by: ["borrowed"] }│
│      pub address: Option<u64>,      // Some(0x7fff_1234_5678)       │
│      pub children: Vec<Variable>,   // [ptr, len, cap fields]       │
│      ...                                                             │
│  }                                                                   │
│                                                                      │
│  pub enum OwnershipState {                                          │
│      Owned,                         // Normal state                 │
│      Borrowed { by: Vec<String> },  // &T borrows                   │
│      MutablyBorrowed { by: String },// &mut T borrow                │
│      MovedFrom { moved_to: ... },   // Value moved out              │
│      Dropped,                       // Destructor ran               │
│      ...                                                             │
│  }                                                                   │
│                                                                      │
│  pub trait DebugSession {          // <-- THE KEY INTERFACE         │
│      fn variables(&self, frame: FrameId) -> &[Variable];            │
│      fn stack_frames(&self) -> &[StackFrame];                       │
│      fn registers(&self) -> &RegisterSet;                           │
│      fn memory(&self, range: Range<u64>) -> Option<&[u8]>;          │
│      fn current_source_location(&self) -> Option<SourceLocation>;   │
│      ...                                                             │
│  }                                                                   │
└─────────────────────────────────────┬───────────────────────────────┘
                                      │
                                      │ UI consumes this trait
                                      ▼
┌─────────────────────────────────────────────────────────────────────┐
│                    ANTEATER-UI (UI Implementation)                  │
│                                                                      │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │  app.rs: AnteaterApp                                        │   │
│  │                                                              │   │
│  │  pub struct AnteaterApp {                                   │   │
│  │      session: Box<dyn DebugSession>,  // <-- polymorphic!   │   │
│  │      dock_state: DockState<PanelType>,                      │   │
│  │      panels: PanelInstances,                                │   │
│  │      current_theme: ITerm2Theme,                            │   │
│  │  }                                                           │   │
│  │                                                              │   │
│  │  impl eframe::App for AnteaterApp {                         │   │
│  │      fn update(&mut self, ctx: &egui::Context, frame) {     │   │
│  │          // Draw menu bar                                   │   │
│  │          // Draw docking area with all panels               │   │
│  │          // Handle keyboard shortcuts                       │   │
│  │      }                                                       │   │
│  │  }                                                           │   │
│  └─────────────────────────────────────────────────────────────┘   │
│                          │                                          │
│                          │ delegates to                             │
│                          ▼                                          │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │  panels/variables.rs: VariablesPanel                        │   │
│  │                                                              │   │
│  │  impl VariablesPanel {                                      │   │
│  │      pub fn render(&mut self,                               │   │
│  │                    ui: &mut egui::Ui,                       │   │
│  │                    session: &dyn DebugSession) {            │   │
│  │                                                              │   │
│  │          let vars = session.variables(selected_frame);      │   │
│  │                                                              │   │
│  │          for var in vars {                                  │   │
│  │              // Render name                                 │   │
│  │              ui.label(&var.name);                           │   │
│  │                                                              │   │
│  │              // Render ownership badge                      │   │
│  │              OwnershipBadge::render(ui, &var.ownership);    │   │
│  │                                                              │   │
│  │              // Render type and value                       │   │
│  │              ui.label(format!("{}: {}", var.rust_type,      │   │
│  │                                          var.value));        │   │
│  │          }                                                   │   │
│  │      }                                                       │   │
│  │  }                                                           │   │
│  └─────────────────────────────────────────────────────────────┘   │
│                          │                                          │
│                          │ calls egui primitives                    │
│                          ▼                                          │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │  egui (immediate mode GUI library)                          │   │
│  │                                                              │   │
│  │  - Receives declarative UI code each frame                  │   │
│  │  - Generates draw commands (shapes, text, etc.)             │   │
│  │  - Handles input (mouse, keyboard)                          │   │
│  └─────────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────┬───────────────────────────────┘
                                      │
                                      │ Produces draw commands
                                      ▼
┌─────────────────────────────────────────────────────────────────────┐
│                      EFRAME (egui backend)                          │
│                                                                      │
│  - Manages window lifecycle                                         │
│  - Renders egui shapes via OpenGL/Vulkan/Metal                      │
│  - Handles OS events                                                │
└─────────────────────────────────────┬───────────────────────────────┘
                                      │
                                      │ GPU rendering
                                      ▼
┌─────────────────────────────────────────────────────────────────────┐
│                            YOUR SCREEN                               │
│                                                                      │
│  ┌────────────────────────────────────────────────────────────┐    │
│  │ Anteater                                    [_] [□] [×]     │    │
│  ├────────────────────────────────────────────────────────────┤    │
│  │ File  Debug  View                                          │    │
│  ├────────────────────────────────────────────────────────────┤    │
│  │                    │                                        │    │
│  │  Source Panel      │  Variables Panel                      │    │
│  │                    │  ┌──────────────────────────────┐     │    │
│  │  let mut data =    │  │ data         Vec<i32>        │     │    │
│  │      vec![1,2,3];  │  │ [&] borrowed by 'borrowed'   │     │    │
│  │  let borrowed =    │  │   ├─ ptr     *const i32      │     │    │
│  │      &data;  ◄─────┼──┤   ├─ len     3               │     │    │
│  │                    │  │   └─ cap     4               │     │    │
│  │                    │  │                              │     │    │
│  │                    │  │ borrowed     &Vec<i32>       │     │    │
│  │                    │  │              &data           │     │    │
│  │                    │  └──────────────────────────────┘     │    │
│  └────────────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────────────┘
```

**Key Points:**

1. **Layered abstraction:** Each layer only talks to the layer below it through a clean interface
2. **ViewModel is the boundary:** UI doesn't know about ptrace, DWARF, or MIR
3. **Polymorphism:** `session: Box<dyn DebugSession>` can be `MockDebugSession` (for development) or `RealDebugSession` (production)
4. **Immediate mode:** UI re-describes itself every frame (60fps)

---

## The ViewModel Pattern

The **ViewModel** is the contract between the semantic layer (you build) and the UI layer (already built).

### The Contract (anteater-ui-types/src/lib.rs)

```rust
/// The main interface the UI uses to get debug information
pub trait DebugSession {
    /// Get all variables visible in a stack frame
    fn variables(&self, frame: FrameId) -> &[Variable];

    /// Get the call stack
    fn stack_frames(&self) -> &[StackFrame];

    /// Get CPU registers
    fn registers(&self) -> &RegisterSet;

    /// Read memory from the debugged process
    fn memory(&self, range: Range<u64>) -> Option<&[u8]>;

    /// Get current source location
    fn current_source_location(&self) -> Option<SourceLocation>;

    /// Get source file contents
    fn source_file(&self, path: &SourcePath) -> Option<&SourceFile>;

    // ... more methods
}

/// A single variable in the debugged program
pub struct Variable {
    /// Variable name (e.g., "data", "self", "x")
    pub name: String,

    /// Rust type (e.g., "Vec<i32>", "&str", "Option<Result<T, E>>")
    pub rust_type: TypeDisplay,

    /// Current value (e.g., "Vec(len=3)", "\"hello\"", "Some(42)")
    pub value: ValueDisplay,

    /// THE MAGIC: Ownership state
    pub ownership: OwnershipState,

    /// Memory address (if known)
    pub address: Option<u64>,

    /// Child fields (for structs, enums, arrays)
    pub children: Vec<Variable>,

    /// True if optimized out by compiler
    pub is_optimized_out: bool,

    /// For enums: which variant is active
    pub enum_variant: Option<String>,
}

/// The core innovation: ownership states
pub enum OwnershipState {
    /// Normal state: variable owns its value
    Owned,

    /// Value was moved out
    MovedFrom {
        moved_to: Option<String>, // e.g., "new_owner"
    },

    /// Borrowed via &T (shared borrow)
    Borrowed {
        by: Vec<String>, // Can have multiple shared borrows
    },

    /// Borrowed via &mut T (exclusive borrow)
    MutablyBorrowed {
        by: String, // Only one &mut allowed
    },

    /// Destructor has run
    Dropped,

    /// Declared but never initialized
    Uninitialized,

    /// Some fields moved, others remain
    PartiallyMoved {
        moved_fields: Vec<String>,
    },

    /// Couldn't determine (optimized build, etc.)
    Unknown {
        reason: Option<String>,
    },
}
```

### How UI Uses ViewModel

**Variables Panel Example:**

```rust
pub fn render(&mut self, ui: &mut egui::Ui, session: &dyn DebugSession) {
    let variables = session.variables(self.selected_frame);

    for var in variables {
        ui.horizontal(|ui| {
            // 1. Render ownership badge (color-coded)
            match &var.ownership {
                OwnershipState::Owned => {
                    // No badge needed (common case)
                }
                OwnershipState::MovedFrom { moved_to } => {
                    ui.label(RichText::new(&var.name)
                        .strikethrough()
                        .color(GRAY));
                    ui.colored_label(GRAY, "moved");
                }
                OwnershipState::Borrowed { by } => {
                    ui.label(&var.name);
                    ui.colored_label(BLUE, "&");
                    if let Some(borrower) = by.first() {
                        ui.label(format!("borrowed by '{}'", borrower));
                    }
                }
                OwnershipState::MutablyBorrowed { by } => {
                    ui.label(&var.name);
                    ui.colored_label(ORANGE, "&mut");
                    ui.label(format!("mut borrowed by '{}'", by));
                }
                // ... other states
            }

            // 2. Render type and value
            ui.label(format!("{}", var.rust_type));
            ui.label(format!("{}", var.value));
        });

        // 3. If it has children (struct fields, etc.), render them indented
        if !var.children.is_empty() {
            ui.indent("children", |ui| {
                for child in &var.children {
                    // Recursive rendering
                    self.render_variable(ui, child);
                }
            });
        }
    }
}
```

**Memory Panel Example:**

```rust
pub fn render(&mut self, ui: &mut egui::Ui, session: &dyn DebugSession) {
    // Get memory from the session
    let memory = session.memory(self.address..self.address + 256);

    if let Some(bytes) = memory {
        // Render as hex dump
        for (offset, chunk) in bytes.chunks(16).enumerate() {
            let addr = self.address + (offset * 16) as u64;

            // Address column
            ui.monospace(format!("0x{:016x}", addr));

            // Hex bytes
            for byte in chunk {
                ui.monospace(format!("{:02x} ", byte));
            }

            // ASCII column
            for byte in chunk {
                let c = if byte.is_ascii_graphic() {
                    *byte as char
                } else {
                    '.'
                };
                ui.monospace(c);
            }
        }
    }
}
```

**Call Stack Panel Example:**

```rust
pub fn render(&mut self, ui: &mut egui::Ui, session: &dyn DebugSession) {
    let frames = session.stack_frames();

    for (i, frame) in frames.iter().enumerate() {
        let is_current = i == 0; // Top frame is current

        ui.horizontal(|ui| {
            // Frame number
            if is_current {
                ui.colored_label(GREEN, "▶");
            } else {
                ui.label(" ");
            }
            ui.label(format!("#{}", i));

            // Function name
            ui.monospace(&frame.function_name);

            // Source location
            if let Some(loc) = &frame.source_location {
                ui.label(format!("at {}:{}", loc.file, loc.line));
            }

            // Module/crate badge
            ui.colored_label(GRAY, &frame.module);
        });

        // Click to select frame
        if ui.button("Select").clicked() {
            self.selected_frame = frame.id;
        }
    }
}
```

### Why This Works

**For UI Developer (AI or human):**
- Just read the trait and structs
- Render the data however makes sense
- No need to understand ptrace, DWARF, MIR, etc.

**For Core Developer (you):**
- Implement `DebugSession` trait
- Fill in `Variable` structs with ownership info
- UI automatically displays it correctly

**For Testing:**
- Create `MockDebugSession` with fake data
- UI works identically with mock or real data
- Parallel development!

---

## egui Immediate Mode Rendering

Understanding **immediate mode** is key to understanding the UI code.

### Traditional UI (Retained Mode)

```
Traditional GUI (like Qt, GTK):

1. Create widgets:
   button = Button("Click me")
   button.set_position(100, 100)
   window.add_child(button)

2. Event loop:
   forever:
       event = get_next_event()
       if event.target == button:
           handle_button_click()
       render_all_widgets()  # Widgets remember their state

3. Update widgets:
   button.set_text("Clicked!")  # Mutate widget state
```

### Immediate Mode (egui)

```
egui (Immediate Mode):

1. No persistent widgets - just code that runs each frame

2. Event loop (60fps):
   forever:
       frame_start()

       // YOU WRITE THIS - it runs every frame:
       if ui.button("Click me").clicked() {
           println!("Clicked!");
       }

       frame_end()  # egui figures out what changed and renders

3. All state lives in YOUR structs, not in the GUI
```

**Example:**

```rust
pub struct MyApp {
    counter: i32,  // <-- State lives HERE
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // This function is called 60 times per second

        egui::CentralPanel::default().show(ctx, |ui| {
            // Declarative: "there should be a label here"
            ui.label(format!("Counter: {}", self.counter));

            // Declarative: "there should be a button here"
            // .clicked() returns true if button was clicked THIS frame
            if ui.button("Increment").clicked() {
                self.counter += 1;  // Mutate OUR state, not widget state
            }
        });

        // Next frame, this function runs again from scratch
        // egui compares with last frame and updates only what changed
    }
}
```

**Why Immediate Mode for Anteater?**

1. **State synchronization is trivial:** When debug state changes, just update `session` and the UI automatically shows new data next frame

2. **No widget lifecycle bugs:** Can't forget to destroy a widget or update its state

3. **Easy to reason about:** `update()` describes the entire UI, always

4. **Performance:** egui is highly optimized; only redraws changed regions

### How Anteater Uses Immediate Mode

```rust
impl eframe::App for AnteaterApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // This runs 60 times per second

        // 1. Menu bar (always at top)
        egui::TopBottomPanel::top("menu").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("Debug", |ui| {
                    if ui.button("Continue (F5)").clicked() {
                        // Send continue command
                    }
                    if ui.button("Step Over (F10)").clicked() {
                        // Send step command
                    }
                });

                ui.menu_button("View", |ui| {
                    if ui.button("Change Theme...").clicked() {
                        self.show_theme_selector = true;
                    }
                });
            });
        });

        // 2. Docking area (takes remaining space)
        DockArea::new(&mut self.dock_state)
            .style(Style::from_egui(ctx.style().as_ref()))
            .show(ctx, &mut TabViewer {
                session: &self.session,
                panels: &mut self.panels,
            });

        // 3. Theme selector dialog (if open)
        if self.show_theme_selector {
            egui::Window::new("Select Theme").show(ctx, |ui| {
                for theme in &self.themes {
                    if ui.button(&theme.name).clicked() {
                        self.apply_theme(ctx, theme.clone());
                        self.show_theme_selector = false;
                    }
                }
            });
        }

        // 4. Keyboard shortcuts
        ctx.input(|i| {
            if i.key_pressed(egui::Key::F5) {
                println!("Continue (will send to debug engine)");
            }
            if i.key_pressed(egui::Key::F10) {
                println!("Step Over (will send to debug engine)");
            }
        });

        // Frame complete - egui renders changes
    }
}
```

Every frame (60fps), this entire function runs. It describes what the UI should look like RIGHT NOW based on current state.

---

## Panel System Deep Dive

Anteater uses **egui_dock** for its docking panel system. Here's how it works:

### Panel Types

```rust
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
```

This enum represents every possible panel. It's:
- `Copy` so we can pass it around freely
- `Hash + Eq` so egui_dock can identify panels
- `Debug` for development

### Panel Instances

```rust
struct PanelInstances {
    variables: variables::VariablesPanel,
    call_stack: call_stack::CallStackPanel,
    registers: registers::RegistersPanel,
    memory: memory::MemoryPanel,
    disassembly: disassembly::DisassemblyPanel,
    breakpoints: breakpoints::BreakpointsPanel,
    source: source::SourcePanel,
}
```

**Key insight:** Each panel has **persistent state** that lives across frames:

```rust
pub struct MemoryPanel {
    address: u64,           // Current address being viewed
    bytes_per_row: usize,   // 16
    visible_rows: usize,    // 32
    scroll_offset: f32,     // Scroll position
}
```

If we recreated `MemoryPanel` every frame, it would reset to address 0x0 constantly. By storing instances in `PanelInstances`, state persists.

### The TabViewer Bridge

egui_dock needs to know how to render each panel. We implement the `TabViewer` trait:

```rust
struct TabViewer<'a> {
    session: &'a dyn DebugSession,
    panels: &'a mut PanelInstances,
}

impl egui_dock::TabViewer for TabViewer<'_> {
    type Tab = PanelType;  // Our panel enum

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        // Return title for the tab
        match tab {
            PanelType::Variables => "Variables".into(),
            PanelType::Memory => "Memory".into(),
            // ...
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        // Render the actual panel content
        match tab {
            PanelType::Variables => {
                self.panels.variables.render(ui, self.session)
            }
            PanelType::Memory => {
                self.panels.memory.render(ui, self.session)
            }
            PanelType::CallStack => {
                self.panels.call_stack.render(ui, self.session)
            }
            // ... etc for all panels
        }
    }
}
```

**Flow:**

```
Each frame:
  DockArea::new(&mut dock_state).show(ctx, &mut tab_viewer)
    ├─> For each visible tab:
    │     ├─> Call tab_viewer.title(&mut panel_type)  // Get tab name
    │     └─> Call tab_viewer.ui(ui, &mut panel_type) // Render content
    │           └─> Match on panel_type
    │                 └─> Call panels.memory.render(ui, session)
    │                       └─> Memory panel reads from session
    │                             └─> Calls egui primitives (ui.label, etc.)
```

### Docking State

```rust
pub struct AnteaterApp {
    dock_state: DockState<PanelType>,
    // ...
}
```

`DockState<PanelType>` is a tree structure representing the current layout:

```
Initial layout (all tabs):

dock_state = Surface {
    root: Node::Leaf {
        tabs: [Source, Variables, CallStack, Registers, ...]
    }
}
```

After user drags Source panel to split left:

```
dock_state = Surface {
    root: Node::Horizontal {
        fraction: 0.7,  // 70% left, 30% right
        left: Node::Leaf {
            tabs: [Source]
        },
        right: Node::Leaf {
            tabs: [Variables, CallStack, Registers, ...]
        }
    }
}
```

After user drags Variables to split top-right:

```
dock_state = Surface {
    root: Node::Horizontal {
        fraction: 0.7,
        left: Node::Leaf {
            tabs: [Source]
        },
        right: Node::Vertical {
            fraction: 0.6,
            top: Node::Leaf {
                tabs: [Variables]
            },
            bottom: Node::Leaf {
                tabs: [CallStack, Registers, ...]
            }
        }
    }
}
```

**egui_dock automatically:**
- Manages this tree
- Handles drag-and-drop
- Draws split bars
- Saves/restores during session

**We just:**
- Provide the initial layout
- Implement rendering (via TabViewer)
- Can query/modify state if needed

### Panel Reopening

When user closes a panel, it's removed from the tree. We added a menu to reopen:

```rust
fn is_panel_open(&self, panel_type: PanelType) -> bool {
    // Walk the dock_state tree looking for this panel
    self.dock_state
        .main_surface()
        .iter()  // Iterates all nodes
        .any(|node| {
            if let Some(tabs) = node.tabs() {
                tabs.iter().any(|tab| *tab == panel_type)
            } else {
                false
            }
        })
}

fn open_panel(&mut self, panel_type: PanelType) {
    if !self.is_panel_open(panel_type) {
        // Add to the currently focused leaf
        self.dock_state
            .main_surface_mut()
            .push_to_focused_leaf(panel_type);
    }
}
```

In menu:

```rust
ui.menu_button("Panels", |ui| {
    for panel_type in PanelType::all() {
        let is_open = self.is_panel_open(panel_type);
        let text = if is_open {
            format!("✓ {}", panel_type.title())
        } else {
            format!("  {}", panel_type.title())
        };

        if ui.button(text).clicked() {
            if !is_open {
                self.open_panel(panel_type);
            }
        }
    }
});
```

---

## Event Loop and State Management

Let's trace a complete frame from start to finish:

```
TIME: Frame N starts (16.6ms after frame N-1, targeting 60fps)

┌────────────────────────────────────────────────────────────────────┐
│ 1. OS EVENT COLLECTION (eframe handles this)                      │
├────────────────────────────────────────────────────────────────────┤
│ - Mouse moved to (523, 341)                                        │
│ - Mouse button 1 clicked                                           │
│ - Key F10 pressed                                                  │
│ - Window needs repaint                                             │
└────────────────────────────────────────────────────────────────────┘
                          │
                          ▼
┌────────────────────────────────────────────────────────────────────┐
│ 2. EGUI FRAME START                                                │
├────────────────────────────────────────────────────────────────────┤
│ ctx.begin_frame()                                                  │
│ - Clear frame state                                                │
│ - Process input events                                             │
│ - Prepare for widget declarations                                  │
└────────────────────────────────────────────────────────────────────┘
                          │
                          ▼
┌────────────────────────────────────────────────────────────────────┐
│ 3. YOUR update() FUNCTION RUNS                                     │
├────────────────────────────────────────────────────────────────────┤
│ impl eframe::App for AnteaterApp {                                 │
│     fn update(&mut self, ctx: &egui::Context, frame: ...) {       │
│                                                                    │
│         // 3a. Check keyboard shortcuts                           │
│         ctx.input(|i| {                                            │
│             if i.key_pressed(egui::Key::F10) {                     │
│                 println!("Step Over!");                            │
│                 // TODO: self.send_command(DebugCommand::StepOver)│
│             }                                                      │
│         });                                                        │
│                                                                    │
│         // 3b. Render menu bar                                    │
│         egui::TopBottomPanel::top("menu").show(ctx, |ui| {        │
│             egui::menu::bar(ui, |ui| {                             │
│                 ui.menu_button("Debug", |ui| {                     │
│                     if ui.button("Continue (F5)").clicked() {     │
│                         println!("Continue!");                     │
│                     }                                              │
│                 });                                                │
│             });                                                    │
│         });                                                        │
│                                                                    │
│         // 3c. Render docking area                                │
│         DockArea::new(&mut self.dock_state)                        │
│             .show(ctx, &mut TabViewer {                            │
│                 session: &self.session,                            │
│                 panels: &mut self.panels,                          │
│             });                                                    │
│             // This calls TabViewer::ui() for each visible panel  │
│                                                                    │
│         // 3d. Render dialogs                                     │
│         if self.show_theme_selector {                              │
│             egui::Window::new("Select Theme")                      │
│                 .show(ctx, |ui| { /* theme buttons */ });          │
│         }                                                          │
│     }                                                              │
│ }                                                                  │
│                                                                    │
│ During this function:                                              │
│ - egui records all ui.label(), ui.button(), etc. calls            │
│ - Checks which widgets were interacted with                        │
│ - Returns interaction results (.clicked(), .changed(), etc.)       │
└────────────────────────────────────────────────────────────────────┘
                          │
                          ▼
┌────────────────────────────────────────────────────────────────────┐
│ 4. EGUI TESSELLATION (converting UI to graphics primitives)       │
├────────────────────────────────────────────────────────────────────┤
│ ctx.end_frame() produces:                                          │
│                                                                    │
│ PaintJobs = [                                                      │
│     // Background                                                  │
│     Rect { pos: (0,0), size: (1920, 1080), color: #1e1e1e },      │
│                                                                    │
│     // Menu bar text "Debug"                                      │
│     Text {                                                         │
│         pos: (10, 5),                                              │
│         text: "Debug",                                             │
│         font: "sans-serif 14px",                                   │
│         color: #ffffff,                                            │
│     },                                                             │
│                                                                    │
│     // Variables panel content                                    │
│     Text { pos: (100, 50), text: "data", ... },                   │
│     Text { pos: (200, 50), text: "Vec<i32>", ... },               │
│     Rect { pos: (180, 48), size: (40, 18), color: #1565c0 },      │
│     Text { pos: (185, 50), text: "&", color: white },             │
│                                                                    │
│     // ... thousands more primitives                              │
│ ]                                                                  │
│                                                                    │
│ Also produces:                                                     │
│ - Which areas changed since last frame (for dirty rect culling)   │
│ - Cursor shape                                                     │
│ - Textures (for icons, images, cached text glyphs)                │
└────────────────────────────────────────────────────────────────────┘
                          │
                          ▼
┌────────────────────────────────────────────────────────────────────┐
│ 5. EFRAME RENDERING (backend-specific)                            │
├────────────────────────────────────────────────────────────────────┤
│ eframe::backend::glow (OpenGL backend):                            │
│                                                                    │
│ for paint_job in paint_jobs {                                     │
│     match paint_job {                                              │
│         Rect => {                                                  │
│             glBindBuffer(vertex_buffer);                           │
│             glBufferData([x, y, x+w, y, x+w, y+h, x, y+h]);       │
│             glDrawArrays(GL_TRIANGLE_FAN, ...);                    │
│         }                                                          │
│         Text => {                                                  │
│             let texture = glyph_cache.get(text, font);             │
│             glBindTexture(texture);                                │
│             glDrawArrays(...);                                     │
│         }                                                          │
│     }                                                              │
│ }                                                                  │
│                                                                    │
│ glSwapBuffers(); // Present frame to screen                        │
└────────────────────────────────────────────────────────────────────┘
                          │
                          ▼
┌────────────────────────────────────────────────────────────────────┐
│ 6. FRAME COMPLETE                                                  │
├────────────────────────────────────────────────────────────────────┤
│ Frame time: 4.2ms (well under 16.6ms budget)                      │
│ FPS: 60                                                            │
│                                                                    │
│ Wait for next frame or next event...                              │
└────────────────────────────────────────────────────────────────────┘

TIME: Frame N+1 starts (16.6ms later)
      Entire process repeats
```

### State Updates

**Question:** If `update()` runs every frame, how does state change?

**Answer:** We mutate `self`:

```rust
impl eframe::App for AnteaterApp {
    fn update(&mut self, ctx: &egui::Context, ...) {
        //        ^^^^ mutable reference to self

        // When user clicks theme:
        if theme_button_clicked {
            self.current_theme = new_theme;  // Mutate state
            self.apply_theme(ctx, &self.current_theme);
        }

        // When user types in memory address:
        ui.text_edit_singleline(&mut self.panels.memory.address);
        //                       ^^^^ egui mutates this String directly

        // When debug engine sends us new data:
        if let Some(msg) = self.debug_rx.try_recv() {
            match msg {
                DebugEvent::Stopped { reason } => {
                    self.session.update_state();  // Refresh session data
                    self.panels.source.scroll_to_current_line();
                }
            }
        }

        // Next frame, update() runs again with new state
        // UI automatically reflects changes
    }
}
```

### Asynchronous Updates

**Question:** Debug operations are slow. How do we not block the UI?

**Answer:** Channels + background thread (future work, but here's the design):

```rust
pub struct AnteaterApp {
    session: Arc<RwLock<DebugSession>>,  // Shared with debug thread
    command_tx: mpsc::Sender<DebugCommand>,
    event_rx: mpsc::Receiver<DebugEvent>,
    // ...
}

// In update():
fn update(&mut self, ctx: &egui::Context, ...) {
    // 1. Check for new events from debug thread (non-blocking)
    while let Ok(event) = self.event_rx.try_recv() {
        match event {
            DebugEvent::Stopped { location } => {
                // Update UI state based on event
            }
            DebugEvent::VariablesUpdated => {
                // Refresh variables panel
            }
        }
    }

    // 2. Render UI normally (always responsive)
    DockArea::new(&mut self.dock_state).show(ctx, ...);

    // 3. If user presses F10 (step over):
    if step_over_pressed {
        // Send command to debug thread (non-blocking)
        self.command_tx.send(DebugCommand::StepOver).unwrap();

        // UI continues running at 60fps
        // When step completes, debug thread sends DebugEvent::Stopped
        // Next frame, we handle it in step 1 above
    }

    // Request repaint so we check event_rx again next frame
    ctx.request_repaint();
}
```

Debug thread (runs independently):

```rust
fn debug_thread(
    command_rx: mpsc::Receiver<DebugCommand>,
    event_tx: mpsc::Sender<DebugEvent>,
    session: Arc<RwLock<DebugSession>>,
) {
    loop {
        // Wait for command from UI
        let command = command_rx.recv().unwrap();

        match command {
            DebugCommand::StepOver => {
                // This might take 100ms (ptrace operations)
                // UI is not blocked - running at 60fps in parallel
                session.write().unwrap().step_over();

                // Notify UI that step completed
                event_tx.send(DebugEvent::Stopped {
                    location: session.read().unwrap().current_location(),
                }).unwrap();
            }

            DebugCommand::Continue => {
                // This blocks until breakpoint hit
                session.write().unwrap().continue_execution();

                event_tx.send(DebugEvent::Stopped { ... }).unwrap();
            }
        }
    }
}
```

**Result:** UI never blocks, always 60fps, responsive even during slow ptrace operations.

---

## Mock Data System

Since the real debug engine doesn't exist yet, we built a complete mock implementation:

### MockDebugSession

```rust
pub struct MockDebugSession {
    // Mock call stack
    stack_frames: Vec<StackFrame>,

    // Mock variables for each frame
    variables: HashMap<FrameId, Vec<Variable>>,

    // Mock CPU registers
    registers: RegisterSet,

    // Mock 4KB of memory
    memory: Vec<u8>,

    // Mock source code
    source_files: HashMap<SourcePath, SourceFile>,

    // Current execution location
    current_location: SourceLocation,

    // Mock breakpoints
    breakpoints: Vec<Breakpoint>,
}

impl DebugSession for MockDebugSession {
    fn variables(&self, frame: FrameId) -> &[Variable] {
        self.variables.get(&frame).unwrap()
    }

    fn stack_frames(&self) -> &[StackFrame] {
        &self.stack_frames
    }

    fn registers(&self) -> &RegisterSet {
        &self.registers
    }

    fn memory(&self, range: Range<u64>) -> Option<&[u8]> {
        let start = range.start as usize;
        let end = range.end as usize;
        self.memory.get(start..end)
    }

    // ... etc for all trait methods
}
```

### Realistic Mock Data

We don't just return empty vectors - we create **realistic** data that exercises all UI features:

```rust
impl MockDebugSession {
    pub fn new() -> Self {
        // Create realistic call stack
        let stack_frames = vec![
            StackFrame {
                id: FrameId(0),
                function_name: "process_request".to_string(),
                source_location: Some(SourceLocation {
                    file: "src/main.rs".into(),
                    line: 42,
                    column: Some(12),
                }),
                module: "anteater_example".to_string(),
                is_inline: false,
            },
            StackFrame {
                id: FrameId(1),
                function_name: "handle_connection".to_string(),
                source_location: Some(SourceLocation {
                    file: "src/server.rs".into(),
                    line: 156,
                    column: Some(8),
                }),
                module: "anteater_example".to_string(),
                is_inline: false,
            },
            // ... more frames
        ];

        // Create realistic variables with ALL ownership states
        let variables = vec![
            // Normal owned variable
            Variable {
                name: "data".to_string(),
                rust_type: TypeDisplay::new("Vec<u8>"),
                value: ValueDisplay::new("Vec(len=1024, cap=2048)"),
                ownership: OwnershipState::Owned,
                address: Some(0x7fff_1234_5678),
                children: vec![
                    Variable {
                        name: "ptr".to_string(),
                        rust_type: TypeDisplay::new("*const u8"),
                        value: ValueDisplay::new("0x55aa_0000_1000"),
                        ownership: OwnershipState::Owned,
                        address: Some(0x7fff_1234_5678),
                        children: vec![],
                        is_optimized_out: false,
                        enum_variant: None,
                    },
                    Variable {
                        name: "len".to_string(),
                        rust_type: TypeDisplay::new("usize"),
                        value: ValueDisplay::new("1024"),
                        ownership: OwnershipState::Owned,
                        address: Some(0x7fff_1234_5680),
                        children: vec![],
                        is_optimized_out: false,
                        enum_variant: None,
                    },
                    Variable {
                        name: "cap".to_string(),
                        rust_type: TypeDisplay::new("usize"),
                        value: ValueDisplay::new("2048"),
                        ownership: OwnershipState::Owned,
                        address: Some(0x7fff_1234_5688),
                        children: vec![],
                        is_optimized_out: false,
                        enum_variant: None,
                    },
                ],
                is_optimized_out: false,
                enum_variant: None,
            },

            // Borrowed variable
            Variable {
                name: "borrowed_data".to_string(),
                rust_type: TypeDisplay::new("&Vec<u8>"),
                value: ValueDisplay::new("&data"),
                ownership: OwnershipState::Borrowed {
                    by: vec!["reader".to_string()],
                },
                address: Some(0x7fff_1234_5690),
                children: vec![],
                is_optimized_out: false,
                enum_variant: None,
            },

            // Moved variable
            Variable {
                name: "moved_string".to_string(),
                rust_type: TypeDisplay::new("String"),
                value: ValueDisplay::new("(moved)"),
                ownership: OwnershipState::MovedFrom {
                    moved_to: Some("new_owner".to_string()),
                },
                address: Some(0x7fff_1234_5698),
                children: vec![],
                is_optimized_out: false,
                enum_variant: None,
            },

            // Mutably borrowed
            Variable {
                name: "buffer".to_string(),
                rust_type: TypeDisplay::new("&mut [u8]"),
                value: ValueDisplay::new("&mut [0; 256]"),
                ownership: OwnershipState::MutablyBorrowed {
                    by: "writer".to_string(),
                },
                address: Some(0x7fff_1234_56a0),
                children: vec![],
                is_optimized_out: false,
                enum_variant: None,
            },

            // Enum with active variant
            Variable {
                name: "result".to_string(),
                rust_type: TypeDisplay::new("Result<i32, String>"),
                value: ValueDisplay::new("Ok(42)"),
                ownership: OwnershipState::Owned,
                address: Some(0x7fff_1234_56b0),
                enum_variant: Some("Ok".to_string()),
                children: vec![
                    Variable {
                        name: "0".to_string(),  // Tuple variant field
                        rust_type: TypeDisplay::new("i32"),
                        value: ValueDisplay::new("42"),
                        ownership: OwnershipState::Owned,
                        address: Some(0x7fff_1234_56b4),
                        children: vec![],
                        is_optimized_out: false,
                        enum_variant: None,
                    },
                ],
                is_optimized_out: false,
            },

            // Optimized out variable
            Variable {
                name: "optimized_away".to_string(),
                rust_type: TypeDisplay::new("i32"),
                value: ValueDisplay::new(""),
                ownership: OwnershipState::Unknown {
                    reason: Some("optimized out".to_string()),
                },
                address: None,
                children: vec![],
                is_optimized_out: true,
                enum_variant: None,
            },
        ];

        // Create realistic memory (4KB)
        let mut memory = vec![0u8; 4096];

        // Put some strings in memory
        memory[0x100..0x10d].copy_from_slice(b"Hello, World!");
        memory[0x200..0x215].copy_from_slice(b"Anteater Debugger");

        // Put some numbers
        memory[0x300..0x304].copy_from_slice(&42i32.to_le_bytes());
        memory[0x304..0x308].copy_from_slice(&3.14f32.to_le_bytes());

        // Create realistic source file
        let source_content = r#"
use std::io::{self, Read};

fn process_request(data: Vec<u8>) -> Result<String, io::Error> {
    let mut borrowed_data = &data;
    let mut buffer = vec![0u8; 256];

    // Parse request
    if borrowed_data.len() < 4 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Request too short"
        ));
    }

    let result = Ok(42); // <-- Current line (breakpoint here)

    Ok(format!("Processed {} bytes", data.len()))
}

fn main() {
    let data = vec![1, 2, 3, 4, 5];
    match process_request(data) {
        Ok(msg) => println!("{}", msg),
        Err(e) => eprintln!("Error: {}", e),
    }
}
"#;

        let mut source_files = HashMap::new();
        source_files.insert(
            SourcePath::from("src/main.rs"),
            SourceFile {
                path: "src/main.rs".into(),
                content: source_content.to_string(),
            },
        );

        // Create realistic registers
        let registers = RegisterSet {
            rax: 0x0000_0000_0000_002a, // 42
            rbx: 0x7fff_1234_5678,
            rcx: 0x0000_0000_0000_0003,
            rdx: 0x0000_0000_0000_0005,
            rsi: 0x7fff_1234_5690,
            rdi: 0x7fff_1234_56a0,
            rbp: 0x7fff_1234_5800,
            rsp: 0x7fff_1234_5600,
            r8:  0x0000_0000_0000_0000,
            r9:  0x0000_0000_0000_0000,
            r10: 0x0000_0000_0000_0000,
            r11: 0x0000_0000_0000_0000,
            r12: 0x0000_0000_0000_0000,
            r13: 0x0000_0000_0000_0000,
            r14: 0x0000_0000_0000_0000,
            r15: 0x0000_0000_0000_0000,
            rip: 0x5555_5555_1234, // instruction pointer
            rflags: 0x0000_0000_0000_0202, // FLAGS register
        };

        // Create mock breakpoints
        let breakpoints = vec![
            Breakpoint {
                id: BreakpointId(1),
                location: BreakpointLocation::SourceLine {
                    file: "src/main.rs".into(),
                    line: 15,
                },
                enabled: true,
                condition: None,
                hit_count: 3,
            },
            Breakpoint {
                id: BreakpointId(2),
                location: BreakpointLocation::Address(0x5555_5555_2000),
                enabled: false,
                condition: Some("data.len() > 100".to_string()),
                hit_count: 0,
            },
        ];

        Self {
            stack_frames,
            variables: [(FrameId(0), variables)].iter().cloned().collect(),
            registers,
            memory,
            source_files,
            current_location: SourceLocation {
                file: "src/main.rs".into(),
                line: 15,
                column: Some(12),
            },
            breakpoints,
        }
    }
}
```

**Why This Matters:**

1. **UI can be fully tested** without real debug engine
2. **All ownership states are represented** - we know UI handles them
3. **Edge cases covered** - optimized out, empty values, deep nesting
4. **Realistic data sizes** - 1024-element Vec, 4KB memory, etc.

When you replace `MockDebugSession` with `RealDebugSession`, if it implements the same trait, UI just works.

---

## Keyboard Shortcuts and Commands

### Current Implementation (Mock)

```rust
impl eframe::App for AnteaterApp {
    fn update(&mut self, ctx: &egui::Context, ...) {
        // Check keyboard input every frame
        ctx.input(|i| {
            if i.key_pressed(egui::Key::F5) {
                println!("Continue (F5) pressed");
                // TODO: self.send_command(DebugCommand::Continue);
            }

            if i.key_pressed(egui::Key::F10) {
                println!("Step Over (F10) pressed");
                // TODO: self.send_command(DebugCommand::StepOver);
            }

            if i.key_pressed(egui::Key::F11) {
                println!("Step Into (F11) pressed");
                // TODO: self.send_command(DebugCommand::StepInto);
            }

            if i.modifiers.shift && i.key_pressed(egui::Key::F11) {
                println!("Step Out (Shift+F11) pressed");
                // TODO: self.send_command(DebugCommand::StepOut);
            }

            if i.key_pressed(egui::Key::F9) {
                println!("Toggle Breakpoint (F9) pressed");
                // TODO: toggle breakpoint at current line
            }
        });
    }
}
```

### Future Implementation (Real)

```rust
pub enum DebugCommand {
    Continue,
    StepOver,
    StepInto,
    StepOut,
    Pause,
    ToggleBreakpoint { file: SourcePath, line: u32 },
    AddBreakpoint { location: BreakpointLocation },
    RemoveBreakpoint { id: BreakpointId },
    Restart,
    Stop,
}

pub enum DebugEvent {
    Stopped { reason: StopReason, location: SourceLocation },
    Continued,
    ProcessExited { code: i32 },
    BreakpointHit { id: BreakpointId },
    VariablesUpdated,
    MemoryChanged { range: Range<u64> },
}

pub struct AnteaterApp {
    session: Arc<RwLock<DebugSession>>,
    command_tx: mpsc::Sender<DebugCommand>,
    event_rx: mpsc::Receiver<DebugEvent>,
    // ...
}

impl eframe::App for AnteaterApp {
    fn update(&mut self, ctx: &egui::Context, ...) {
        // 1. Process events from debug thread
        while let Ok(event) = self.event_rx.try_recv() {
            match event {
                DebugEvent::Stopped { reason, location } => {
                    // Update current location
                    self.current_location = location;

                    // Scroll source panel to current line
                    self.panels.source.scroll_to_line(location.line);

                    // Refresh variables
                    // (session is updated by debug thread)
                }
                DebugEvent::BreakpointHit { id } => {
                    // Flash the breakpoint panel
                    self.panels.breakpoints.highlight(id);
                }
                // ... etc
            }
        }

        // 2. Handle keyboard shortcuts
        ctx.input(|i| {
            if i.key_pressed(egui::Key::F5) {
                self.send_command(DebugCommand::Continue);
            }

            if i.key_pressed(egui::Key::F10) {
                self.send_command(DebugCommand::StepOver);
            }

            if i.key_pressed(egui::Key::F9) {
                // Toggle breakpoint at current line
                if let Some(loc) = &self.current_location {
                    self.send_command(DebugCommand::ToggleBreakpoint {
                        file: loc.file.clone(),
                        line: loc.line,
                    });
                }
            }
        });

        // 3. Render UI
        DockArea::new(&mut self.dock_state).show(ctx, ...);

        // 4. Request repaint to check for events next frame
        ctx.request_repaint();
    }
}

impl AnteaterApp {
    fn send_command(&self, command: DebugCommand) {
        self.command_tx.send(command).expect("Debug thread died");
    }
}
```

Debug thread:

```rust
fn debug_thread(
    command_rx: mpsc::Receiver<DebugCommand>,
    event_tx: mpsc::Sender<DebugEvent>,
    session: Arc<RwLock<DebugSession>>,
) {
    loop {
        let command = command_rx.recv().unwrap();

        match command {
            DebugCommand::Continue => {
                let mut session = session.write().unwrap();
                session.continue_execution(); // Blocks until breakpoint

                let location = session.current_location();
                event_tx.send(DebugEvent::Stopped {
                    reason: StopReason::Breakpoint,
                    location,
                }).unwrap();
            }

            DebugCommand::StepOver => {
                let mut session = session.write().unwrap();
                session.step_over();

                let location = session.current_location();
                event_tx.send(DebugEvent::Stopped {
                    reason: StopReason::Step,
                    location,
                }).unwrap();
            }

            DebugCommand::ToggleBreakpoint { file, line } => {
                let mut session = session.write().unwrap();

                if let Some(bp) = session.find_breakpoint(&file, line) {
                    session.remove_breakpoint(bp.id);
                } else {
                    session.add_breakpoint(BreakpointLocation::SourceLine {
                        file, line
                    });
                }
            }

            // ... etc
        }
    }
}
```

**Flow:**

```
User presses F10:
  ├─> ctx.input() detects key press
  ├─> send_command(DebugCommand::StepOver)
  ├─> command_tx sends to debug thread
  ├─> UI continues rendering at 60fps
  │
  Debug thread:
  ├─> Receives StepOver command
  ├─> Calls ptrace(PTRACE_SINGLESTEP, ...)
  ├─> Waits for process to stop (blocks here, 100ms)
  ├─> Reads new registers, memory, variables
  ├─> Updates session (Arc<RwLock<DebugSession>>)
  ├─> Sends DebugEvent::Stopped back to UI
  │
  Next UI frame:
  ├─> event_rx.try_recv() gets Stopped event
  ├─> Updates current_location
  ├─> UI renders with new data
  └─> User sees updated state
```

UI never blocks, always responsive.

---

## Theme System

Anteater supports **iTerm2 color schemes** (the `.itermcolors` XML format).

### iTerm2 Theme Format

```xml
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN"
  "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <!-- Foreground color -->
    <key>Foreground Color</key>
    <dict>
        <key>Red Component</key>
        <real>0.972549</real>
        <key>Green Component</key>
        <real>0.972549</real>
        <key>Blue Component</key>
        <real>0.94901961</real>
    </dict>

    <!-- Background color -->
    <key>Background Color</key>
    <dict>
        <key>Red Component</key>
        <real>0.15686275</real>
        <key>Green Component</key>
        <real>0.16470588</real>
        <key>Blue Component</key>
        <real>0.21176471</real>
    </dict>

    <!-- 16 ANSI colors (Ansi 0 through Ansi 15) -->
    <key>Ansi 0 Color</key>
    <dict>
        <key>Red Component</key>
        <real>0.0</real>
        <key>Green Component</key>
        <real>0.0</real>
        <key>Blue Component</key>
        <real>0.0</real>
    </dict>

    <!-- ... Ansi 1 through Ansi 15 ... -->
</dict>
</plist>
```

Colors are RGB in 0.0-1.0 range (not 0-255).

### Parsing iTerm2 Themes

```rust
use plist::Dictionary;

pub struct ITerm2Theme {
    pub name: String,
    pub foreground: Color32,
    pub background: Color32,
    pub cursor: Color32,
    pub selection: Color32,
    pub ansi_colors: [Color32; 16],
}

impl ITerm2Theme {
    pub fn from_itermcolors(path: &Path) -> Result<Self, String> {
        let file = std::fs::File::open(path)
            .map_err(|e| format!("Failed to open file: {}", e))?;

        let dict: Dictionary = plist::from_reader(file)
            .map_err(|e| format!("Failed to parse plist: {}", e))?;

        Ok(ITerm2Theme {
            name: path.file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("Unnamed")
                .to_string(),
            foreground: parse_color(&dict, "Foreground Color")?,
            background: parse_color(&dict, "Background Color")?,
            cursor: parse_color(&dict, "Cursor Color")?,
            selection: parse_color(&dict, "Selection Color")?,
            ansi_colors: [
                parse_color(&dict, "Ansi 0 Color")?,
                parse_color(&dict, "Ansi 1 Color")?,
                parse_color(&dict, "Ansi 2 Color")?,
                // ... through Ansi 15
            ],
        })
    }
}

fn parse_color(dict: &Dictionary, key: &str) -> Result<Color32, String> {
    let color_dict = dict.get(key)
        .and_then(|v| v.as_dictionary())
        .ok_or_else(|| format!("Missing key: {}", key))?;

    let r = color_dict.get("Red Component")
        .and_then(|v| v.as_real())
        .ok_or("Missing Red Component")? as f32;

    let g = color_dict.get("Green Component")
        .and_then(|v| v.as_real())
        .ok_or("Missing Green Component")? as f32;

    let b = color_dict.get("Blue Component")
        .and_then(|v| v.as_real())
        .ok_or("Missing Blue Component")? as f32;

    Ok(Color32::from_rgb(
        (r * 255.0) as u8,
        (g * 255.0) as u8,
        (b * 255.0) as u8,
    ))
}
```

### Built-in Themes

Instead of shipping .itermcolors files, we have built-in themes:

```rust
impl ITerm2Theme {
    pub fn default_dark() -> Self {
        ITerm2Theme {
            name: "Default Dark".to_string(),
            foreground: Color32::from_rgb(220, 220, 220),
            background: Color32::from_rgb(30, 30, 30),
            cursor: Color32::from_rgb(255, 255, 255),
            selection: Color32::from_rgb(70, 70, 90),
            ansi_colors: [
                Color32::from_rgb(0, 0, 0),         // Black
                Color32::from_rgb(205, 49, 49),     // Red
                Color32::from_rgb(13, 188, 121),    // Green
                Color32::from_rgb(229, 229, 16),    // Yellow
                Color32::from_rgb(36, 114, 200),    // Blue
                Color32::from_rgb(188, 63, 188),    // Magenta
                Color32::from_rgb(17, 168, 205),    // Cyan
                Color32::from_rgb(229, 229, 229),   // White
                Color32::from_rgb(102, 102, 102),   // Bright Black
                Color32::from_rgb(241, 76, 76),     // Bright Red
                Color32::from_rgb(35, 209, 139),    // Bright Green
                Color32::from_rgb(245, 245, 67),    // Bright Yellow
                Color32::from_rgb(59, 142, 234),    // Bright Blue
                Color32::from_rgb(214, 112, 214),   // Bright Magenta
                Color32::from_rgb(41, 184, 219),    // Bright Cyan
                Color32::from_rgb(255, 255, 255),   // Bright White
            ],
        }
    }

    pub fn dracula() -> Self {
        ITerm2Theme {
            name: "Dracula".to_string(),
            foreground: Color32::from_rgb(248, 248, 242),
            background: Color32::from_rgb(40, 42, 54),
            cursor: Color32::from_rgb(248, 248, 242),
            selection: Color32::from_rgb(68, 71, 90),
            ansi_colors: [
                Color32::from_rgb(0, 0, 0),
                Color32::from_rgb(255, 85, 85),
                Color32::from_rgb(80, 250, 123),
                Color32::from_rgb(241, 250, 140),
                Color32::from_rgb(189, 147, 249),
                Color32::from_rgb(255, 121, 198),
                Color32::from_rgb(139, 233, 253),
                Color32::from_rgb(255, 255, 255),
                // Bright colors...
            ],
        }
    }

    // nord(), one_dark(), gruvbox_dark() similarly...
}
```

### Applying Themes to egui

```rust
impl AnteaterApp {
    fn apply_theme(&mut self, ctx: &egui::Context, theme: &ITerm2Theme) {
        let mut visuals = egui::Visuals::dark();

        // Background
        visuals.panel_fill = theme.background;
        visuals.window_fill = theme.background;
        visuals.extreme_bg_color = theme.background;

        // Foreground text
        visuals.override_text_color = Some(theme.foreground);

        // Widget colors (use ANSI colors)
        visuals.widgets.inactive.bg_fill = theme.ansi_colors[0];
        visuals.widgets.inactive.fg_stroke.color = theme.foreground;

        visuals.widgets.hovered.bg_fill = theme.ansi_colors[8]; // Bright black
        visuals.widgets.hovered.fg_stroke.color = theme.foreground;

        visuals.widgets.active.bg_fill = theme.ansi_colors[4]; // Blue
        visuals.widgets.active.fg_stroke.color = theme.ansi_colors[15]; // Bright white

        // Selection
        visuals.selection.bg_fill = theme.selection;
        visuals.selection.stroke.color = theme.foreground;

        // Hyperlinks
        visuals.hyperlink_color = theme.ansi_colors[12]; // Bright blue

        // Apply to context
        ctx.set_visuals(visuals);

        // Save current theme
        self.current_theme = theme.clone();
    }
}
```

### Ownership State Colors (Mapping to ANSI)

```rust
pub fn ownership_colors(state: &OwnershipState, theme: &ITerm2Theme)
    -> (Color32, Color32)  // (background, foreground)
{
    match state {
        OwnershipState::Owned => {
            (Color32::TRANSPARENT, theme.foreground)
        }
        OwnershipState::MovedFrom { .. } => {
            (Color32::TRANSPARENT, theme.ansi_colors[8]) // Gray
        }
        OwnershipState::Borrowed { .. } => {
            (theme.ansi_colors[4].gamma_multiply(0.2),   // Blue bg, dim
             theme.ansi_colors[12])                      // Blue fg, bright
        }
        OwnershipState::MutablyBorrowed { .. } => {
            (theme.ansi_colors[3].gamma_multiply(0.2),   // Yellow/Orange bg
             theme.ansi_colors[11])                      // Bright yellow/orange
        }
        OwnershipState::Dropped => {
            (Color32::TRANSPARENT, theme.ansi_colors[8]) // Gray
        }
        OwnershipState::Uninitialized => {
            (Color32::TRANSPARENT, theme.ansi_colors[8]) // Gray
        }
        OwnershipState::PartiallyMoved { .. } => {
            (theme.ansi_colors[3].gamma_multiply(0.2),   // Yellow
             theme.ansi_colors[11])
        }
        OwnershipState::Unknown { .. } => {
            (Color32::TRANSPARENT, theme.ansi_colors[8]) // Gray
        }
    }
}
```

**Usage in panels:**

```rust
fn render_variable(ui: &mut egui::Ui, var: &Variable, theme: &ITerm2Theme) {
    let (bg, fg) = ownership_colors(&var.ownership, theme);

    ui.horizontal(|ui| {
        // Variable name with ownership-based color
        let name_text = if use_strikethrough(&var.ownership) {
            RichText::new(&var.name).strikethrough().color(fg)
        } else {
            RichText::new(&var.name).color(fg)
        };

        ui.label(name_text);

        // Ownership badge
        if bg != Color32::TRANSPARENT {
            ui.colored_label(fg, badge_text(&var.ownership));
        }

        // Type and value
        ui.label(&var.rust_type);
        ui.label(&var.value);
    });
}
```

---

## Memory Layout Examples

Let's trace a concrete example through the entire system:

### Rust Code Being Debugged

```rust
fn main() {
    let mut data = vec![1u8, 2, 3, 4, 5];    // Line 2
    let borrowed = &data;                     // Line 3 <-- stopped here
    let mut_borrowed = &mut data[0];         // Line 4
    println!("{:?}", borrowed);              // Line 5
}
```

### Process Memory Layout

```
Process virtual memory (simplified):

0x5555_5555_0000 - 0x5555_5555_3000    .text (code)
0x5555_5555_3000 - 0x5555_5555_4000    .rodata (constants)
0x5555_5555_4000 - 0x5555_5555_5000    .data (globals)
0x5555_5555_5000 - 0x5555_5555_6000    .bss (uninitialized globals)

0x7fff_0000_0000 - 0x7fff_ffff_ffff    Stack (grows down)

0x55aa_0000_0000 - 0x55aa_ffff_ffff    Heap (grows up)


Stack frame for main() (at line 3):

High addresses
┌─────────────────────────────────────┐
│  Return address to __libc_start     │ RBP + 8
├─────────────────────────────────────┤
│  Saved RBP                           │ RBP (0x7fff_1234_5800)
├─────────────────────────────────────┤
│                                      │
│  Local variables:                    │
│                                      │
│  mut_borrowed: &mut u8               │ RBP - 8
│    = (uninitialized, line 4 not hit) │
│                                      │
│  borrowed: &Vec<u8>                  │ RBP - 16 (0x7fff_1234_57f0)
│    = 0x7fff_1234_57e0 ───────┐      │
│                              │       │
│  data: Vec<u8>              ◄┘      │ RBP - 40 (0x7fff_1234_57e0)
│    ├─ ptr: *const u8               │
│    │  = 0x55aa_0000_1000 ─────┐    │
│    ├─ len: usize             │     │
│    │  = 5                     │     │
│    └─ cap: usize             │     │
│       = 5                    │     │
└─────────────────────────────│───────┘
Low addresses (ESP)           │
                              │
                              │
Heap:                         │
                              │
0x55aa_0000_1000 ◄────────────┘
┌─────────────────────────────────────┐
│  [01 02 03 04 05]                   │  Vec data
└─────────────────────────────────────┘
```

### DWARF Debug Info

```
.debug_info section (simplified):

DW_TAG_subprogram
  DW_AT_name: "main"
  DW_AT_low_pc: 0x5555_5555_1234
  DW_AT_high_pc: 0x5555_5555_1298

  DW_TAG_variable
    DW_AT_name: "data"
    DW_AT_type: -> DW_TAG_structure_type "Vec<u8>"
    DW_AT_location: DW_OP_fbreg -40  (RBP - 40)

  DW_TAG_variable
    DW_AT_name: "borrowed"
    DW_AT_type: -> DW_TAG_reference_type "&Vec<u8>"
    DW_AT_location: DW_OP_fbreg -16  (RBP - 16)

  DW_TAG_variable
    DW_AT_name: "mut_borrowed"
    DW_AT_type: -> DW_TAG_reference_type "&mut u8"
    DW_AT_location: DW_OP_fbreg -8   (RBP - 8)

DW_TAG_structure_type
  DW_AT_name: "Vec<u8>"

  DW_TAG_member
    DW_AT_name: "ptr"
    DW_AT_type: -> DW_TAG_pointer_type "*const u8"
    DW_AT_data_member_location: 0

  DW_TAG_member
    DW_AT_name: "len"
    DW_AT_type: -> DW_TAG_base_type "usize"
    DW_AT_data_member_location: 8

  DW_TAG_member
    DW_AT_name: "cap"
    DW_AT_type: -> DW_TAG_base_type "usize"
    DW_AT_data_member_location: 16
```

DWARF knows:
- Variable names and types
- Memory locations (RBP offsets)
- Struct layouts

DWARF does NOT know:
- That `data` is borrowed
- That the borrow is to `borrowed`
- That `data` owns the heap allocation

### MIR (Mid-level IR)

```
fn main() -> () {
    let mut _0: ();
    let mut _1: std::vec::Vec<u8>;        // data
    let _2: &std::vec::Vec<u8>;           // borrowed
    let _3: &mut u8;                      // mut_borrowed

    bb0: {
        _1 = Vec::<u8>::new();
        _1 = Vec::<u8>::push_within_capacity(move _1, const 1_u8);
        _1 = Vec::<u8>::push_within_capacity(move _1, const 2_u8);
        // ... pushes 3, 4, 5
        goto -> bb1;
    }

    bb1: {
        _2 = &_1;                         // <-- Borrow created here!
        goto -> bb2;                      //     THIS is what we need
    }

    bb2: {
        _3 = &mut (*_1)[const 0_usize];   // <-- Mut borrow here
        goto -> bb3;
    }

    // ...
}
```

MIR knows:
- `_2 = &_1` means borrowed borrows data
- Borrow relationships explicit
- Move operations explicit

### Correlation (The Magic)

```
Anteater Engine does this:

1. Read DWARF: "Variable 'data' is at RBP-40, type Vec<u8>"

2. Read MIR: "_1 = Vec::<u8>::new(); _2 = &_1;"

3. Correlate: "DWARF variable 'data' = MIR local '_1'"
   (Match by name, type, and scope)

4. Track: "At line 3 (bb1), _2 = &_1, so 'borrowed' borrows 'data'"

5. Current PC: 0x5555_5555_1240 (line 3)
   MIR state: bb1 complete, bb2 not yet executed

6. Infer ownership:
   - data: Borrowed { by: ["borrowed"] }
   - borrowed: Owned (it owns the reference)
   - mut_borrowed: Uninitialized (line 4 not hit yet)
```

### ViewModel Generated

```rust
Variable {
    name: "data".to_string(),
    rust_type: TypeDisplay::new("Vec<u8>"),
    value: ValueDisplay::new("Vec(len=5, cap=5)"),
    ownership: OwnershipState::Borrowed {
        by: vec!["borrowed".to_string()],
    },
    address: Some(0x7fff_1234_57e0),
    children: vec![
        Variable {
            name: "ptr".to_string(),
            rust_type: TypeDisplay::new("*const u8"),
            value: ValueDisplay::new("0x55aa00001000"),
            ownership: OwnershipState::Owned,
            address: Some(0x7fff_1234_57e0),
            children: vec![],
            is_optimized_out: false,
            enum_variant: None,
        },
        Variable {
            name: "len".to_string(),
            rust_type: TypeDisplay::new("usize"),
            value: ValueDisplay::new("5"),
            ownership: OwnershipState::Owned,
            address: Some(0x7fff_1234_57e8),
            children: vec![],
            is_optimized_out: false,
            enum_variant: None,
        },
        Variable {
            name: "cap".to_string(),
            rust_type: TypeDisplay::new("usize"),
            value: ValueDisplay::new("5"),
            ownership: OwnershipState::Owned,
            address: Some(0x7fff_1234_57f0),
            children: vec![],
            is_optimized_out: false,
            enum_variant: None,
        },
    ],
    is_optimized_out: false,
    enum_variant: None,
}

Variable {
    name: "borrowed".to_string(),
    rust_type: TypeDisplay::new("&Vec<u8>"),
    value: ValueDisplay::new("&data"),
    ownership: OwnershipState::Owned,  // The reference itself is owned
    address: Some(0x7fff_1234_57f0),
    children: vec![],
    is_optimized_out: false,
    enum_variant: None,
}

Variable {
    name: "mut_borrowed".to_string(),
    rust_type: TypeDisplay::new("&mut u8"),
    value: ValueDisplay::new(""),
    ownership: OwnershipState::Uninitialized,
    address: Some(0x7fff_1234_57f8),
    children: vec![],
    is_optimized_out: false,
    enum_variant: None,
}
```

### UI Rendering

Variables Panel shows:

```
┌──────────────────────────────────────────────┐
│ Variables                                [×] │
├──────────────────────────────────────────────┤
│                                              │
│ [▼] data         Vec<u8>      Vec(len=5)    │
│     [&] borrowed by 'borrowed'               │
│                                              │
│     └─ [▶] ptr      *const u8  0x55aa...    │
│        [▶] len      usize      5             │
│        [▶] cap      usize      5             │
│                                              │
│ borrowed         &Vec<u8>     &data          │
│                                              │
│ mut_borrowed     &mut u8      (uninitialized)│
│                  [—]                         │
│                                              │
└──────────────────────────────────────────────┘
```

**Color coding:**
- `data` has blue accent (borrowed)
- `[&]` badge in blue
- `mut_borrowed` is grayed out (uninitialized)

Memory Panel shows (if user navigates to 0x55aa_0000_1000):

```
┌──────────────────────────────────────────────┐
│ Memory                                   [×] │
├──────────────────────────────────────────────┤
│                                              │
│ Address: [0x55aa00001000         ] [Go]      │
│                                              │
│ 0x55aa00001000  01 02 03 04 05 00 00 00  .....   │
│ 0x55aa00001008  00 00 00 00 00 00 00 00  ........│
│                                              │
└──────────────────────────────────────────────┘
```

---

## Complete Request/Response Example

Let's trace user clicking "Step Over" through the entire system:

```
┌────────────────────────────────────────────────────────────────────┐
│ USER                                                               │
│ - Presses F10 (Step Over)                                          │
└────────────┬───────────────────────────────────────────────────────┘
             │
             ▼
┌────────────────────────────────────────────────────────────────────┐
│ EGUI EVENT PROCESSING                                              │
│ - Captures F10 key press event                                     │
│ - Stores in input state for this frame                             │
└────────────┬───────────────────────────────────────────────────────┘
             │
             ▼
┌────────────────────────────────────────────────────────────────────┐
│ AnteaterApp::update()                                              │
│                                                                    │
│ ctx.input(|i| {                                                    │
│     if i.key_pressed(egui::Key::F10) {                             │
│         self.send_command(DebugCommand::StepOver);  // <--         │
│     }                                                              │
│ });                                                                │
└────────────┬───────────────────────────────────────────────────────┘
             │
             ▼
┌────────────────────────────────────────────────────────────────────┐
│ Command Channel (mpsc)                                             │
│ - DebugCommand::StepOver sent to debug thread                      │
│ - UI thread continues (doesn't block)                              │
└────────────┬───────────────────────────────────────────────────────┘
             │
             ├──> UI thread: continues rendering at 60fps
             │    (shows "stepping..." indicator maybe)
             │
             └──> Debug thread receives command:
                  ▼
┌────────────────────────────────────────────────────────────────────┐
│ Debug Thread                                                       │
│                                                                    │
│ match command_rx.recv() {                                          │
│     DebugCommand::StepOver => {                                    │
│         let mut session = session.write().unwrap();                │
│         session.step_over();  // <-- Calls into anteater-engine   │
│     }                                                              │
│ }                                                                  │
└────────────┬───────────────────────────────────────────────────────┘
             │
             ▼
┌────────────────────────────────────────────────────────────────────┐
│ ANTEATER-ENGINE: step_over()                                       │
│                                                                    │
│ 1. Get current instruction address (RIP)                           │
│    rip = self.core.read_register(Register::RIP);                  │
│    // rip = 0x5555_5555_1240                                       │
│                                                                    │
│ 2. Read instruction at RIP                                         │
│    instr = self.core.read_memory(rip, 15);                         │
│    // instr = [0xe8, 0x12, 0x34, 0x56, 0x78]  (CALL instruction)  │
│                                                                    │
│ 3. Decode instruction                                              │
│    if instr.is_call() {                                            │
│        // Step over = run until return from call                   │
│        next_instr = rip + instr.length();                          │
│        self.set_temp_breakpoint(next_instr);                       │
│        self.core.continue_execution();                             │
│        self.wait_for_stop();                                       │
│        self.remove_temp_breakpoint(next_instr);                    │
│    } else {                                                        │
│        // Just step one instruction                                │
│        self.core.single_step();                                    │
│    }                                                               │
└────────────┬───────────────────────────────────────────────────────┘
             │
             ▼
┌────────────────────────────────────────────────────────────────────┐
│ ANTEATER-CORE: ptrace operations                                   │
│                                                                    │
│ pub fn single_step(&mut self) -> Result<()> {                     │
│     unsafe {                                                       │
│         ptrace(PTRACE_SINGLESTEP, self.pid, null_mut(), null_mut())│
│     };                                                             │
│     self.waitpid()?;  // Wait for process to stop                  │
│     Ok(())                                                         │
│ }                                                                  │
│                                                                    │
│ // This blocks for ~100ms while debuggee executes one instruction  │
└────────────┬───────────────────────────────────────────────────────┘
             │
             ▼
┌────────────────────────────────────────────────────────────────────┐
│ Debugged Process                                                   │
│ - Executes ONE instruction                                         │
│ - Stops due to PTRACE_SINGLESTEP                                   │
│ - Kernel sends SIGTRAP to debugger                                 │
└────────────┬───────────────────────────────────────────────────────┘
             │
             ▼
┌────────────────────────────────────────────────────────────────────┐
│ ANTEATER-ENGINE: After step completes                              │
│                                                                    │
│ 1. Update cached state                                             │
│    self.current_regs = self.core.read_registers();                │
│    self.current_location = self.dwarf.location_from_pc(            │
│        self.current_regs.rip                                       │
│    );                                                              │
│                                                                    │
│ 2. Re-analyze ownership state                                      │
│    let mir_state = self.mir.execution_state(                       │
│        self.current_location.line                                  │
│    );                                                              │
│    self.variables = self.correlate_variables(mir_state);           │
│                                                                    │
│ 3. Send event to UI thread                                         │
│    event_tx.send(DebugEvent::Stopped {                             │
│        reason: StopReason::Step,                                   │
│        location: self.current_location.clone(),                    │
│    }).unwrap();                                                    │
└────────────┬───────────────────────────────────────────────────────┘
             │
             ▼
┌────────────────────────────────────────────────────────────────────┐
│ Event Channel (mpsc)                                               │
│ - DebugEvent::Stopped sent back to UI thread                       │
└────────────┬───────────────────────────────────────────────────────┘
             │
             ▼
┌────────────────────────────────────────────────────────────────────┐
│ AnteaterApp::update() — Next Frame                                │
│                                                                    │
│ // Check for events from debug thread                             │
│ while let Ok(event) = self.event_rx.try_recv() {                  │
│     match event {                                                  │
│         DebugEvent::Stopped { location, .. } => {                  │
│             // Update UI state                                     │
│             self.current_location = location;                      │
│             self.panels.source.scroll_to_line(location.line);      │
│             // session was updated by debug thread via Arc         │
│         }                                                          │
│     }                                                              │
│ }                                                                  │
│                                                                    │
│ // Render UI with new state                                       │
│ DockArea::new(&mut self.dock_state).show(ctx, &mut TabViewer {    │
│     session: &self.session,  // <-- Has new data now              │
│     panels: &mut self.panels,                                      │
│ });                                                                │
└────────────┬───────────────────────────────────────────────────────┘
             │
             ▼
┌────────────────────────────────────────────────────────────────────┐
│ Panels Render                                                      │
│                                                                    │
│ SourcePanel:                                                       │
│ - Highlights line 4 (new current line)                             │
│ - Was on line 3, now on line 4                                     │
│                                                                    │
│ VariablesPanel:                                                    │
│ - Shows new variable state                                         │
│ - "mut_borrowed" now shows as Owned (was Uninitialized)            │
│ - "data" now shows as MutablyBorrowed { by: "mut_borrowed" }       │
│                                                                    │
│ RegistersPanel:                                                    │
│ - Highlights RIP (changed from 0x...1240 to 0x...1248)             │
│ - Maybe other registers changed                                    │
└────────────┬───────────────────────────────────────────────────────┘
             │
             ▼
┌────────────────────────────────────────────────────────────────────┐
│ USER SEES                                                          │
│ - Source panel advanced to next line                               │
│ - Variables updated with new ownership states                      │
│ - Entire operation felt instant (~100ms total)                     │
│ - UI never froze (was rendering at 60fps the whole time)           │
└────────────────────────────────────────────────────────────────────┘
```

**Timeline:**

```
T+0ms:    User presses F10
T+0ms:    egui captures event
T+0ms:    update() detects key, sends command
T+0ms:    UI continues rendering (60fps)
T+1ms:    Debug thread receives command
T+1ms:    Calls ptrace(SINGLESTEP)
T+1-100ms: Debugged process executes one instruction
T+100ms:  ptrace returns, step complete
T+100ms:  Engine updates variables, sends event
T+101ms:  UI thread receives event in next frame
T+101ms:  UI re-renders with new state
T+101ms:  User sees result

Total perceived latency: ~100ms
UI froze for: 0ms (never blocked)
```

---

## Summary: Key Takeaways

### 1. Clean Architecture

```
UI (anteater-ui)
  └─> Only depends on ViewModel (anteater-ui-types)
      └─> Implemented by Engine (anteater-engine)
          └─> Uses Core (anteater-core)
```

Each layer has ONE job. Boundaries are enforced by Rust's module system.

### 2. Immediate Mode is Simple

```rust
// Every frame:
fn update(&mut self, ctx, frame) {
    // Describe UI
    ui.label("Hello");
    if ui.button("Click").clicked() {
        self.state += 1;
    }

    // egui handles rendering
}
```

No widget lifecycle, no retained state in UI framework, just pure functions of data.

### 3. ViewModel is the Contract

```rust
pub trait DebugSession {
    fn variables(&self, frame: FrameId) -> &[Variable];
    // ...
}
```

UI doesn't know about ptrace, DWARF, MIR. Just reads ViewModel and renders it.

### 4. Async is via Channels

```rust
// UI thread:
command_tx.send(DebugCommand::StepOver);
ctx.request_repaint();  // Keep checking for events

// Debug thread:
do_slow_operation();
event_tx.send(DebugEvent::Stopped);

// Next UI frame:
while let Ok(event) = event_rx.try_recv() {
    handle(event);
}
```

UI never blocks, always responsive.

### 5. Ownership is the Innovation

```rust
pub enum OwnershipState {
    Owned,
    Borrowed { by: Vec<String> },
    MovedFrom { moved_to: Option<String> },
    MutablyBorrowed { by: String },
    // ...
}
```

This is what no other debugger can show. The engine correlates MIR + DWARF to fill this in, UI makes it legible.

### 6. Mock Data Enables Development

```rust
impl DebugSession for MockDebugSession { ... }
impl DebugSession for RealDebugSession { ... }

// UI doesn't care which:
let session: Box<dyn DebugSession> = ...;
```

Parallel development: UI built with mock, core built independently, integrate seamlessly.

---

## Files Quick Reference

**Core architecture:**
- `Cargo.toml` — Workspace manifest
- `crates/anteater-ui-types/src/lib.rs` — **THE CONTRACT** (85 lines, read this first)
- `crates/anteater-ui/src/app.rs` — Main app, docking, event loop
- `crates/anteater/src/main.rs` — Entry point (20 lines)

**UI implementation:**
- `crates/anteater-ui/src/mock.rs` — Mock debug session
- `crates/anteater-ui/src/theme.rs` — iTerm2 theme support
- `crates/anteater-ui/src/panels/*.rs` — All 7 panels
- `crates/anteater-ui/src/widgets/*.rs` — Reusable UI components

**Documentation:**
- `docs/ARCHITECTURE.md` — High-level overview
- `docs/UI_DEVELOPMENT_GUIDE.md` — UI coding conventions
- `docs/VISUAL_LANGUAGE.md` — Ownership state design
- `docs/HOW_IT_WORKS.md` — **This file** (deep technical dive)
- `docs/WORKSPACE_STRUCTURE.md` — Crate organization
- `docs/WORK_LOG.md` — Session-by-session progress

**Placeholders (you build these):**
- `crates/anteater-core/src/lib.rs` — ptrace, DWARF (future)
- `crates/anteater-engine/src/lib.rs` — MIR, correlation (future)

---

**Total lines of code (current):**
- UI types: ~300 lines
- UI implementation: ~2500 lines
- Documentation: ~3000 lines
- **Total: ~5800 lines**

**What works now:**
- Full UI with mock data
- All panels functional
- Docking system
- Themes
- Keyboard shortcuts
- 60fps rendering

**What's next:**
- You: Build anteater-core (ptrace, DWARF)
- You: Build anteater-engine (MIR, correlation)
- Integration: Replace MockDebugSession with RealDebugSession
- Polish: Layout persistence, more themes, command palette

---

This is Anteater. A clean, modular, well-documented foundation ready for the hard part: making Rust semantics visible during debugging.

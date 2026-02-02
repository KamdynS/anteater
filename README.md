# Anteater

A Rust-aware debugger that understands ownership, borrowing, and lifetimes.

## What is Anteater?

Anteater is a graphical debugger for Rust that goes beyond traditional debuggers by understanding Rust's semantics. By correlating MIR (Mid-level Intermediate Representation) with DWARF debug information, Anteater can answer questions that no other debugger can:

- **Has this variable been moved?**
- **What's currently borrowing this value?**
- **Why can't I access this variable?**
- **Which match arm executed and why?**

## Current Status

**UI Layer:** ✅ Complete and functional
**Core/Engine:** 🚧 In development

The UI is fully implemented with mock data and ready to be connected to the debug engine.

## Features

### Implemented ✅

- **Docking Panel System**: Drag-and-drop panels, fully customizable layout
- **Syntax-Highlighted Source View**: Rust code with breakpoint gutters, current line indicator
- **Memory Viewer**: Hex dump with ASCII sidebar, address navigation
- **Variable Inspector**: Tree view with ownership states (Owned, Moved, Borrowed, etc.)
- **Register View**: CPU registers with change highlighting
- **Call Stack**: Stack frames with source locations
- **Disassembly**: Machine code with current instruction highlighting
- **Breakpoint Management**: List view with conditions and hit counts
- **Keyboard Shortcuts**: F5 (Continue), F10 (Step Over), F11 (Step Into), F9 (Toggle Breakpoint)
- **iTerm2 Theme Support**: Full theme customization support
- **Visual Language**: Clear ownership state indicators

### In Progress 🚧

- **Debug Core**: ptrace control, DWARF parsing (you're building this)
- **Semantic Layer**: MIR correlation, ownership tracking (you're building this)

### Planned 🎯

- **Borrow Visualization**: Visual arrows showing borrow relationships
- **Pattern Match State**: Show which match arm executed and why
- **Command Palette**: Fuzzy command search (Cmd/Ctrl+P)
- **Layout Persistence**: Save/load workspace configurations
- **JetBrains Mono Integration**: Embedded font (currently uses system font)

## Quick Start

```bash
# Build and run
cargo run -p anteater

# Build specific crate
cargo build -p anteater-ui

# Run tests
cargo test --workspace
```

## Project Structure

```
anteater/
├── docs/               # Comprehensive documentation
│   ├── SESSION_SUMMARY.md    # Complete session overview
│   ├── ARCHITECTURE.md       # System design
│   ├── THEMES.md            # Theme customization guide
│   └── ...
└── crates/
    ├── anteater-ui-types/    # ViewModel (UI contract)
    ├── anteater-ui/          # UI implementation (complete)
    ├── anteater-engine/      # Semantic layer (your work)
    ├── anteater-core/        # Debug core (your work)
    └── anteater/             # Main binary
```

## Architecture

```
┌─────────────────────────────────────────────┐
│          UI Layer (egui)                    │  ← ✅ Complete
│  Panels, docking, syntax highlighting       │
└──────────────────┬──────────────────────────┘
                   │ ViewModel types
┌──────────────────▼──────────────────────────┐
│        Semantic Layer                       │  ← 🚧 In Progress
│  MIR correlation, ownership tracking        │
└──────────────────┬──────────────────────────┘
                   │
┌──────────────────▼──────────────────────────┐
│         Debug Core                          │  ← 🚧 In Progress
│  ptrace, DWARF parsing, memory access       │
└─────────────────────────────────────────────┘
```

## Documentation

- **[SESSION_SUMMARY.md](docs/SESSION_SUMMARY.md)** - Complete overview of what was built
- **[ARCHITECTURE.md](docs/ARCHITECTURE.md)** - System architecture and design
- **[UI_DEVELOPMENT_GUIDE.md](docs/UI_DEVELOPMENT_GUIDE.md)** - UI coding conventions
- **[VISUAL_LANGUAGE.md](docs/VISUAL_LANGUAGE.md)** - Ownership state visual design
- **[THEMES.md](docs/THEMES.md)** - Theme customization guide
- **[WORKSPACE_STRUCTURE.md](docs/WORKSPACE_STRUCTURE.md)** - Crate organization

## Technologies

- **UI**: [egui](https://github.com/emilk/egui) - Immediate mode GUI
- **Docking**: [egui_dock](https://github.com/Adanos020/egui_dock) - Panel system
- **Syntax Highlighting**: [syntect](https://github.com/trishume/syntect) - Rust highlighting
- **Themes**: [plist](https://github.com/ebarnard/rust-plist) - iTerm2 theme parsing

## Try It Out

The UI is fully functional with mock data. Run it to see:

```bash
cargo run -p anteater
```

You'll see:
- Syntax-highlighted Rust source code (mock file)
- Variables with ownership states (owned, moved, borrowed)
- Memory hex dump with ASCII sidebar
- CPU registers with values
- Call stack with frames
- All panels draggable and rearrangeable

## Development

**For UI work:**
- All panels in `crates/anteater-ui/src/panels/`
- Mock data in `crates/anteater-ui/src/mock.rs`
- Add panels, modify layouts, experiment with UX

**For Core work:**
- `crates/anteater-core/` - ptrace, DWARF
- `crates/anteater-engine/` - MIR, ownership tracking
- Implement `DebugSession` interface from `anteater-ui-types`

## Contributing

See [docs/UI_DEVELOPMENT_GUIDE.md](docs/UI_DEVELOPMENT_GUIDE.md) for UI development conventions.

For core development, see integration notes in [docs/SESSION_SUMMARY.md](docs/SESSION_SUMMARY.md).

## License

MIT OR Apache-2.0

## Acknowledgments

- Inspired by [RAD Debugger](https://github.com/EpicGames/raddebugger)
- Built with [egui](https://github.com/emilk/egui)
- Theme support inspired by [Ghostty](https://github.com/ghostty-org/ghostty)

# Anteater Workspace Structure

## Overview

Anteater uses a modular Rust workspace architecture for clean separation of concerns and parallel development.

## Crate Organization

```
anteater/                        # Workspace root
├── Cargo.toml                   # Workspace manifest
├── docs/                        # Documentation
│   ├── README.md
│   ├── ARCHITECTURE.md
│   ├── WORKSPACE_STRUCTURE.md   # This file
│   └── ...
└── crates/
    ├── anteater-ui-types/       # ViewModel types (shared contract)
    ├── anteater-ui/             # UI layer (egui, panels, rendering)
    ├── anteater-engine/         # Semantic layer (MIR, ownership tracking)
    ├── anteater-core/           # Debug core (ptrace, DWARF)
    └── anteater/                # Final binary (composes all crates)
```

## Crate Dependencies

```
anteater (binary)
  ├─> anteater-ui
  │     └─> anteater-ui-types
  ├─> anteater-engine
  │     ├─> anteater-ui-types
  │     └─> anteater-core
  └─> anteater-core
```

## Development Philosophy

**Highly modular:** Each crate has a single, well-defined responsibility.

**Clean boundaries:** The `anteater-ui-types` crate defines the contract between UI and engine. UI never directly depends on `anteater-core`.

**Parallel development:** UI can be built against mock implementations while the core is being developed.

## Building

```bash
# Build entire workspace
cargo build

# Build specific crate
cargo build -p anteater-ui

# Run the debugger
cargo run -p anteater

# Check all crates
cargo check --workspace
```

## Testing

```bash
# Test all crates
cargo test --workspace

# Test specific crate
cargo test -p anteater-ui
```

## Design Rationale

### Why separate `anteater-ui-types`?

- **Stability:** UI and engine can evolve independently as long as they agree on the types
- **Testability:** UI can be tested with mock data, engine can be tested without rendering
- **Clarity:** The ViewModel layer makes the contract explicit

### Why not monorepo?

We ARE a monorepo (workspace), but with clear module boundaries. This gives us:
- Fast incremental compilation
- Shared dependencies (via workspace.dependencies)
- Ability to version crates independently later if needed

## Future Expansion

As the project grows, we may add:
- `anteater-mir` - MIR parsing and analysis (split from engine)
- `anteater-dwarf` - DWARF utilities (split from core)
- `anteater-ptrace` - ptrace abstraction (split from core)
- `anteater-cli` - Command-line interface
- `anteater-rpc` - Remote debugging protocol

The modular structure makes this easy.

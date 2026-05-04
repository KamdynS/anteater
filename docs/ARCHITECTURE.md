# Anteater Architecture

## Overview

Anteater is a graphical debugger for Rust that understands ownership, borrowing, and lifetimes by correlating MIR with DWARF debug info.

```
┌─────────────────────────────────────────┐
│           UI Layer (egui)               │  ← Complete
└──────────────────┬──────────────────────┘
                   │ ViewModel types
┌──────────────────▼──────────────────────┐
│          Semantic Layer                 │  ← In progress
│   MIR correlation, ownership tracking   │
└──────────────────┬──────────────────────┘
                   │
┌──────────────────▼──────────────────────┐
│           Debug Core                    │  ← In progress
│    ptrace, DWARF parsing, memory read   │
└─────────────────────────────────────────┘
```

## Key Decisions

- **Platform:** Linux x86-64 only (ptrace-based)
- **UI:** egui (immediate mode, Rust-native)
- **Separation:** ViewModel types (`anteater-ui-types`) define the UI/core contract

## Why MIR Matters

DWARF describes memory layout but not Rust semantics. Two variables at the same address after a move look identical in DWARF.

Anteater correlates MIR (where ownership operations are explicit) with DWARF (where addresses are concrete) to answer "has this variable been moved?" — a question no traditional debugger can answer.

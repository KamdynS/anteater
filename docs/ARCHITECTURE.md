# Anteater Architecture Overview

## Document Purpose

This document provides context for AI agents working on Anteater. It explains how the system fits together, what's being built by whom, and what assumptions are safe to make.

**Last updated:** Initial version  
**Status:** Living document — update as understanding evolves

---

## System Overview

Anteater is a graphical debugger for Rust that understands Rust semantics (ownership, borrowing, traits) rather than just memory addresses.

```
┌─────────────────────────────────────────────────────┐
│                    UI Layer (egui)                  │
│         Panels, views, interaction handling         │  ← AI agents help here
└─────────────────────┬───────────────────────────────┘
                      │ Consumes ViewModel types
┌─────────────────────▼───────────────────────────────┐
│                 Semantic Layer                      │
│    Rust-aware interpretation of debug state         │  ← Core developer builds
│    MIR correlation, ownership tracking, traits      │
└─────────────────────┬───────────────────────────────┘
                      │
┌─────────────────────▼───────────────────────────────┐
│                  Debug Core                         │
│      ptrace control, DWARF parsing, memory read     │  ← Core developer builds
└─────────────────────────────────────────────────────┘
```

## Division of Labor

**Core developer builds:**
- Debug Core (ptrace, DWARF parsing, process control)
- Semantic Layer (MIR parsing, MIR+DWARF correlation, ownership tracking)
- The ViewModel types that define the UI contract
- Novel UI features (borrow scope visualization, ownership futures)
- Overall app shell, layout, keyboard handling

**AI agents can build:**
- Standard debugger panels (memory view, registers, disassembly, call stack)
- Variable inspector (following the ViewModel spec)
- Breakpoint management UI
- Source code panel (basic rendering, syntax highlighting)
- General egui boilerplate and polish

## Key Insight: Why MIR Matters

Traditional debuggers use DWARF debug info, which describes memory layout but not Rust semantics. Two variables at the same address after a move look identical in DWARF.

Anteater correlates MIR (where ownership operations are explicit) with DWARF (where addresses are concrete). This lets us answer "has this variable been moved?" — a question no traditional debugger can answer.

See `mir_dwarf_design.md` in the project root for technical details.

## The ViewModel Contract

The file `ui_types.rs` defines the data structures the UI layer consumes. Key principles:

1. **UI is dumb.** It receives fully-interpreted data and renders it. No DWARF parsing, no MIR analysis, no ptrace calls in UI code.

2. **Types are provisional.** The semantic layer is under active development. Types will change. UI code should be structured to accommodate this.

3. **Ownership state is special.** The `OwnershipState` enum is the core innovation. UI must display these states clearly and consistently.

## Performance Requirements

From the project description:
> Performance is non-negotiable. A debugger that stutters or lags breaks flow state. Anteater must feel instant—UI at 60fps minimum, operations completing in milliseconds.

UI code must:
- Avoid allocations in the render loop where possible
- Not block on debug operations (the semantic layer handles async)
- Handle large data gracefully (thousands of variables, megabytes of memory)

## Platform Constraints

- Linux x86-64 only (for now)
- egui for UI framework
- Rust obviously

## What Success Looks Like

A Rust developer debugging their code should:
1. See at a glance whether a variable is owned, borrowed, or moved
2. Understand borrow relationships visually
3. Never feel like they're fighting the debugger
4. Have the standard debugger features work flawlessly

The UI's job is to make the semantic layer's insights *legible*.

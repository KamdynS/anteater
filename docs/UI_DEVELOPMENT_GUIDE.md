# UI Development Guide

## For AI Agents Building Anteater UI

This guide explains how to build UI components for Anteater. Follow these conventions to ensure your work integrates cleanly.

---

## Core Principles

### 1. The UI is a Pure Function of ViewModel

```rust
// Good: UI reads from ViewModel, renders it
fn render_variable_panel(ui: &mut egui::Ui, session: &DebugSession, frame: FrameId) {
    for var in session.variables(frame) {
        render_variable(ui, var);
    }
}

// Bad: UI reaching into internals or doing debug operations
fn render_variable_panel(ui: &mut egui::Ui, dwarf: &DwarfInfo, pid: Pid) {
    // NO — don't touch these layers
}
```

### 2. Use the Theme Module

Don't hardcode colors. Use `theme::ownership_colors()` and related functions.

```rust
// Good
let (bg, fg, label) = theme::ownership_colors(&var.ownership);

// Bad
let color = if var.ownership == OwnershipState::Owned { 
    Color32::GREEN  // Hardcoded
} else { ... };
```

### 3. Handle Missing/Error States Gracefully

Variables can be optimized out. Memory reads can fail. Always handle these cases.

```rust
fn render_variable(ui: &mut egui::Ui, var: &Variable) {
    if var.is_optimized_out {
        ui.add(egui::Label::new(
            RichText::new(&var.name).strikethrough().color(Color32::GRAY)
        ));
        ui.label("(optimized out)");
        return;
    }
    // Normal rendering...
}
```

### 4. Performance Matters

- Don't allocate strings in the render loop if avoidable
- Use `egui::ScrollArea` with `show_rows()` for long lists (virtual scrolling)
- Cache expensive computations outside the immediate mode loop

```rust
// Bad: Allocates every frame
ui.label(format!("0x{:016x}", address));

// Better: Pre-format in ViewModel, or use write! to a reusable buffer
```

### 5. Keyboard Navigation

The debugger should be keyboard-drivable. Add keyboard shortcuts for common actions.

```rust
if ui.input(|i| i.key_pressed(egui::Key::F5)) {
    commands.push(DebugCommand::Continue);
}
if ui.input(|i| i.key_pressed(egui::Key::F10)) {
    commands.push(DebugCommand::StepOver);
}
```

---

## Panel Specifications

### Memory View Panel

**Input:** Address range, `&[u8]` from `session.memory()`  
**Layout:** 
- Address column (hex, 16 bytes wide)
- Hex bytes (16 per row, grouped in 8s)
- ASCII interpretation (printable chars or '.')

**Interactions:**
- Click address to go to that location
- Scroll to navigate
- Highlight range (for showing a variable's memory)

**Performance:** Must handle megabytes. Use virtual scrolling.

### Register Panel

**Input:** `session.registers()`  
**Layout:** Table with Name | Hex Value | Decimal Value  
**Interactions:**
- Highlight registers that changed since last stop
- Click to copy value

### Disassembly Panel

**Input:** `session.disassembly(range)`  
**Layout:**
- Address | Bytes | Instruction
- Current instruction highlighted
- Breakpoint indicators in gutter

**Interactions:**
- Click to set/remove breakpoint
- Double-click to go to source (if available)

### Call Stack Panel

**Input:** `session.stack_frames()`  
**Layout:** List of frames, innermost at top  
**Interactions:**
- Click to select frame (updates variable view)
- Visual indicator for current frame
- Dim or badge inline frames

### Variable Inspector Panel

**Input:** `session.variables(frame_id)`  
**Layout:** Tree view with expandable nodes  
**Key features:**
- Ownership state badges (see theme module)
- Type display (possibly truncated for long types)
- Value preview
- Expand to see struct fields, enum contents

**This is the most important panel for Anteater's value proposition.**

### Source Panel

**Input:** `session.source_file(path)`, `session.current_source_location()`  
**Layout:**
- Line numbers in gutter
- Breakpoint indicators
- Current line highlight
- Syntax highlighting (use `syntect` or similar)

**Interactions:**
- Click gutter to toggle breakpoint
- Scroll to current line on stop

### Breakpoint Panel

**Input:** `session.breakpoints()`  
**Layout:** Table with Enable | Location | Condition | Hit Count  
**Interactions:**
- Toggle enable
- Delete
- Edit condition
- Click to go to location

---

## File Structure Convention

```
src/
  ui/
    mod.rs              # Re-exports, shared state
    app.rs              # Main app, panel layout, top-level keyboard handling
    panels/
      mod.rs
      memory.rs         # Memory view panel
      registers.rs      # Register panel
      disassembly.rs    # Disassembly panel
      stack.rs          # Call stack panel
      variables.rs      # Variable inspector
      source.rs         # Source code panel
      breakpoints.rs    # Breakpoint management
    widgets/
      mod.rs
      ownership_badge.rs    # Reusable ownership indicator
      type_display.rs       # Rust type formatting widget
      hex_view.rs          # Hex dump widget (used by memory panel)
```

---

## Testing UI Code

Since the UI consumes ViewModel types, you can test rendering with mock data:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    fn mock_variable() -> Variable {
        Variable {
            name: "test_var".to_string(),
            rust_type: TypeDisplay::new("Vec<u8>"),
            value: ValueDisplay::new("Vec(len=3)"),
            ownership: OwnershipState::Owned,
            children: vec![],
            address: Some(0x7fff_1234_5678),
            is_optimized_out: false,
            enum_variant: None,
        }
    }
    
    // Test that rendering doesn't panic, produces expected layout, etc.
}
```

---

## When You're Stuck

If you need information not in the ViewModel types:

1. **Check if it should be there.** Maybe it's an oversight in the types.
2. **Propose an addition.** Document what you need and why.
3. **Don't reach around the abstraction.** The separation exists for a reason.

If you're unsure whether a feature is in scope:

1. **Check ARCHITECTURE.md** for division of labor
2. **Check the project description** for stated features
3. **When in doubt, ask** rather than building something that won't integrate

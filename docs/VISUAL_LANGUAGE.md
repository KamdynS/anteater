# Visual Language: Ownership States

This document defines how ownership states should be displayed in the Anteater UI. All UI components should follow these conventions for consistency.

---

## Design Goals

1. **Glanceable:** A user should understand a variable's ownership state in <100ms
2. **Unobtrusive:** Owned (the common case) shouldn't be visually noisy
3. **Scannable:** In a list of 50 variables, problems should pop out
4. **Colorblind-accessible:** Don't rely on color alone; use shape/text/position too

---

## Ownership States

### Owned

The normal, happy state. Variable owns its value.

**Visual treatment:**
- No special indicator (clean, default appearance)
- *Or* subtle green dot/checkmark if user enables "always show ownership"
- Normal text color

**Rationale:** Most variables are owned most of the time. Making this visually "loud" would create noise.

### MovedFrom

Value has been moved out. The variable name exists but the value is gone.

**Visual treatment:**
- ~~Strikethrough text~~ on variable name
- Grayed out (reduced opacity)
- Badge: "moved" or "→" icon
- If `moved_to` is known: show as tooltip or inline "→ new_owner"

**Rationale:** Strikethrough universally communicates "no longer valid." Gray reinforces inaccessibility.

### Borrowed (shared)

Value is borrowed via `&T`.

**Visual treatment:**
- Blue accent (border, dot, or subtle highlight)
- Badge: "&" or "borrowed"  
- Show borrower: "borrowed by `x`" (inline or tooltip)

**Rationale:** Blue is conventionally "informational" — not an error, but worth knowing.

### MutablyBorrowed

Value is mutably borrowed via `&mut T`.

**Visual treatment:**
- Orange/amber accent (more prominent than blue)
- Badge: "&mut" or "mut borrowed"
- Show borrower: "mut borrowed by `x`"

**Rationale:** Mutable borrows are more significant (exclusive access). Orange signals "attention."

### Dropped

Destructor has run. Variable is dead.

**Visual treatment:**
- Grayed out (similar to MovedFrom)
- Badge: "dropped" or "☠" icon
- Strikethrough optional

**Rationale:** Similar to MovedFrom — the value is gone. User can distinguish via badge text.

### Uninitialized

Declared but never assigned: `let x: i32;`

**Visual treatment:**
- Grayed out
- Badge: "uninitialized" or "—"
- Value field shows "—" or empty

**Rationale:** This is unusual and potentially a bug. Gray signals "nothing here."

### PartiallyMoved

Some fields of a struct have been moved out.

**Visual treatment:**
- Yellow/amber accent
- Badge: "partial" 
- Expandable to show which fields remain vs moved

**Rationale:** This is a complex state that needs investigation. Yellow = caution.

### Unknown

Ownership state couldn't be determined.

**Visual treatment:**
- Gray with "?" badge
- Tooltip shows reason if available

**Rationale:** Honest about limitations. User knows they can't rely on ownership info here.

---

## Color Palette (Reference)

These are suggestions. Exact values should work with egui's theming.

| State | Background | Text/Accent | Badge |
|-------|------------|-------------|-------|
| Owned | transparent | default | (none) |
| MovedFrom | `#fafafa` | `#9e9e9e` | "moved" |
| Borrowed | `#e3f2fd` | `#1565c0` | "&" |
| MutablyBorrowed | `#fff3e0` | `#e65100` | "&mut" |
| Dropped | `#fafafa` | `#757575` | "dropped" |
| Uninitialized | `#fafafa` | `#bdbdbd` | "—" |
| PartiallyMoved | `#fff8e1` | `#ff8f00` | "partial" |
| Unknown | `#fafafa` | `#9e9e9e` | "?" |

---

## Iconography (If Using Icons)

If using icons instead of text badges:

| State | Icon Suggestion |
|-------|-----------------|
| Owned | ✓ (subtle, or nothing) |
| MovedFrom | → or ⊘ |
| Borrowed | ⟲ or chain link |
| MutablyBorrowed | ⟲ with emphasis |
| Dropped | ☠ or × |
| Uninitialized | — |
| PartiallyMoved | ◐ (half-filled circle) |
| Unknown | ? |

---

## Compound Display Examples

### Variable List

```
┌─────────────────────────────────────────────────┐
│ data         Vec<u8>              Vec(len=3)    │  ← Owned (clean)
│ ~~config~~   &mut Config          (moved)       │  ← MovedFrom (struck, gray)
│ & name       &str                 "alice"       │  ← Borrowed (blue accent)
│ &mut buffer  &mut [u8]            [0, 0, 0]     │  ← MutBorrowed (orange)
└─────────────────────────────────────────────────┘
```

### Inline with Borrower Info

```
items: Vec<Item>          Vec(len=5)
  └─ borrowed by `iter`
```

---

## Animation (Optional, Future)

If we add animation:
- Ownership changes could briefly highlight/flash
- Move operations could show value "traveling" to new location
- Borrow creation could draw connecting line

Keep animations subtle and fast (<200ms). They should aid understanding, not distract.

---

## Accessibility Notes

- Don't rely on color alone: use text badges, strikethrough, icons
- Ensure sufficient contrast (WCAG AA minimum)
- Strikethrough should be thick enough to be visible
- Consider offering high-contrast theme

---

## Implementation Notes

The `theme` module in `ui_types.rs` provides:
- `ownership_colors(state)` → color tuple
- `use_strikethrough(state)` → bool

UI code should use these rather than hardcoding, so we can tune the visual language in one place.

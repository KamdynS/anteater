# Open Questions

This document tracks questions that need resolution from the core developer or through experimentation. It helps AI agents know what's uncertain and avoid building on shaky assumptions.

---

## Format

Questions are tagged with:
- **Priority:** P1 (blocks work), P2 (impacts design), P3 (nice to clarify)
- **Domain:** Core, UI, Integration, UX
- **Status:** Open, Resolved, Deferred

---

## Open Questions

### Q001: Crate structure — where does UI live?

**Priority:** P2  
**Domain:** Integration  
**Status:** Open

**Question:** Should the UI be a separate crate or a module within the main crate? If separate, what's the crate graph look like?

**Options:**
1. Single crate, `src/ui/` module
2. Workspace with `anteater-core` and `anteater-ui` crates
3. Workspace with more fine-grained split (`anteater-dwarf`, `anteater-mir`, etc.)

**Impact:** Affects where `ui_types.rs` lives and how dependencies flow.

---

### Q002: How does the semantic layer expose data to UI?

**Priority:** P1  
**Domain:** Integration  
**Status:** Open

**Question:** Is `DebugSession` a concrete struct the semantic layer provides, or a trait the UI depends on? How is it accessed?

**Options:**
1. Concrete struct, UI holds `Arc<Mutex<DebugSession>>` or similar
2. Trait, semantic layer provides implementation
3. Message-passing between UI and debug threads

**Impact:** Affects how UI code is structured, how we handle async operations, testing strategy.

---

### Q003: Threading model — who owns the event loop?

**Priority:** P2  
**Domain:** Core  
**Status:** Open

**Question:** How do ptrace operations (which block) coexist with the egui render loop (which needs to stay responsive)?

**Possibilities:**
- Separate thread for ptrace, UI polls for updates
- Async runtime managing ptrace operations
- UI blocks on ptrace but operations are fast enough?

**Impact:** UI needs to know if it can assume synchronous access to debug state or needs to handle "loading" states.

---

### Q004: What happens when ownership info is unavailable?

**Priority:** P2  
**Domain:** UX  
**Status:** Open

**Question:** In optimized builds or complex scenarios, MIR correlation may fail. What should the UI show?

**Options:**
1. Fall back to traditional debugger view (no ownership info)
2. Show `OwnershipState::Unknown` everywhere with explanatory message
3. Partial info: show what we know, mark what we don't

**Impact:** Affects how UI handles the `Unknown` state and whether there's a "degraded mode" UX.

---

### Q005: Syntax highlighting approach

**Priority:** P3  
**Domain:** UI  
**Status:** Open

**Question:** What crate/approach for syntax highlighting Rust code in the source panel?

**Options:**
1. `syntect` — widely used, good Rust support
2. `tree-sitter-rust` — more structural, could enable semantic highlighting
3. Simple regex-based — fewer dependencies, probably good enough

**Impact:** Affects dependencies, quality of source panel.

---

### Q006: Configuration/settings persistence

**Priority:** P3  
**Domain:** UI  
**Status:** Open

**Question:** Where do user settings live (window size, panel layout, theme, etc.)? What format?

**Options:**
1. XDG config directory, TOML file
2. Embedded in debug session files
3. No persistence initially (reset on restart)

**Impact:** Affects whether UI needs settings infrastructure now or can defer.

---

### Q007: How are DebugCommands dispatched?

**Priority:** P2  
**Domain:** Integration  
**Status:** Open

**Question:** When UI wants to "step over," does it call a method directly, send a message, or push to a command queue?

**Options:**
1. Direct method calls on `DebugSession`
2. `mpsc` channel, UI sends `DebugCommand`, debug thread processes
3. Command queue that debug thread polls

**Impact:** Affects UI code structure, especially for keyboard shortcuts and buttons.

---

### Q008: Mock data for UI development

**Priority:** P2  
**Domain:** UI  
**Status:** Open

**Question:** How should AI agents test UI components before the real semantic layer exists?

**Possibilities:**
- Create mock `DebugSession` implementation
- Static sample data structs
- Record/replay actual debug sessions

**Current plan:** Define mock constructors for ViewModel types. Revisit when more UI exists.

---

### Q009: Error representation in ViewModel

**Priority:** P2  
**Domain:** Integration  
**Status:** Open

**Question:** How are errors (memory read failed, DWARF parse error, etc.) represented in ViewModel types?

**Options:**
1. `Result<T, E>` wrapper on fallible fields
2. Separate error state on `DebugSession`
3. Inline in types (e.g., `is_optimized_out` field)

**Current approach:** Mix of approaches — `is_optimized_out` for variables, `Option` for things that may be absent. May need revision.

---

### Q010: Borrow visualization UX

**Priority:** P3  
**Domain:** UX  
**Status:** Open

**Question:** How exactly should borrow relationships be visualized? This is novel UI territory.

**Ideas:**
- Arrows/lines connecting borrower to borrowed
- Highlighting spans in source
- Timeline view showing borrow scopes
- Collapsible relationship list

**Note:** This is marked as "Tier 3" — core developer should prototype before AI generates.

---

## Resolved Questions

(Move questions here when resolved, documenting the decision)

---

## Template

```markdown
### QNNN: [Brief title]

**Priority:** P1 | P2 | P3  
**Domain:** Core | UI | Integration | UX  
**Status:** Open | Resolved | Deferred

**Question:** [The question]

**Options/Possibilities:**
1. ...
2. ...

**Impact:** [Why this matters for the work]
```

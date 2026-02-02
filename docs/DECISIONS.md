# Decision Log

This document records significant decisions made during Anteater development. It helps maintain continuity across sessions and contributors (human or AI).

---

## Format

Each decision entry should include:
- **Date:** When decided
- **Context:** What prompted the decision
- **Decision:** What was decided
- **Rationale:** Why this choice over alternatives
- **Status:** Active, Superseded, or Revisit

---

## Decisions

### 001: Use egui for UI framework

**Date:** Project inception  
**Context:** Need a UI framework for the debugger  
**Decision:** Use egui (immediate mode GUI)  
**Rationale:** 
- Rust-native, fits project philosophy
- Immediate mode simplifies state management
- Good baseline performance
- Active development, good ecosystem

**Status:** Active

---

### 002: Linux x86-64 only (initial scope)

**Date:** Project inception  
**Context:** Need to bound initial scope  
**Decision:** Target only Linux x86-64 initially  
**Rationale:**
- ptrace API is Linux-specific anyway
- Narrowing scope allows focus on core value (Rust semantics)
- Can expand platform support later

**Status:** Active

---

### 003: AI assists with UI, human builds core

**Date:** [Current session]  
**Context:** Determining division of labor  
**Decision:**
- Core developer builds: Debug Core, Semantic Layer, novel UI features
- AI agents can build: Standard debugger panels, boilerplate UI

**Rationale:**
- Core systems work builds durable expertise for developer
- Ownership/MIR correlation is novel and requires deep understanding
- Standard debugger UI is well-understood, less interesting to implement manually
- Clear boundary (ViewModel types) keeps integration clean

**Status:** Active

---

### 004: ViewModel as UI contract

**Date:** [Current session]  
**Context:** Need clean boundary between AI-built UI and human-built core  
**Decision:** Define explicit ViewModel types (`ui_types.rs`) that the UI consumes  
**Rationale:**
- UI can be built against stable-ish interface
- Semantic layer internals can evolve independently
- Makes UI testing easier (mock ViewModels)
- Enforces "dumb UI" principle

**Status:** Active

---

### 005: Types marked with stability levels

**Date:** [Current session]  
**Context:** ViewModel types are provisional while core is being built  
**Decision:** Mark each type with STABLE / PROVISIONAL / SPECULATIVE  
**Rationale:**
- AI agents know what's likely to change
- Encourages loose coupling for SPECULATIVE types
- Documents uncertainty explicitly

**Status:** Active

---

## Template for New Decisions

```markdown
### NNN: [Short title]

**Date:** [Date]  
**Context:** [What situation prompted this decision?]  
**Decision:** [What was decided?]  
**Rationale:** [Why this over alternatives?]  
**Status:** Active | Superseded by NNN | Revisit [condition]
```

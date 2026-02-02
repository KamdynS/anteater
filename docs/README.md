# /docs

Documentation for Anteater development, especially for maintaining context across AI agent sessions.

## Contents

| File | Purpose |
|------|---------|
| `ARCHITECTURE.md` | System overview, layer responsibilities, division of labor |
| `UI_DEVELOPMENT_GUIDE.md` | Conventions and specifications for building UI components |
| `VISUAL_LANGUAGE.md` | How to display ownership states consistently |
| `DECISIONS.md` | Log of significant architectural/design decisions |
| `OPEN_QUESTIONS.md` | Uncertainties that need resolution |
| `WORK_LOG.md` | Session-by-session progress tracking |

## For AI Agents

**Start here:**
1. Read `ARCHITECTURE.md` for context
2. Read `UI_DEVELOPMENT_GUIDE.md` for conventions
3. Check `OPEN_QUESTIONS.md` for known uncertainties
4. Log your work in `WORK_LOG.md`

**Key files outside this folder:**
- `ui_types.rs` — the ViewModel types you'll build against
- Project root: `Project_description`, `mir_dwarf_design.md` — original vision docs

## For Human Developer

These docs are designed to give AI agents enough context to work productively on UI components while you focus on the core. Review and correct them as your understanding of the system evolves.

When you make decisions that affect the UI contract:
1. Update relevant types in `ui_types.rs`
2. Log the decision in `DECISIONS.md`
3. Resolve relevant items in `OPEN_QUESTIONS.md`

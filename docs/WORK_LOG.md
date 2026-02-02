# Work Log

This document tracks work done on Anteater, providing continuity across sessions and contributors.

---

## How to Use This Log

After each work session, add an entry documenting:
- What was accomplished
- What's in progress
- What's blocked or needs decisions
- Any questions for the core developer

This helps the next session (whether same contributor or different) pick up efficiently.

---

## Log Entries

### 2026-01-31: Initial UI Implementation

**Contributor:** AI Agent (Claude)

**Accomplished:**
- Created complete workspace structure with 5 crates:
  - `anteater-ui-types`: ViewModel types (contract between UI and engine)
  - `anteater-ui`: UI layer with egui and egui_dock
  - `anteater-engine`: Semantic layer placeholder (for core developer)
  - `anteater-core`: Debug core placeholder (for core developer)
  - `anteater`: Main binary that composes everything
- Set up documentation in `/docs`:
  - `ARCHITECTURE.md`, `UI_DEVELOPMENT_GUIDE.md`, `DECISIONS.md`
  - `WORK_LOG.md`, `VISUAL_LANGUAGE.md`, `OPEN_QUESTIONS.md`
  - `WORKSPACE_STRUCTURE.md` — new doc explaining crate organization
- Implemented docking panel system using egui_dock
- Created mock DebugSession with realistic test data
- Built functioning panels:
  - **Variables Panel**: Shows variables with ownership states, expandable tree
  - **Registers Panel**: Displays CPU registers with change highlighting
  - **Call Stack Panel**: Shows stack frames with source locations
  - **Disassembly Panel**: Shows machine code with current instruction
  - **Breakpoints Panel**: Lists breakpoints with conditions
  - Placeholder panels for Memory and Source (TODO)
- Built reusable widgets:
  - Ownership badge widget (follows visual language spec)
  - Type display widget
- Created fully functional app shell with:
  - Menu bar (File, Debug, View)
  - Status bar showing execution state
  - Docking area with drag-to-rearrange panels
  - Layout reset functionality
- **Application builds and runs successfully!**

**In Progress:**
- N/A (initial setup complete)

**Blocked:**
- None

**Next Steps / TODOs:**
1. Implement Memory panel with hex dump + ASCII view
2. Implement Source panel with syntax highlighting
3. Add keyboard shortcuts (F5=Continue, F10=Step Over, etc.)
4. Implement actual DebugCommand dispatch (currently just UI placeholders)
5. Add panel state persistence (save/load layouts)
6. Improve the default layout (currently all panels are tabs - could create better initial split)
7. Add more mock data scenarios for testing edge cases
8. Consider adding a console/output panel

**Questions for Core Developer:**
1. The TabViewer in app.rs currently recreates panel instances on each frame. Should panels be persistent structs stored in AnteaterApp instead?
2. How should DebugCommands be dispatched? Direct method calls, mpsc channel, or command queue?
3. Do you want layout persistence now or defer it?
4. Any corrections to the ViewModel types based on what you're building?

**Notes:**
- All panels use mock data for now - ready to be hooked up to real DebugSession
- Visual language follows VISUAL_LANGUAGE.md spec (colors, badges, strikethrough)
- Panel system is fully composable - users can drag panels to create custom layouts
- Code is well-commented and follows the conventions in UI_DEVELOPMENT_GUIDE.md

---

## Template for New Entries

```markdown
### [Date/Session ID]: [Brief Description]

**Contributor:** [Human / AI Agent]

**Accomplished:**
- Item 1
- Item 2

**In Progress:**
- Item (state: where it's at)

**Blocked:**
- Item — blocked on [reason]

**Questions for Core Developer:**
1. Question?
```

# Development Workflow

## Overview

This workflow guides the implementation of new features and bug fixes for the Rust Todo application.

## Feature Implementation Flow

```
Research → Design → Implement → Test → Review → Commit
```

### Phase 1: Research

**Step 1.1 - Codebase Research**
- Read existing code in relevant modules
- Check similar patterns in codebase
- Document current behavior and data structures

**Step 1.2 - External Research (Required)**
- Launch **deep-researcher** subagent to search for:
  - Existing Rust libraries for the feature
  - Open-source implementations (GitHub)
  - Best practices from crates.io
- Filter and summarize findings
- Report: "Useful because..." for each finding

**Step 1.3 - Requirement Clarification**
- Verify feature scope with user if unclear
- Confirm edge cases
- Ask: "For this feature, I understand X. Is this correct?"

---

### Phase 2: Design

**Step 2.1 - Framework Design**
- Outline the implementation structure
- Identify files and functions to modify
- Define interfaces between modules

**Step 2.2 - Framework Review**
- Present design to user
- Verify design matches feature requirements
- Adjust based on feedback

**Step 2.3 - Checklist**
- [ ] New data structures defined
- [ ] New functions outlined
- [ ] Existing code to modify identified
- [ ] Edge cases considered

---

### Phase 3: Implement

**Step 3.1 - Scaffold**
- Add new types/functions (stubs if needed)
- Connect modules

**Step 3.2 - Logic**
- Implement core functionality
- Match existing code style (camelCase, comments)

**Step 3.3 - Integration**
- Wire keyboard shortcuts
- Connect UI actions
- Ensure existing features work

**⚠️ Clarification Rule**: If implementation method is unclear, stop and ask user before continuing.

---

### Phase 4: Test

**Step 4.1 - Manual Test**
- Build: `cargo build --release`
- Run: `cargo run --release`
- Test the new feature
- Test related existing features

**Step 4.2 - Edge Cases**
- Empty state
- Edge inputs
- Error recovery

---

### Phase 5: Review

**Step 5.1 - Code Review**
- Launch **code-reviewer** subagent
- Address CRITICAL/HIGH issues
- Fix MEDIUM issues when possible

**Step 5.2 - Quality Check**
- [ ] No new warnings
- [ ] No hardcoded values
- [ ] Comments accurate
- [ ] README updated if needed

---

### Phase 6: Commit

**Commit Message Format**

```
<type>: <description>

<optional body>
```

Types: feat, fix, refactor, docs, test, chore, polish

**Example**:
```
feat: implement sort mode with date/urgency options

- Add SortMode to CurrentMode enum
- Sort tasks by due date (asc/desc)
- Sort tasks by urgency (asc/desc)
- Allow manual reorder after sort
```

---

## Priority Order

Current focus (see AGENTS.md):

| Priority | Feature | Status |
|----------|---------|--------|
| 1 | Sort Mode | Pending |
| 2 | Tag System | Pending |
| 3 | Cleanup | Pending |

## Quality Checklist

Before any commit:

- [ ] Research completed (codebase + external)
- [ ] Design approved
- [ ] Implementation compiles
- [ ] Manual test passed
- [ ] Code review passed
- [ ] No TODO comments left
- [ ] README updated

## Key Files Reference

| Feature | Primary Files |
|---------|-------------|
| Sort Mode | `appstate.rs`, `ui.rs`, `keymap.rs` |
| Tag System | `todolistwidget.rs`, `ui.rs` |
| Workspace | `workspacewidget.rs`, `ui.rs` |

## Notes

- **Data format**: Keep JSON unchanged
- **Testing**: Manual testing via `cargo run --release`
- **Code style**: camelCase, detailed comments matching existing
- **Safety**: Never commit hardcoded secrets
# Development Tasks

## Current Phase: Tag System Implementation

Last Updated: 2026-04-28

---

## Active Tasks

| ID | Task | Status | Priority |
|----|------|--------|----------|
| T006 | Tag System: Research | IN_PROGRESS | MEDIUM |
| T007 | Tag System: Design (data model, UI) | PENDING | MEDIUM |
| T008 | Tag System: Implement (add tags field, UI) | PENDING | MEDIUM |
| T009 | Tag System: Test | PENDING | MEDIUM |
| T010 | Tag System: Review | PENDING | MEDIUM |

---

## Upcoming Tasks

### Tag System

| ID | Task | Status | Priority |
|----|------|--------|----------|
| T006 | Tag System: Research | PENDING | MEDIUM |
| T007 | Tag System: Design (data model, UI) | PENDING | MEDIUM |
| T008 | Tag System: Implement (add tags field, UI) | PENDING | MEDIUM |
| T009 | Tag System: Test | PENDING | MEDIUM |
| T010 | Tag System: Review | PENDING | MEDIUM |

### Cleanup

| ID | Task | Status | Priority |
|----|------|--------|----------|
| T011 | Remove TODO comments | PENDING | LOW |
| T012 | Update README | PENDING | LOW |

---

## Completed Tasks

| ID | Task | Date Completed |
|----|------|---------------|
| T001 | Sort Mode: Research | 2026-04-28 |
| T002 | Sort Mode: Implement | 2026-04-28 |
| T003 | Sort Mode: Test | 2026-04-28 |
| T004 | Sort Mode: Review | 2026-04-28 |
| T005 | Sort Mode: Cleanup | 2026-04-28 |
| B001 | Bug: JSON ws_type missing | 2026-04-28 |
| B002 | Bug: archived_ws missing | 2026-04-28 |
| B003 | Bug: Sort mode deadlock | 2026-04-28 |
| B004 | Bug: Archived display | 2026-04-28 |

---

## Task Details

### T001: Sort Mode - Research

**Description**: Read existing code to understand current implementation

**Checklist**:
- [ ] Read `src/app/appstate.rs` - CurrentMode::Sort
- [ ] Read `src/app/ui.rs` lines 1485-1535 - WidgetAction::Sort
- [ ] Read `src/app/ui/keymap.rs` - sort_hint definitions
- [ ] Read `src/app/ui/todolistwidget.rs` - Task structure (due_date, urgency)

**Exit Criteria**: Understand data structures, can explain sort flow

---

### T002: Sort Mode - Implement

**Description**: Implement 4 sort functions in ui.rs

**Target Code** (ui.rs around line 1524):
```rust
match sort_method.as_str() {
    "da" => { /* sort by due date asc */ }
    "dd" => { /* sort by due date desc */ }
    "ua" => { /* sort by urgency asc */ }
    "ud" => { /* sort by urgency desc */ }
    _ => {}
}
```

**Checklist**:
- [ ] Implement sort_by_due_date_asc()
- [ ] Implement sort_by_due_date_desc()
- [ ] Implement sort_by_urgency_asc()
- [ ] Implement sort_by_urgency_desc()
- [ ] Handle None values for due_date

**Exit Criteria**: Compiles successfully

---

### T003: Sort Mode - Test

**Description**: Build and test the implementation

**Checklist**:
- [ ] Run `cargo build --release` - no errors
- [ ] Test `s` key enters sort mode
- [ ] Test `da` - sort by due date asc
- [ ] Test `dd` - sort by due date desc
- [ ] Test `ua` - sort by urgency asc
- [ ] Test `ud` - sort by urgency desc
- [ ] Test Esc exits sort mode
- [ ] Test j/k moves after sort

---

### T004: Sort Mode - Code Review

**Description**: Submit code for review

**Checklist**:
- [ ] Run code through code-reviewer agent
- [ ] Address CRITICAL issues
- [ ] Address HIGH issues

---

### T005: Sort Mode - Cleanup

**Description**: Final polish

**Checklist**:
- [ ] Remove any TODO comments
- [ ] Update ARCHITECTURE.md if needed
- [ ] Update this TASKS.md (mark complete)

---

## Notes

- Sort Mode: COMPLETED
- Tag System: Pending
- Bug fixes: B001-B004 completed
- Total bugs fixed in this session: 4

---

## Current Status

**Sort Mode**: ✅ COMPLETED
**Tag System**: NEXT
**Bugs Fixed**: 4
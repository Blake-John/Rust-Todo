# System Overview

## 1. System Architecture

```mermaid
graph TD
    subgraph Terminal["Terminal Environment"]
        subgraph Threads["Application Threads"]
            KeyThread["KeyEvent Thread<br/>(tokio runtime)"]
            MsgThread["Message Handler Thread<br/>(tokio runtime)"]
            UIThread["UI Render Thread<br/>(tokio runtime)"]
        end
        
        subgraph UI["User Interface (ratatui)"]
            Workspace["Workspace Widget"]
            TodoList["TodoList Widget"]
            Archived["Archived Workspace Widget"]
            Help["Help Widget"]
            Calendar["Calendar Widget"]
            StatusBar["Status Bar"]
        end
    end
    
    subgraph Core["Core Modules"]
        App["App.rs<br/>(Main Coordinator)"]
        AppState["AppState<br/>(Shared State)"]
        Message["Message Enum<br/>(IPC Channel)"]
        UiMessage["UiMessage Enum<br/>(IPC Channel)"]
    end
    
    subgraph Data["Data Layer"]
        Persistence["persistence.rs<br/>(JSON)"]
        DataFile["~/.todo/data.json"]
    end
    
    KeyThread -->|KeyEvent| App
    App -->|Message| MsgThread
    MsgThread -->|UiMessage| UIThread
    UIThread -->|render| UI
    
    AppState <-->|Arc&ltMutex&gt| Threads
    UI -->|load/save| Persistence
    Persistence -->|read/write| DataFile
```

## 2. Component Hierarchy

```mermaid
graph BT
    subgraph App["App (Main)"]
        main["main.rs<br/>Entry Point"]
        app["app.rs<br/>App Coordinator"]
    end
    
    subgraph State["State Management"]
        appstate["appstate.rs"]
        mode["InputMode"]
        focus["FocusPanel"]
        message["Message"]
    end
    
    subgraph UI["UI Components"]
        ui["ui.rs"]
        workspace["workspacewidget.rs"]
        todolist["todolistwidget.rs"]
        help["helpwidget.rs"]
        calendar["calendarwidget.rs"]
        keymap["keymap.rs"]
        prompt["prompt.rs"]
    end
    
    subgraph Data["Data Layer"]
        data["data.rs"]
        errors["errors.rs"]
    end
    
    main --> app
    app --> appstate
    app --> ui
    
    ui --> workspace
    ui --> todolist
    ui --> help
    ui --> calendar
    ui --> keymap
    ui --> prompt
    
    app --> data
    data --> errors
```

## 3. Thread Architecture

```mermaid
graph LR
    subgraph Input["Input Layer"]
        Key["crossterm<br/>KeyEvent"]
    end
    
    subgraph Channels["IPC Channels"]
        Ch1["Channel&ltMessage&gt"]
        Ch2["Channel&ltUiMessage&gt"]
        Ch3["Channel&ltKeyEvent&gt"]
    end
    
    subgraph State["Shared State"]
        Shared["Arc&ltMutex&lt;AppState&gt;&gt;"]
    end
    
    subgraph Threads["3 Application Threads"]
        T1["KeyEvent Handler"]
        T2["Message Handler"]
        T3["UI Render"]
    end
    
    Key --> T1
    T1 -->|KeyEvent| Ch3
    T1 -->|Message| Ch1
    Ch1 --> T2
    T2 -->|UiMessage| Ch2
    Ch2 --> T3
    T1 --> Shared
    T2 --> Shared
    T3 --> Shared
```

## 4. Module Dependency Graph

```mermaid
graph TD
    main["main.rs"] --> app["app.rs"]
    
    app --> appstate["appstate.rs"]
    app --> data["data.rs"]
    app --> ui["ui.rs"]
    
    appstate --> SearchEvent["SearchEvent"]
    
    data --> Datas["Datas"]
    data --> persistence["persistence.rs"]
    data --> errors["errors.rs"]
    data --> WorkspaceWidget["WorkspaceWidget"]
    data --> TodoWidget["TodoWidget"]
    
    ui -->|imports| ui_modules["ui/*.rs"]
    ui_modules --> ratatui["ratatui"]
    ui_modules --> crossterm["crossterm"]
    ui_modules --> TextArea["TextArea"]
    ui_modules --> CalendarWidget
    ui_modules --> HelpWidget
    ui_modules --> KeymapWidget
    ui_modules --> PromptWidget
    
    ui_modules --> WorkspaceWidget
    ui_modules --> TodoWidget
    ui_modules --> Task
    ui_modules --> Workspace
    ui_modules --> TaskStatus
    
    WorkspaceWidget --> Workspace
    TodoWidget --> TodoList
    TodoWidget --> Task
```

## 5. UI Layout Structure

```mermaid
graph TD
    subgraph Layout["Layout (ratatui)"]
        Root["Root Layout<br/>(Vertical)"]
        
        Top["Top Panel<br/>(Fill)"]
        Bottom["Bottom Panel<br/>(Max 1)"]
        
        TopLeft["Workspace Area<br/>(20%)"]
        TopRight["Todo Area<br/>(80%)"]
        
        WorkspaceTop["Active Workspaces<br/>(80%)"]
        WorkspaceBottom["Archived Workspaces<br/>(20%)"]
        
        LeftBottom["Keymap Hints<br/>(Fill)"]
        RightBottom["Status Bar<br/>(Length N)"]
    end
    
    Root --> Top
    Root --> Bottom
    
    Top --> TopLeft
    Top --> TopRight
    
    TopLeft --> WorkspaceTop
    TopLeft --> WorkspaceBottom
    
    Bottom --> LeftBottom
    Bottom --> RightBottom
```

## 6. Data Model Architecture

```mermaid
erDiagram
    WORKSPACE {
        string id PK
        string name
        string parent_id FK "nullable"
        string workspace_type "Normal|Archived"
    }
    
    TASK {
        string id PK
        string title
        string status "Todo|InProgress|Completed|Deprecated"
        string due_date "nullable"
        int urgency
        string parent_task_id FK "nullable"
        string todolist_id FK
    }
    
    TODOLIST {
        string id PK
        string workspace_id FK
    }
    
    WORKSPACE ||--o{ WORKSPACE : "has children"
    WORKSPACE ||--o| TODOLIST : "has"
    TODOLIST ||--o{ TASK : "contains"
    TASK ||--o{ TASK : "has subtasks"
```

## 7. State Machine

```mermaid
stateDiagram-v2
    [*] --> NormalMode
    
    state NormalMode {
        [*] --> WorkspaceFocused
        [*] --> TodoListFocused
        [*] --> ArchivedFocused
        
        WorkspaceFocused --> TodoListFocused: Tab/1/2/3
        TodoListFocused --> WorkspaceFocused: Tab/h
        ArchivedFocused --> TodoListFocused: Tab
    }
    
    NormalMode --> InsertMode: a/i/r/D
    NormalMode --> SearchMode: f//
    NormalMode --> HelpMode: ?
    
    InsertMode --> NormalMode: Enter/Esc
    SearchMode --> NormalMode: Esc
    HelpMode --> NormalMode: q/Esc
    
    InsertMode --> SortMode: s
    SortMode --> NormalMode: Select/Esc
```

## 8. Component Overview

| Component | Responsibility | Public API |
|-----------|----------------|------------|
| `main.rs` | Entry point | `main()` |
| `app.rs` | Thread coordination | `App::new()`, `App::run()` |
| `appstate.rs` | State management | `AppState`, `Message`, `CurrentMode`, `CurrentFocus` |
| `data.rs` | Persistence | `load_data()`, `save_data()` |
| `ui.rs` | UI orchestration | `Ui::new()`, `Ui::update()` |
| `workspacewidget.rs` | Workspace list | `WorkspaceWidget`, `Workspace` |
| `todolistwidget.rs` | Task list | `TodoWidget`, `TodoList`, `Task` |
| `calendarwidget.rs` | Date picker | `CalendarWidget` |
| `helpwidget.rs` | Help overlay | `HelpWidget` |
| `keymap.rs` | Key hints | `KeymapWidget` |
| `prompt.rs` | Status bar | `PromptWidget` |

## 9. Technology Stack

| Layer | Technology | Version |
|-------|-----------|---------|
| UI Framework | ratatui | 0.29.0 |
| Terminal Input | crossterm | 0.29.0 |
| Async Runtime | tokio | 1.47.1 |
| Serialization | serde_json | 1.0.143 |
| Date/Time | chrono | 0.4.41 |
| Unique IDs | uuid | 1.18.0 |
| Text Input | tui-textarea | 0.7.0 |
| Regex | regex | 1.11.1 |
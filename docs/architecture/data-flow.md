# Data Flow

## 1. Overall Data Flow

```mermaid
flowchart LR
    subgraph Input["User Input"]
        Key["Keyboard Events"]
        Mouse["Mouse Events"]
        Resize["Terminal Resize"]
    end
    
    subgraph Processing["Processing"]
        KeyThread["KeyEvent Handler"]
        MsgThread["Message Handler"]
    end
    
    subgraph State["State Management"]
        AppState["AppState<br/>(Arc&ltMutex&gt)"]
    end
    
    subgraph UI["User Interface"]
        UIThread["UI Render"]
        Widgets["Widgets"]
    end
    
    subgraph Storage["Persistence"]
        JSON["data.json"]
    end
    
    Key --> KeyThread
    KeyThread -->|Message| MsgThread
    MsgThread -->|UiMessage| UIThread
    UIThread -->|render| Widgets
    
    KeyThread <-->|read/write| AppState
    MsgThread <-->|read/write| AppState
    UIThread <-->|read/write| AppState
    
    Widgets -->|save| JSON
    JSON -->|load| Widgets
```

## 2. Key Event Flow

```mermaid
sequenceDiagram
    participant User
    participant Crossterm
    participant KeyHandler
    participant AppState
    participant Channel1
    participant Channel3
    
    User->>Crossterm: KeyPress
    Crossterm->>KeyHandler: KeyEvent
    KeyHandler->>KeyHandler: Read mode
    
    alt Normal/Search Mode
        KeyHandler->>KeyHandler: Map key to Message
        KeyHandler->>Channel1: Send(Message)
        KeyHandler->>Channel3: (drop for Insert mode)
    else Insert/Help Mode
        KeyHandler->>Channel3: Send(KeyEvent)
    end
    
    KeyHandler->>AppState: Read current_focus/mode
```

## 3. Message Flow

```mermaid
sequenceDiagram
    participant KeyThread
    participant MsgThread
    participant AppState
    participant UITx
    participant Channel2
    
    KeyThread->>Channel2: Message::AddItem
    Channel2->>MsgThread: recv()
    
    MsgThread->>MsgThread: Match Message type
    
    alt AddItem
        MsgThread->>AppState: current_mode = Insert
        MsgThread->>UITx: UiMessage::WAction(AddWorkspace)
    end
    
    alt MoveUp/Down
        MsgThread->>UITx: UiMessage::WAction(SelectUp/SelectDown)
    end
    
    alt ChangeFocus
        MsgThread->>AppState: current_focus = new_focus
        MsgThread->>UITx: UiMessage::WAction(FocusXxx)
    end
    
    MsgThread->>AppState: Release lock
```

## 4. UI Update Flow

```mermaid
flowchart TD
    subgraph Trigger["UI Update Trigger"]
        T1["UiMessage::Update"]
        T2["UiMessage::UpdateUi"]
        T3["WidgetAction"]
    end
    
    subgraph Process["UI Processing"]
        P1["Receive UiMessage"]
        P2["Match message type"]
        P3["Execute action"]
        P4["Update state"]
    end
    
    subgraph Render["Rendering"]
        R1["terminal.draw()"]
        R2["calculate layout"]
        R3["render widgets"]
        R4["refresh display"]
    end
    
    T1 --> P1
    T2 --> P1
    T3 --> P1
    
    P1 --> P2
    P2 --> P3
    P3 --> P4
    P4 --> R1
    R1 --> R2
    R2 --> R3
    R3 --> R4
```

## 5. Control Flow by Mode

```mermaid
flowchart TD
    subgraph Entry["KeyEvent Entry"]
        E1["Read key event"]
        E2["Get AppState"]
    end
    
    subgraph Decision["Mode Decision"]
        D1{"current_mode"}
    end
    
    subgraph Normal["Normal Mode"]
        N1["Match key code"]
        N2["Send Message"]
    end
    
    subgraph Insert["Insert Mode"]
        I1["Forward to input_rx"]
    end
    
    subgraph Help["Help Mode"]
        H1["Match navigation keys"]
        H2["Scroll help"]
    end
    
    subgraph Search["Search Mode"]
        S1["Forward to input_rx"]
    end
    
    E1 --> E2
    E2 --> D1
    
    D1 -->|Normal| Normal
    D1 -->|Insert| Insert
    D1 -->|Help| Help
    D1 -->|Search| Search
    D1 -->|Sort| Insert
    
    Normal --> N1 --> N2
    Insert --> I1
    Help --> H1 --> H2
    Search --> S1
```

## 6. Persistence Flow

```mermaid
flowchart LR
    subgraph Save["Save Flow"]
        S1["User presses Ctrl+S"]
        S2["Send Message::SaveData"]
        S3["UI receives UiMessage::SaveData"]
        S4["Clone widgets to Datas"]
        S5["serde_json::to_string_pretty"]
        S6["fs::write to data.json"]
    end
    
    subgraph Load["Load Flow"]
        L1["App::run() starts"]
        L2["Check data.json exists"]
        L3["fs::read_to_string"]
        L4["serde_json::from_str"]
        L5["Initialize widgets"]
    end
    
    subgraph Auto["Auto Save"]
        A1["App exits"]
        A2["Clone widgets to Datas"]
        A3["save_data()"]
    end
    
    S1 --> S2 --> S3 --> S4 --> S5 --> S6
    L1 --> L2 --> L3 --> L4 --> L5
    A1 --> A2 --> A3
```

## 7. Task State Transitions

```mermaid
stateDiagram-v2
    [*] --> Todo
    
    Todo --> InProgress: p
    Todo --> Completed: c
    Todo --> Deprecated: d
    
    InProgress --> Todo: t
    InProgress --> Completed: c
    InProgress --> Deprecated: d
    
    Completed --> Todo: t
    Completed --> InProgress: p
    Completed --> Deprecated: d
    
    Deprecated --> Todo: t
    Deprecated --> InProgress: p
    Deprecated --> Completed: c
```

## 8. Focus Panel Transitions

```mermaid
flowchart LR
    subgraph Trigger["Focus Change Triggers"]
        T1["Tab key"]
        T2["1/2/3 keys"]
        T3["Enter/h keys"]
    end
    
    subgraph Panels["Panel States"]
        P1["Workspace"]
        P2["TodoList"]
        P3["Archived"]
    end
    
    T1 --> P1
    T1 --> P2
    T1 --> P3
    
    P1 <-.-|Tab| P2
    P2 <-.-|Tab| P3
    P3 <-.-|Tab| P1
    
    P2 -.->|h| P1
    P1 -.->|Enter| P2
    
    P1 -.->|1| P1
    P2 -.->|2| P3
    P3 -.->|3| P2
```

## 9. Data Lifecycle

```mermaid
graph TD
    subgraph Init["Initialization"]
        I1["Create App"]
        I2["Create terminal"]
        I3["Load data.json"]
        I4["Initialize UI"]
    end
    
    subgraph Runtime["Runtime"]
        R1["User interactions"]
        R2["State updates"]
        R3["UI renders"]
    end
    
    subgraph Persistence["Persistence"]
        P1["Auto-save on exit"]
        P2["Manual Ctrl+S"]
    end
    
    Init --> I1 --> I2 --> I3 --> I4
    
    I4 --> R1
    R1 --> R2 --> R3
    R3 --> R1
    
    R1 -.->|Ctrl+S| P2
    P2 --> I3
    
    R3 -.->|exit| P1
    P1 --> Init
```
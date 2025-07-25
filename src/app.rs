use std::{
    path::Path, sync::{Arc, Mutex}
};
use tokio::sync::mpsc;

use crossterm::event;

use crate::app::{
    appstate::{AppState, CurrentFocus, CurrentMode, Message},
    data::Datas,
    ui::{InputEvent, UiMessage, WidgetAction, todolistwidget::TaskStatus},
};

pub mod appstate;
pub mod data;
pub mod errors;
pub mod ui;

/// The Basic Structure of the App
///
/// # Fields
///
/// - `appstate` (`Arc<Mutex<AppState>>`) - A structure that holds the state of the app.
///
/// # Examples
///
/// just simply create a new App by
///
/// ```
/// use crate::app::App;
/// let s = App::new();
/// ```
///
/// or
///
/// ```
/// use crate::app::App;
/// let s = App {
///     appstate: Arc::new(Mutex::new(AppState::new())),
/// };
/// ```
#[derive(Debug)]
pub struct App {
    appstate: Arc<Mutex<AppState>>,
}

impl App {
    pub fn new() -> Self {
        Self {
            appstate: Arc::new(Mutex::new(AppState::new())),
        }
    }
    /// The main function of the app
    ///
    /// # Arguments
    ///
    /// - `&self` ([`App`])
    ///
    /// # Returns
    ///
    /// - `Result<(), errors::Errors>` - the result of the run process.
    ///
    /// # Errors
    ///
    /// see [`errors::Errors`] for more details.
    ///
    /// # Examples
    ///
    /// ```
    /// use crate::app::App;
    ///
    /// let app = App::new();
    /// let res = app.run();
    /// ```
    pub fn run(&self) -> Result<(), errors::Errors> {
        let mut terminal = ratatui::init();
        let (tx, rx) = mpsc::channel::<Message>(10);
        let (ui_tx, ui_rx) = mpsc::channel::<UiMessage>(10);
        let (input_tx, input_rx) = mpsc::channel::<InputEvent>(10);

        let apps_in_keyhand = self.appstate.clone();
        let key_handle = std::thread::spawn(move || {
            let rt = tokio::runtime::Builder::new_current_thread()
                .build()
                .unwrap();
            rt.block_on(handle_keyevt(tx, input_tx, apps_in_keyhand));
        });

        let apps_in_msghand = self.appstate.clone();
        let ui_tx_in_msg = ui_tx.clone();
        let _msg_handle = std::thread::spawn(|| {
            let rt = tokio::runtime::Builder::new_current_thread()
                .build()
                .unwrap();
            rt.block_on(handle_msg(rx, ui_tx_in_msg, apps_in_msghand));
        });

        let apps_in_ui = self.appstate.clone();
        let ui_handle = std::thread::spawn(move || -> Result<(), errors::Errors> {
            let mut ui = ui::Ui::new(ui_rx, input_rx);
            let path = Path::new(std::env::home_dir().unwrap_or(std::path::PathBuf::from("/home/blake/")).as_path()).join(".todo/data.json");
            let data = data::load_data(path.as_path())?;
            ui.workspace = data.workspace;
            ui.todolist = data.todolist;

            ui.refresh_current();
            let mut apps = apps_in_ui.lock().unwrap();
            apps.current_focus = if ui.workspace.focused {
                CurrentFocus::Workspace
            } else {
                CurrentFocus::TodoList
            };
            drop(apps);
            let rt = tokio::runtime::Builder::new_current_thread()
                .build()
                .unwrap();

            rt.block_on(ui.handle_uimsg(&mut terminal, apps_in_ui));
            let datas = Datas {
                workspace: ui.workspace,
                todolist: ui.todolist,
            };

            data::save_data(path.as_path(), &datas)
        });

        let rt = tokio::runtime::Builder::new_current_thread()
            .build()
            .unwrap();
        rt.block_on(async move {
            let _ = ui_tx.send(UiMessage::Update).await;
            let _ = ui_tx.send(UiMessage::UpdateUi).await;
        });

        key_handle
            .join()
            .map_err(|_| errors::Errors::AppError)
            .unwrap();
        let result = ui_handle
            .join()
            .map_err(|_| errors::Errors::UiError)
            .unwrap();

        ratatui::restore();
        result
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

/// A function handles the keyboard events runing in a thread
///
/// # Arguments
///
/// - `tx` (`mpsc`) - a mpsc to send [`Message`] to the message handler
/// - `input_tx` (`mpsc`) - a mpsc sender to send [`InputEvent`] to the ui module for input handling
/// - `appstate` (`Arc<Mutex<AppState>>`) - the state of the app
///
/// # Examples
///
/// ```no_run
/// use crate::app::*;
///
/// async {
///   let result = handle_keyevt().await;
/// };
/// ```
async fn handle_keyevt(
    tx: mpsc::Sender<Message>,
    input_tx: mpsc::Sender<InputEvent>,
    appstate: Arc<Mutex<AppState>>,
) {
    loop {
        let evt = event::read().unwrap();
        if let event::Event::Key(key_evt) = evt {
            if let event::KeyEventKind::Press = key_evt.kind {
                let apps = appstate.lock().unwrap();
                match apps.current_mode {
                    CurrentMode::Normal => match key_evt.code {
                        event::KeyCode::Char('q') => {
                            let _ = tx.send(Message::Exit).await;
                            break;
                        }
                        event::KeyCode::Char('a') => {
                            let _ = tx.send(Message::AddItem).await;
                        }
                        event::KeyCode::Char('i') => {
                            let _ = tx.send(Message::AddChild).await;
                        }
                        event::KeyCode::Char('j') => {
                            let _ = tx.send(Message::MoveDown).await;
                        }
                        event::KeyCode::Char('k') => {
                            let _ = tx.send(Message::MoveUp).await;
                        }
                        event::KeyCode::Char('l') => {
                            if let CurrentFocus::Workspace = apps.current_focus {
                                let _ = tx.send(Message::SelectWorkspace).await;
                            }
                        }
                        event::KeyCode::Char('c') => {
                            if let CurrentFocus::TodoList = apps.current_focus {
                                let _ = tx.send(Message::Complete).await;
                            }
                        }
                        event::KeyCode::Char('t') => {
                            if let CurrentFocus::TodoList = apps.current_focus {
                                let _ = tx.send(Message::Todo).await;
                            }
                        }
                        event::KeyCode::Char('p') => {
                            if let CurrentFocus::TodoList = apps.current_focus {
                                let _ = tx.send(Message::InProcess).await;
                            }
                        }
                        event::KeyCode::Char('A') => {
                            if let CurrentFocus::Workspace = apps.current_focus {
                                let _ = tx.send(Message::Archive).await;
                            }
                        }
                        event::KeyCode::Char('h') => {
                            if let CurrentFocus::TodoList = apps.current_focus {
                                let _ =
                                    tx.send(Message::ChangeFocus(CurrentFocus::Workspace)).await;
                            }
                        }
                        event::KeyCode::Char('d') => {
                            if let CurrentFocus::TodoList = apps.current_focus {
                                let _ = tx.send(Message::Deprecated).await;
                            }
                        }
                        event::KeyCode::Char('x') => {
                            let _ = tx.send(Message::DeleteItem).await;
                        }
                        event::KeyCode::Tab => match apps.current_focus {
                            CurrentFocus::TodoList => {
                                let _ =
                                    tx.send(Message::ChangeFocus(CurrentFocus::Workspace)).await;
                            }
                            CurrentFocus::Workspace => {
                                let _ = tx.send(Message::SelectWorkspace).await;
                            }
                        },
                        event::KeyCode::Enter => {
                            if let CurrentFocus::Workspace = apps.current_focus {
                                let _ = tx.send(Message::SelectWorkspace).await;
                            }
                        }
                        _ => {}
                    },
                    CurrentMode::Insert => match key_evt.code {
                        event::KeyCode::Char(c) => {
                            let _ = input_tx.send(InputEvent::InsertChar(c)).await;
                        }
                        event::KeyCode::Backspace => {
                            let _ = input_tx.send(InputEvent::Backspace).await;
                        }
                        event::KeyCode::Esc => {
                            let _ = input_tx.send(InputEvent::Esc).await;
                        }
                        event::KeyCode::Enter => {
                            let _ = input_tx.send(InputEvent::Enter).await;
                        }
                        event::KeyCode::Left => {
                            let _ = input_tx.send(InputEvent::Left).await;
                        }
                        event::KeyCode::Right => {
                            let _ = input_tx.send(InputEvent::Right).await;
                        }
                        _ => {}
                    },
                }
            }
        } else if let event::Event::Resize(_, _) = evt {
            let _ = tx.send(Message::Update).await;
        }
    }
}

/// The function handle the message from keyevent handler
///
/// # Arguments
///
/// - `mut rx` (`mpsc`) - mpsc receiver to receive message from keyevent handler
/// - `ui_tx` (`mpsc`) - mpsc sender to send message to ui
/// - `appstate` (`Arc<Mutex<AppState>>`) - the state of the app
///
/// # Examples
///
/// ```no_run
/// use crate::app::handle_msg;
///
/// async {
///   let result = handle_msg().await;
/// };
/// ```
async fn handle_msg(
    mut rx: mpsc::Receiver<Message>,
    ui_tx: mpsc::Sender<UiMessage>,
    appstate: Arc<Mutex<AppState>>,
) {
    loop {
        match rx.recv().await.unwrap_or(Message::Exit) {
            Message::Exit => {
                let mut apps = appstate.lock().unwrap();
                apps.exit = true;
                break;
            }
            Message::AddItem => {
                let mut apps = appstate.lock().unwrap();
                match apps.current_focus {
                    CurrentFocus::Workspace => {
                        apps.current_mode = CurrentMode::Insert;
                        let _ = ui_tx
                            .send(UiMessage::WAction(WidgetAction::AddWorkspace))
                            .await;
                    }
                    CurrentFocus::TodoList => {
                        apps.current_mode = CurrentMode::Insert;
                        let _ = ui_tx.send(UiMessage::WAction(WidgetAction::AddTask)).await;
                    }
                }
            }
            Message::AddChild => {
                let mut apps = appstate.lock().unwrap();
                match apps.current_focus {
                    CurrentFocus::Workspace => {
                        apps.current_mode = CurrentMode::Insert;
                        let _ = ui_tx
                            .send(UiMessage::WAction(WidgetAction::AddWorkspaceChild))
                            .await;
                    }
                    CurrentFocus::TodoList => {
                        apps.current_mode = CurrentMode::Insert;
                        let _ = ui_tx
                            .send(UiMessage::WAction(WidgetAction::AddTaskChild))
                            .await;
                    }
                }
            }
            Message::ChangeMode(mode) => {
                let mut apps = appstate.lock().unwrap();
                apps.current_mode = mode;
            }
            Message::ChangeFocus(focus) => {
                let mut apps = appstate.lock().unwrap();
                apps.current_focus = focus.clone();
                let _ = ui_tx
                    .send(match focus {
                        CurrentFocus::Workspace => UiMessage::WAction(WidgetAction::FocusWorkspace),
                        CurrentFocus::TodoList => UiMessage::WAction(WidgetAction::FocusTodolist),
                    })
                    .await;
            }
            Message::SelectWorkspace => {
                let _ = ui_tx
                    .send(UiMessage::WAction(WidgetAction::EnterWorkspace))
                    .await;
            }
            Message::MoveUp => {
                let _ = ui_tx.send(UiMessage::WAction(WidgetAction::SelectUp)).await;
            }
            Message::MoveDown => {
                let _ = ui_tx
                    .send(UiMessage::WAction(WidgetAction::SelectDown))
                    .await;
            }
            Message::Update => {
                let _ = ui_tx.send(UiMessage::UpdateUi).await;
            }
            Message::DeleteItem => {
                let mut apps = appstate.lock().unwrap();
                apps.current_mode = CurrentMode::Insert;
                match apps.current_focus {
                    CurrentFocus::Workspace => {
                        let _ = ui_tx
                            .send(UiMessage::WAction(WidgetAction::DeleteWorkspace))
                            .await;
                    }
                    CurrentFocus::TodoList => {
                        let _ = ui_tx
                            .send(UiMessage::WAction(WidgetAction::DeleteTask))
                            .await;
                    }
                }
            }
            Message::Archive => {
                let _ = ui_tx
                    .send(UiMessage::WAction(WidgetAction::ArchiveWS))
                    .await;
            }
            Message::Complete => {
                let _ = ui_tx
                    .send(UiMessage::WAction(WidgetAction::MarkTaskStatus(
                        TaskStatus::Finished,
                    )))
                    .await;
            }
            Message::InProcess => {
                let _ = ui_tx
                    .send(UiMessage::WAction(WidgetAction::MarkTaskStatus(
                        TaskStatus::InProcess,
                    )))
                    .await;
            }
            Message::Todo => {
                let _ = ui_tx
                    .send(UiMessage::WAction(WidgetAction::MarkTaskStatus(
                        TaskStatus::Todo,
                    )))
                    .await;
            }
            Message::Deprecated => {
                let _ = ui_tx
                    .send(UiMessage::WAction(WidgetAction::MarkTaskStatus(
                        TaskStatus::Deprecated,
                    )))
                    .await;
            }
        }
    }
}

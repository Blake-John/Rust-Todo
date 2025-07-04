use serde_json::ser;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

use crossterm::event;

use crate::app::{
    appstate::{AppState, CurrentFocus, CurrentMode, Message},
    ui::{UiMessage, WidgetAction},
};

mod appstate;
mod errors;
mod ui;

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
    pub fn run(&self) -> Result<(), errors::Errors> {
        let mut terminal = ratatui::init();
        let (tx, rx) = mpsc::channel::<Message>(10);
        let (ui_tx, ui_rx) = mpsc::channel::<UiMessage>(10);

        let apps_in_keyhand = self.appstate.clone();
        let key_handle = std::thread::spawn(move || {
            let rt = tokio::runtime::Builder::new_current_thread()
                .build()
                .unwrap();
            rt.block_on(handle_keyevt(tx, apps_in_keyhand));
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
        let _ui_handle = std::thread::spawn(move || {
            let mut ui = ui::Ui::new(ui_rx);
            let rt = tokio::runtime::Builder::new_current_thread()
                .build()
                .unwrap();

            rt.block_on(ui.handle_uimsg(&mut terminal, apps_in_ui));
            // let path = "data.json";
            // let data = serde_json::to_string_pretty(&ui.workspace).unwrap();
            // std::fs::write(path, data);
        });

        let rt = tokio::runtime::Builder::new_current_thread()
            .build()
            .unwrap();
        let _ = rt.block_on(async move {
            let _ = ui_tx.send(UiMessage::Update).await;
            let _ = ui_tx.send(UiMessage::UpdateUi).await;
        });

        let result = key_handle
            .join()
            .map_err(|_| errors::Errors::AppError)
            .unwrap();
        let result = _ui_handle
            .join()
            .map_err(|_| errors::Errors::UiError)
            .unwrap();

        ratatui::restore();
        Ok(result)
    }
}

async fn handle_keyevt(tx: mpsc::Sender<Message>, appstate: Arc<Mutex<AppState>>) {
    loop {
        if let event::Event::Key(key_evt) = event::read().unwrap() {
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
                        _ => {}
                    },
                    CurrentMode::Insert => {
                        // match key_evt.code {
                        //     event::KeyCode::Esc => {
                        //         let _ = tx.send(Message::ChangeMode(CurrentMode::Normal)).await;
                        //     }
                        //     _ => {}
                        // }
                    }
                }
            }
        }
    }
}

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
                        let _ = ui_tx
                            .send(UiMessage::WAction(WidgetAction::AddWorkspace))
                            .await;
                        apps.current_mode = CurrentMode::Insert;
                    }
                    CurrentFocus::TodoList => {
                        let _ = ui_tx
                            .send(UiMessage::WAction(WidgetAction::AddTodoList))
                            .await;
                        apps.current_mode = CurrentMode::Insert;
                    }
                }
            }
            Message::AddChild => {
                let mut apps = appstate.lock().unwrap();
                match apps.current_focus {
                    CurrentFocus::Workspace => {
                        let _ = ui_tx
                            .send(UiMessage::WAction(WidgetAction::AddWorkspaceChild))
                            .await;
                        apps.current_mode = CurrentMode::Insert;
                    }
                    CurrentFocus::TodoList => {}
                }
            }
            Message::ChangeMode(mode) => {
                let mut apps = appstate.lock().unwrap();
                apps.current_mode = mode;
            }
            Message::MoveUp => {
                let _ = ui_tx.send(UiMessage::WAction(WidgetAction::SelectUp)).await;
            }
            Message::MoveDown => {
                let _ = ui_tx
                    .send(UiMessage::WAction(WidgetAction::SelectDown))
                    .await;
            }
            _ => {}
        }
    }
}

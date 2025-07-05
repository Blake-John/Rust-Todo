use std::{fs, path::Path};

use serde::{Deserialize, Serialize};

use crate::app::{
    errors,
    ui::{todolistwidget::TodoWidget, workspacewidget::WorkspaceWidget},
};

#[derive(Debug, Serialize, Deserialize)]
pub struct Datas {
    pub workspace: WorkspaceWidget,
    pub todolist: TodoWidget,
}

pub fn save_data(path: &Path, datas: &Datas) -> Result<(), errors::Errors> {
    let res = serde_json::to_string_pretty(datas).unwrap();
    let write_res = fs::write(path, res).map_err(|_| errors::Errors::WriteError);
    write_res
}

pub fn load_data(path: &Path) -> Result<Datas, errors::Errors> {
    if path.exists() {
        let content = fs::read_to_string(path).map_err(|_| errors::Errors::LoadError)?;
        let data = serde_json::from_str(&content).map_err(|_| errors::Errors::LoadError);
        data
    } else {
        Ok(Datas {
            workspace: WorkspaceWidget::new(),
            todolist: TodoWidget::new(),
        })
    }
}

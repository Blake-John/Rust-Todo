use std::{fs, path::Path};

use serde::{Deserialize, Serialize};

use crate::app::{
    errors,
    ui::{todolistwidget::TodoWidget, workspacewidget::WorkspaceWidget},
};

/// Struct to store the data of the application only when loading and saving datas
/// 
/// # Fields
/// 
/// - `workspace` ([`WorkspaceWidget`])
/// - `todolist` ([`TodoWidget`])
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Datas {
    pub workspace: WorkspaceWidget,
    pub todolist: TodoWidget,
}

/// save the application data to spesific file
/// 
/// # Arguments
/// 
/// - `path` (`&Path`) - path of data file
/// - `datas` (`&Datas`) - datas to saves
/// 
/// # Returns
/// 
/// - `Result<(), errors::Errors>` - () or error while saving data
/// 
/// # Errors
/// 
/// more detials see [`errors::Errors`]
pub fn save_data(path: &Path, datas: &Datas) -> Result<(), errors::Errors> {
    let res = serde_json::to_string_pretty(datas).unwrap();
    let write_res = fs::write(path, res).map_err(|_| errors::Errors::WriteError);
    write_res
}

/// load the data from the specific file
/// 
/// # Arguments
/// 
/// - `path` (`&Path`) - path of the data file
/// 
/// # Returns
/// 
/// - `Result<Datas, errors::Errors>` - () or error while loading data
/// 
/// # Errors
/// 
/// more details see [`errors::Errors`]
pub fn load_data(path: &Path) -> Result<Datas, errors::Errors> {
    if path.exists() {
        let content = fs::read_to_string(path).map_err(|_| errors::Errors::LoadError)?;
        // let data = serde_json::from_str(&content).map_err(|_| errors::Errors::LoadError);
        let data = serde_json::from_str(&content).unwrap();
        Ok(data)
    } else {
        Ok(Datas::default())
    }
}

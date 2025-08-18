//! Data persistence module
//!
//! This module handles loading and saving application data to/from JSON files.
//! It provides serialization and deserialization functionality for the main
//! application data structures including workspaces, todo lists, and archived items.

use std::{fs, path::Path};

use serde::{Deserialize, Serialize};

use crate::app::{
    errors,
    ui::{
        todolistwidget::TodoWidget,
        workspacewidget::{self, WorkspaceType, WorkspaceWidget},
    },
};

/// Data structure for application persistence
///
/// This structure represents the complete serialized state of the application,
/// containing all workspaces, todo lists, and archived items. It is used for
/// saving and loading application data to/from JSON files.
///
/// The Datas struct serves as the root serialization container that holds all
/// application state that needs to be persisted between sessions.
///
/// # Fields
///
/// - `workspace` ([`WorkspaceWidget`]) - The main workspace data containing active workspaces
/// - `todolist` ([`TodoWidget`]) - The todo list data containing all tasks organized by workspace
/// - `archived_ws` ([`WorkspaceWidget`]) - The archived workspace data containing archived workspaces
///
/// # Examples
///
/// ```
/// use crate::app::data::Datas;
/// use crate::app::ui::workspacewidget::{WorkspaceWidget, WorkspaceType};
/// use crate::app::ui::todolistwidget::TodoWidget;
///
/// let data = Datas {
///     workspace: WorkspaceWidget::new(WorkspaceType::Normal),
///     todolist: TodoWidget::new(),
///     archived_ws: WorkspaceWidget::new(WorkspaceType::Archived),
/// };
/// ```
#[derive(Debug, Serialize, Deserialize)]
pub struct Datas {
    /// The main workspace data containing active workspaces
    pub workspace: WorkspaceWidget,
    /// The todo list data containing all tasks organized by workspace
    pub todolist: TodoWidget,
    /// The archived workspace data containing archived workspaces
    pub archived_ws: WorkspaceWidget,
}

impl Default for Datas {
    fn default() -> Self {
        Self {
            workspace: workspacewidget::WorkspaceWidget::new(WorkspaceType::Normal),
            todolist: TodoWidget::new(),
            archived_ws: workspacewidget::WorkspaceWidget::new(WorkspaceType::Archived),
        }
    }
}

/// Save the application data to a specific file
///
/// Serializes the application data to JSON format and writes it to the specified file path.
/// This function is used to persist the current state of the application including all
/// workspaces, tasks, and archived items.
///
/// # Arguments
///
/// - `path` (`&Path`) - The file path where the data should be saved
/// - `datas` (`&Datas`) - The data structure containing all application data to be saved
///
/// # Returns
///
/// - `Result<(), errors::Errors>` - Ok(()) on successful save, or an error if the operation fails
///
/// # Errors
///
/// Returns [`errors::Errors::WriteError`] if there are issues writing to the file system
///
/// # Examples
///
/// ```
/// use std::path::Path;
/// use crate::app::data::{Datas, save_data};
///
/// // Create sample data
/// let datas = Datas::default();
///
/// // Save to a file (this would typically use a proper path)
/// // let path = Path::new("/path/to/data.json");
/// // let result = save_data(path, &datas);
/// ```
pub fn save_data(path: &Path, datas: &Datas) -> Result<(), errors::Errors> {
    let res = serde_json::to_string_pretty(datas).unwrap();

    fs::write(path, res).map_err(|_| errors::Errors::WriteError)
}

/// Load the application data from a specific file
///
/// Reads and deserializes application data from a JSON file. If the file doesn't exist,
/// this function will create the necessary directory structure and return default data.
///
/// # Arguments
///
/// - `path` (`&Path`) - The file path from which to load the data
///
/// # Returns
///
/// - `Result<Datas, errors::Errors>` - The loaded data structure on success, or an error if the operation fails
///
/// # Errors
///
/// Returns [`errors::Errors::LoadError`] if there are issues reading from the file system or parsing the JSON
///
/// # Examples
///
/// ```
/// use std::path::Path;
/// use crate::app::data::{Datas, load_data};
///
/// // Load from a file (this would typically use a proper path)
/// // let path = Path::new("/path/to/data.json");
/// // let result = load_data(path);
/// ```
pub fn load_data(path: &Path) -> Result<Datas, errors::Errors> {
    if path.exists() {
        let content = fs::read_to_string(path).map_err(|_| errors::Errors::LoadError)?;
        // let data = serde_json::from_str(&content).map_err(|_| errors::Errors::LoadError);
        let data = serde_json::from_str(&content).unwrap();
        Ok(data)
    } else {
        let _ = fs::create_dir_all(
            path.parent().unwrap_or(
                std::env::home_dir()
                    .unwrap_or(std::path::PathBuf::from("/home/blake"))
                    .as_path(),
            ),
        );
        Ok(Datas::default())
    }
}

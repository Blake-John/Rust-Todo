//! A Tui Todo Application Based on Ratatui
//!
//! This is the entry point of the Rust Todo application, a terminal-based task management tool
//! built with the Ratatui library. The application provides a text-based user interface for
//! managing tasks and workspaces with features such as:
//!
//! - Task creation, modification, and deletion
//! - Workspace management (create, archive, recover)
//! - Task status tracking (todo, in-progress, completed, deprecated)
//! - Due date management
//! - Nested task and workspace support
//!
//! ## Usage
//!
//! To run the application, simply execute the binary:
//!
//! ```bash
//! cargo run
//! ```
//!
//! The application will start in your terminal and provide keyboard-driven controls for
//! managing your tasks and workspaces.
//!
//! ## Entry Point
//!
//! The `main` function serves as the application's entry point, creating an instance of
//! [`app::App`] and running it. It handles any errors that may occur during execution
//! and displays them to the user.

pub mod app;

/// The main entry point for the Rust Todo application.
///
/// This function initializes and runs the application:
/// 1. Creates a new instance of the [`app::App`] struct
/// 2. Calls the [`app::App::run`] method to start the application
/// 3. Handles any errors that occur during execution
/// 4. Displays a termination message when the application exits
///
/// # Examples
///
/// ```bash
/// cargo run
/// ```
///
/// # Errors
///
/// If the application encounters an error during execution, it will be printed to stdout
/// in the format: "The app end with error: {:?}", err
pub fn main() {
    let app = app::App::new();
    let appresult = app.run();
    if let Err(err) = appresult {
        println!("The app end with error: {:?}", err);
    }

    println!("The Application is End !");
}

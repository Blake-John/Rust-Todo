//! Error handling module
//!
//! This module defines the error types used throughout the application.
//! It provides a centralized error enum for consistent error handling
//! across all components of the application.

/// The Error type of the application
///
/// This enum represents all possible error conditions that can occur
/// in the application. Each variant corresponds to a specific type
/// of error that may happen during application execution.
///
/// # Variants
///
/// - `AppError` - General runtime error in the application
/// - `UiError` - Error originating from the UI module
/// - `WriteError` - Error occurred while saving data to file
/// - `LoadError` - Error occurred while loading data from file
///
/// # Examples
///
/// ```
/// use crate::app::errors::Errors;
///
/// // Example of returning different error types
/// fn save_operation() -> Result<(), Errors> {
///     // Some operation that might fail
///     Err(Errors::WriteError)
/// }
///
/// fn load_operation() -> Result<(), Errors> {
///     // Some operation that might fail
///     Err(Errors::LoadError)
/// }
/// ```
#[derive(Debug)]
pub enum Errors {
    /// General application runtime error
    AppError,
    /// Error from the UI module
    UiError,
    /// Error while saving data to file
    WriteError,
    /// Error while loading data from file
    LoadError,
}

/// The Error type of the application
/// 
/// # Variants
/// 
/// - `AppError` - runtime error
/// - `UiError` - error from ui module
/// - `WriteError` - error while save the data
/// - `LoadError` - error while load the data
#[derive(Debug)]
pub enum Errors {
    AppError,
    UiError,
    WriteError,
    LoadError,
}

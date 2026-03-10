use crate::models::log_entry::ParseResult;
use crate::parser;

/// Open and parse a log file, auto-detecting its format.
#[tauri::command]
pub fn open_log_file(path: String) -> Result<ParseResult, String> {
    parser::parse_file(&path)
}

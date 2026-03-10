use std::path::PathBuf;

use tauri::State;

use crate::models::log_entry::ParseResult;
use crate::parser;
use crate::state::app_state::{AppState, OpenFile};

/// Open and parse a log file, auto-detecting its format.
/// Stores the detected format and date order in AppState for tail reading.
#[tauri::command]
pub fn open_log_file(path: String, state: State<'_, AppState>) -> Result<ParseResult, String> {
    let (result, date_order) = parser::parse_file(&path)?;

    // Store in AppState so the tail reader can use date_order later
    let mut open_files = state.open_files.lock().map_err(|e| e.to_string())?;
    open_files.insert(
        PathBuf::from(&path),
        OpenFile {
            path: PathBuf::from(&path),
            entries: vec![], // entries live in the frontend
            format: result.format_detected,
            byte_offset: result.byte_offset,
            date_order,
        },
    );

    Ok(result)
}

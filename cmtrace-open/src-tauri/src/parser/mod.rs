pub mod ccm;
pub mod detect;
pub mod plain;
pub mod severity;
pub mod simple;
pub mod timestamped;

use crate::models::log_entry::{LogFormat, ParseResult};
use crate::parser::timestamped::DateOrder;
use std::path::Path;

/// Parse a log file, auto-detecting its format.
/// Returns the parse result and the detected date order (relevant for Timestamped format).
pub fn parse_file(path: &str) -> Result<(ParseResult, DateOrder), String> {
    let path_obj = Path::new(path);
    let content = read_file_content(path)?;
    let file_size = std::fs::metadata(path).map(|m| m.len()).unwrap_or(0);

    let detected = detect::detect_format(&content);
    let lines: Vec<&str> = content.lines().collect();
    let total_lines = lines.len() as u32;

    let (entries, parse_errors) = match detected.format {
        LogFormat::Ccm => ccm::parse_lines(&lines, path),
        LogFormat::Simple => simple::parse_lines(&lines, path),
        LogFormat::Plain => plain::parse_lines(&lines, path),
        LogFormat::Timestamped => timestamped::parse_lines(&lines, path, detected.date_order),
    };

    let result = ParseResult {
        entries,
        format_detected: detected.format,
        total_lines,
        parse_errors,
        file_path: path_obj.to_string_lossy().to_string(),
        file_size,
        // After initial parse, the byte offset is the file size
        byte_offset: file_size,
    };

    Ok((result, detected.date_order))
}

/// Read file content, handling BOM and encoding fallback.
fn read_file_content(path: &str) -> Result<String, String> {
    let bytes = std::fs::read(path).map_err(|e| format!("Failed to read file {}: {}", path, e))?;

    // Try UTF-8 first (strip BOM if present)
    let bytes_no_bom = if bytes.starts_with(&[0xEF, 0xBB, 0xBF]) {
        &bytes[3..]
    } else {
        &bytes
    };

    match std::str::from_utf8(bytes_no_bom) {
        Ok(s) => Ok(s.to_string()),
        Err(_) => {
            // Fallback to Windows-1252 (common for SCCM logs)
            let (cow, _, had_errors) = encoding_rs::WINDOWS_1252.decode(bytes_no_bom);
            if had_errors {
                log::warn!("Encoding errors while reading {}", path);
            }
            Ok(cow.into_owned())
        }
    }
}

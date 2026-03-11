use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelectionSnapshot {
    pub parser: String,
    pub implementation: String,
    pub provenance: String,
    pub parse_quality: String,
    pub record_framing: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntrySnapshot {
    pub id: u64,
    pub line_number: u32,
    pub message: String,
    pub component: Option<String>,
    pub timestamp_display: Option<String>,
    pub severity: String,
    pub format: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedFixture {
    pub selection: SelectionSnapshot,
    pub compatibility_format: String,
    pub total_lines: u32,
    pub parse_errors: u32,
    pub file_size: u64,
    pub byte_offset: u64,
    pub entries: Vec<EntrySnapshot>,
}

pub fn fixture_path(relative: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("corpus")
        .join(relative)
}

pub fn detect_fixture(relative: &str) -> SelectionSnapshot {
    let path = fixture_path(relative);
    let content = fs::read_to_string(&path).expect("fixture should be readable as UTF-8");
    let selection = app_lib::parser::detect::detect_parser(&path.to_string_lossy(), &content);

    selection_snapshot(&selection)
}

pub fn parse_fixture(relative: &str) -> ParsedFixture {
    let path = fixture_path(relative);
    let path_str = path.to_string_lossy().to_string();
    let file_size = fs::metadata(&path)
        .expect("fixture metadata should be readable")
        .len();
    let (result, selection) =
        app_lib::parser::parse_file(&path_str).expect("fixture should parse successfully");

    ParsedFixture {
        selection: selection_snapshot(&selection),
        compatibility_format: format!("{:?}", result.format_detected),
        total_lines: result.total_lines,
        parse_errors: result.parse_errors,
        file_size,
        byte_offset: result.byte_offset,
        entries: result
            .entries
            .into_iter()
            .map(|entry| EntrySnapshot {
                id: entry.id,
                line_number: entry.line_number,
                message: entry.message,
                component: entry.component,
                timestamp_display: entry.timestamp_display,
                severity: format!("{:?}", entry.severity),
                format: format!("{:?}", entry.format),
            })
            .collect(),
    }
}

fn selection_snapshot(selection: &app_lib::parser::ResolvedParser) -> SelectionSnapshot {
    SelectionSnapshot {
        parser: format!("{:?}", selection.parser),
        implementation: format!("{:?}", selection.implementation),
        provenance: format!("{:?}", selection.provenance),
        parse_quality: format!("{:?}", selection.parse_quality),
        record_framing: format!("{:?}", selection.record_framing),
    }
}
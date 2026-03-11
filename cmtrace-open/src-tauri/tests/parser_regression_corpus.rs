mod common;

use common::{detect_fixture, parse_fixture, ParsedFixture, SelectionSnapshot};

fn assert_selection(
    selection: &SelectionSnapshot,
    parser: &str,
    implementation: &str,
    provenance: &str,
    parse_quality: &str,
    record_framing: &str,
) {
    assert_eq!(selection.parser, parser);
    assert_eq!(selection.implementation, implementation);
    assert_eq!(selection.provenance, provenance);
    assert_eq!(selection.parse_quality, parse_quality);
    assert_eq!(selection.record_framing, record_framing);
}

fn assert_parsed_selection(
    parsed: &ParsedFixture,
    parser: &str,
    implementation: &str,
    provenance: &str,
    parse_quality: &str,
    record_framing: &str,
    compatibility_format: &str,
) {
    assert_selection(
        &parsed.selection,
        parser,
        implementation,
        provenance,
        parse_quality,
        record_framing,
    );
    assert_eq!(parsed.compatibility_format, compatibility_format);
    assert_eq!(parsed.byte_offset, parsed.file_size);
}

#[test]
fn panther_clean_fixture_detects_and_parses_multiline_records() {
    let detected = detect_fixture("panther/clean/setupact.log");
    assert_selection(
        &detected,
        "Panther",
        "GenericTimestamped",
        "Dedicated",
        "SemiStructured",
        "LogicalRecord",
    );

    let parsed = parse_fixture("panther/clean/setupact.log");
    assert_parsed_selection(
        &parsed,
        "Panther",
        "GenericTimestamped",
        "Dedicated",
        "SemiStructured",
        "LogicalRecord",
        "Timestamped",
    );
    assert_eq!(parsed.total_lines, 4);
    assert_eq!(parsed.parse_errors, 0);
    assert_eq!(parsed.entries.len(), 2);
    assert_eq!(parsed.entries[0].line_number, 1);
    assert_eq!(parsed.entries[0].component.as_deref(), Some("MIG"));
    assert_eq!(
        parsed.entries[0].message,
        "[0x080489] Gather started\nAdditional migration detail\n    indented continuation"
    );
    assert_eq!(parsed.entries[1].severity, "Warning");
    assert_eq!(parsed.entries[1].message, "Retry required");
}

#[test]
fn panther_mixed_fixture_preserves_fallback_segments() {
    let detected = detect_fixture("panther/mixed/setuperr.log");
    assert_selection(
        &detected,
        "Panther",
        "GenericTimestamped",
        "Dedicated",
        "SemiStructured",
        "LogicalRecord",
    );

    let parsed = parse_fixture("panther/mixed/setuperr.log");
    assert_parsed_selection(
        &parsed,
        "Panther",
        "GenericTimestamped",
        "Dedicated",
        "SemiStructured",
        "LogicalRecord",
        "Timestamped",
    );
    assert_eq!(parsed.total_lines, 5);
    assert_eq!(parsed.parse_errors, 2);
    assert_eq!(parsed.entries.len(), 4);
    assert_eq!(parsed.entries[0].message, "orphan preamble");
    assert_eq!(parsed.entries[1].message, "Setup started\ncontinuation detail");
    assert_eq!(
        parsed.entries[2].message,
        "2024-01-15 08:00:01, UnexpectedLevel SP malformed header"
    );
    assert_eq!(parsed.entries[3].component.as_deref(), Some("SP"));
    assert_eq!(parsed.entries[3].severity, "Error");
}

#[test]
fn cbs_clean_fixture_detects_and_parses_multiline_records() {
    let detected = detect_fixture("cbs/clean/CBS.log");
    assert_selection(
        &detected,
        "Cbs",
        "GenericTimestamped",
        "Dedicated",
        "SemiStructured",
        "LogicalRecord",
    );

    let parsed = parse_fixture("cbs/clean/CBS.log");
    assert_parsed_selection(
        &parsed,
        "Cbs",
        "GenericTimestamped",
        "Dedicated",
        "SemiStructured",
        "LogicalRecord",
        "Timestamped",
    );
    assert_eq!(parsed.total_lines, 4);
    assert_eq!(parsed.parse_errors, 0);
    assert_eq!(parsed.entries.len(), 2);
    assert_eq!(parsed.entries[0].component.as_deref(), Some("CBS"));
    assert_eq!(
        parsed.entries[0].message,
        "Exec: Processing package\nContinuation detail\n    indented continuation"
    );
    assert_eq!(parsed.entries[1].component.as_deref(), Some("CSI"));
    assert_eq!(parsed.entries[1].severity, "Error");
}

#[test]
fn cbs_mixed_fixture_preserves_fallback_segments() {
    let detected = detect_fixture("cbs/mixed/CBS.log");
    assert_selection(
        &detected,
        "Cbs",
        "GenericTimestamped",
        "Dedicated",
        "SemiStructured",
        "LogicalRecord",
    );

    let parsed = parse_fixture("cbs/mixed/CBS.log");
    assert_parsed_selection(
        &parsed,
        "Cbs",
        "GenericTimestamped",
        "Dedicated",
        "SemiStructured",
        "LogicalRecord",
        "Timestamped",
    );
    assert_eq!(parsed.total_lines, 5);
    assert_eq!(parsed.parse_errors, 2);
    assert_eq!(parsed.entries.len(), 4);
    assert_eq!(parsed.entries[0].message, "orphan preamble");
    assert_eq!(parsed.entries[1].message, "Exec: Processing package\nContinuation detail");
    assert_eq!(
        parsed.entries[2].message,
        "2024-01-15 08:00:01, UnexpectedLevel       CBS    malformed header"
    );
    assert_eq!(parsed.entries[3].component.as_deref(), Some("CSI"));
    assert_eq!(parsed.entries[3].severity, "Warning");
}

#[test]
fn dism_clean_fixture_detects_and_parses_multiline_records() {
    let detected = detect_fixture("dism/clean/dism.log");
    assert_selection(
        &detected,
        "Dism",
        "GenericTimestamped",
        "Dedicated",
        "SemiStructured",
        "LogicalRecord",
    );

    let parsed = parse_fixture("dism/clean/dism.log");
    assert_parsed_selection(
        &parsed,
        "Dism",
        "GenericTimestamped",
        "Dedicated",
        "SemiStructured",
        "LogicalRecord",
        "Timestamped",
    );
    assert_eq!(parsed.total_lines, 3);
    assert_eq!(parsed.parse_errors, 0);
    assert_eq!(parsed.entries.len(), 2);
    assert_eq!(parsed.entries[0].component.as_deref(), Some("DISM"));
    assert_eq!(
        parsed.entries[0].message,
        "DISM Provider Store: PID=100 TID=200 loaded provider\nContinuation detail"
    );
    assert_eq!(parsed.entries[1].severity, "Warning");
}

#[test]
fn dism_mixed_fixture_preserves_fallback_segments() {
    let detected = detect_fixture("dism/mixed/dism.log");
    assert_selection(
        &detected,
        "Dism",
        "GenericTimestamped",
        "Dedicated",
        "SemiStructured",
        "LogicalRecord",
    );

    let parsed = parse_fixture("dism/mixed/dism.log");
    assert_parsed_selection(
        &parsed,
        "Dism",
        "GenericTimestamped",
        "Dedicated",
        "SemiStructured",
        "LogicalRecord",
        "Timestamped",
    );
    assert_eq!(parsed.total_lines, 5);
    assert_eq!(parsed.parse_errors, 2);
    assert_eq!(parsed.entries.len(), 4);
    assert_eq!(parsed.entries[0].message, "orphan preamble");
    assert_eq!(
        parsed.entries[1].message,
        "DISM Package Manager: Processing package\nContinuation detail"
    );
    assert_eq!(
        parsed.entries[2].message,
        "2024-01-15 08:00:01, UnexpectedLevel       DISM   malformed header"
    );
    assert_eq!(parsed.entries[3].component.as_deref(), Some("DISM"));
    assert_eq!(parsed.entries[3].severity, "Error");
}

#[test]
fn reporting_events_clean_fixture_detects_and_parses_rows() {
    let detected = detect_fixture("reporting_events/clean/ReportingEvents.log");
    assert_selection(
        &detected,
        "ReportingEvents",
        "ReportingEvents",
        "Dedicated",
        "Structured",
        "PhysicalLine",
    );

    let parsed = parse_fixture("reporting_events/clean/ReportingEvents.log");
    assert_parsed_selection(
        &parsed,
        "ReportingEvents",
        "ReportingEvents",
        "Dedicated",
        "Structured",
        "PhysicalLine",
        "Timestamped",
    );
    assert_eq!(parsed.total_lines, 2);
    assert_eq!(parsed.parse_errors, 0);
    assert_eq!(parsed.entries.len(), 2);
    assert_eq!(parsed.entries[0].component.as_deref(), Some("Windows Update Agent"));
    assert_eq!(
        parsed.entries[0].timestamp_display.as_deref(),
        Some("2024-01-15 08:00:00.123")
    );
    assert!(parsed.entries[0].message.contains("Success | Installation"));
    assert_eq!(parsed.entries[1].severity, "Error");
    assert!(parsed.entries[1].message.contains("HRESULT 0x80240022"));
}

#[test]
fn reporting_events_mixed_fixture_preserves_fallback_rows() {
    let detected = detect_fixture("reporting_events/mixed/ReportingEvents.log");
    assert_selection(
        &detected,
        "ReportingEvents",
        "ReportingEvents",
        "Dedicated",
        "Structured",
        "PhysicalLine",
    );

    let parsed = parse_fixture("reporting_events/mixed/ReportingEvents.log");
    assert_parsed_selection(
        &parsed,
        "ReportingEvents",
        "ReportingEvents",
        "Dedicated",
        "Structured",
        "PhysicalLine",
        "Timestamped",
    );
    assert_eq!(parsed.total_lines, 4);
    assert_eq!(parsed.parse_errors, 2);
    assert_eq!(parsed.entries.len(), 4);
    assert_eq!(
        parsed.entries[1].message,
        "{33333333-3333-3333-3333-333333333333}\tnot-a-timestamp\t2\tSoftware Update\t3\t{44444444-4444-4444-4444-444444444444}\t0x80240022\tWindows Update Agent\tFailure\tInstallation\tInstallation failed for KB5034441"
    );
    assert_eq!(parsed.entries[2].message, "orphan raw line");
    assert_eq!(parsed.entries[3].severity, "Warning");
    assert_eq!(parsed.entries[3].line_number, 4);
}
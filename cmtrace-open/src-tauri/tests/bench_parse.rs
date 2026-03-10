use std::time::Instant;

/// Returns true if running in debug (unoptimized) mode.
fn is_debug_build() -> bool {
    cfg!(debug_assertions)
}

#[test]
fn bench_parse_100k_lines() {
    let path = "/tmp/test_large.log";
    if !std::path::Path::new(path).exists() {
        eprintln!("Skipping benchmark: {} not found", path);
        return;
    }
    if is_debug_build() {
        eprintln!("Skipping perf assertions in debug build (run with --release)");
    }

    // Measure file read
    let start = Instant::now();
    let content = std::fs::read_to_string(path).unwrap();
    let read_ms = start.elapsed().as_millis();

    let file_size_mb = content.len() as f64 / 1024.0 / 1024.0;
    let line_count = content.lines().count();
    eprintln!("File: {:.1} MB, {} lines", file_size_mb, line_count);
    eprintln!("Read:  {}ms", read_ms);

    // Measure CCM parse
    let start = Instant::now();
    let result = app_lib::parser::parse_file(path);
    let parse_ms = start.elapsed().as_millis();

    match result {
        Ok((r, _date_order)) => {
            eprintln!("Parse: {}ms ({} entries, {} errors, format: {:?})",
                parse_ms, r.entries.len(), r.parse_errors, r.format_detected);
            eprintln!("Throughput: {:.0} lines/sec", r.entries.len() as f64 / (parse_ms as f64 / 1000.0));
            assert!(r.entries.len() > 50000, "Expected at least 50K entries parsed");
        }
        Err(e) => panic!("Parse failed: {}", e),
    }

    // Performance assertion: 100K lines should parse in under 2 seconds (release only)
    if !is_debug_build() {
        assert!(parse_ms < 2000, "Parse took {}ms, expected <2000ms", parse_ms);
    }
}

#[test]
fn bench_intune_analysis_100k() {
    let path = "/tmp/test_large.log";
    if !std::path::Path::new(path).exists() {
        eprintln!("Skipping benchmark: {} not found", path);
        return;
    }
    if is_debug_build() {
        eprintln!("Skipping perf assertions in debug build (run with --release)");
    }

    let start = Instant::now();
    let content = std::fs::read_to_string(path).unwrap();
    let read_ms = start.elapsed().as_millis();

    // Measure IME parse
    let start = Instant::now();
    let lines = app_lib::intune::ime_parser::parse_ime_content(&content);
    let ime_ms = start.elapsed().as_millis();
    eprintln!("IME parse: {}ms ({} lines)", ime_ms, lines.len());

    // Measure event extraction
    let start = Instant::now();
    let events = app_lib::intune::event_tracker::extract_events(&lines, path);
    let event_ms = start.elapsed().as_millis();
    eprintln!("Event extract: {}ms ({} events)", event_ms, events.len());

    // Measure timeline build
    let start = Instant::now();
    let timeline = app_lib::intune::timeline::build_timeline(events);
    let timeline_ms = start.elapsed().as_millis();
    eprintln!("Timeline: {}ms ({} events)", timeline_ms, timeline.len());

    // Measure download stats
    let start = Instant::now();
    let downloads = app_lib::intune::download_stats::extract_downloads(&lines);
    let download_ms = start.elapsed().as_millis();
    eprintln!("Downloads: {}ms ({} stats)", download_ms, downloads.len());

    let total_ms = read_ms + ime_ms + event_ms + timeline_ms + download_ms;
    eprintln!("Total Intune analysis: {}ms", total_ms);

    // Intune analysis of 100K lines should complete in under 5 seconds (release only)
    if !is_debug_build() {
        assert!(total_ms < 5000, "Intune analysis took {}ms, expected <5000ms", total_ms);
    }
}

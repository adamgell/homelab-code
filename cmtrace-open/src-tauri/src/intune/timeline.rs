use std::collections::HashSet;

use chrono::NaiveDateTime;

use super::models::{IntuneEvent, IntuneStatus};

/// Sort events chronologically and deduplicate paired events.
/// After `event_tracker::extract_events` has already paired start/end events,
/// this function cleans up the timeline by:
/// 1. Removing "end" events that were consumed by pairing
/// 2. Sorting by start_time
/// 3. Re-assigning sequential IDs
pub fn build_timeline(events: Vec<IntuneEvent>) -> Vec<IntuneEvent> {
    let mut timeline = deduplicate_events(events);

    // Sort by parsed timestamp first, then source+line for deterministic ordering.
    timeline.sort_by(|a, b| {
        let a_time = parsed_event_time(a);
        let b_time = parsed_event_time(b);

        a_time
            .cmp(&b_time)
            .then_with(|| a.source_file.cmp(&b.source_file))
            .then_with(|| a.line_number.cmp(&b.line_number))
            .then_with(|| a.name.cmp(&b.name))
    });

    // Re-assign sequential IDs
    for (i, event) in timeline.iter_mut().enumerate() {
        event.id = i as u64;
    }

    timeline
}

/// Remove duplicate or already-consumed events before timeline ordering.
/// This keeps paired start events, drops consumed completion rows when they
/// still leak through, and removes exact duplicate entries from the same file.
fn deduplicate_events(events: Vec<IntuneEvent>) -> Vec<IntuneEvent> {
    let paired_keys: HashSet<(Option<String>, String, String)> = events
        .iter()
        .filter(|e| e.end_time.is_some())
        .map(|e| {
            (
                e.guid.clone(),
                format!("{:?}", e.event_type),
                e.source_file.clone(),
            )
        })
        .collect();

    let mut seen_exact: HashSet<(String, u32, String, String, Option<String>, String)> =
        HashSet::new();

    let mut result = Vec::new();
    for event in events {
        let exact_key = (
            event.source_file.clone(),
            event.line_number,
            format!("{:?}", event.event_type),
            format!("{:?}", event.status),
            event.start_time.clone(),
            event.name.clone(),
        );

        if !seen_exact.insert(exact_key) {
            continue;
        }

        if event.end_time.is_some() {
            result.push(event);
            continue;
        }

        if event.guid.is_none() {
            result.push(event);
            continue;
        }

        let key = (
            event.guid.clone(),
            format!("{:?}", event.event_type),
            event.source_file.clone(),
        );
        let is_consumed_end = (event.status == IntuneStatus::Success
            || event.status == IntuneStatus::Failed
            || event.status == IntuneStatus::Timeout)
            && paired_keys.contains(&key);

        if !is_consumed_end {
            result.push(event);
        }
    }

    result
}

fn parsed_event_time(event: &IntuneEvent) -> Option<NaiveDateTime> {
    event
        .start_time
        .as_deref()
        .and_then(parse_timestamp)
        .or_else(|| event.end_time.as_deref().and_then(parse_timestamp))
}

fn parse_timestamp(value: &str) -> Option<NaiveDateTime> {
    const FORMATS: &[&str] = &[
        "%m-%d-%Y %H:%M:%S%.f",
        "%Y-%m-%d %H:%M:%S%.f",
        "%Y/%m/%d %H:%M:%S%.f",
    ];

    for format in FORMATS {
        if let Ok(parsed) = NaiveDateTime::parse_from_str(value, format) {
            return Some(parsed);
        }
    }

    None
}

/// Calculate the time span covered by a set of events.
/// Returns a human-readable string like "2h 15m 30s".
pub fn calculate_time_span(events: &[IntuneEvent]) -> Option<String> {
    if events.is_empty() {
        return None;
    }

    let mut earliest: Option<&str> = None;
    let mut latest: Option<&str> = None;

    for event in events {
        if let Some(ref t) = event.start_time {
            match earliest {
                None => earliest = Some(t.as_str()),
                Some(e) if t.as_str() < e => earliest = Some(t.as_str()),
                _ => {}
            }
        }
        // Check end_time too for latest
        let end = event.end_time.as_deref().or(event.start_time.as_deref());
        if let Some(t) = end {
            match latest {
                None => latest = Some(t),
                Some(l) if t > l => latest = Some(t),
                _ => {}
            }
        }
    }

    match (earliest, latest) {
        (Some(start), Some(end)) => {
            let duration = estimate_duration_secs(start, end)?;
            Some(format_duration(duration))
        }
        _ => None,
    }
}

/// Estimate duration between two timestamp strings in seconds.
fn estimate_duration_secs(start: &str, end: &str) -> Option<f64> {
    let parse_seconds = |ts: &str| -> Option<f64> {
        let time_part = ts.split_whitespace().last()?;
        let parts: Vec<&str> = time_part.split(':').collect();
        if parts.len() >= 3 {
            let h: f64 = parts[0].parse().ok()?;
            let m: f64 = parts[1].parse().ok()?;
            let s: f64 = parts[2].parse().ok()?;
            Some(h * 3600.0 + m * 60.0 + s)
        } else {
            None
        }
    };

    let start_secs = parse_seconds(start)?;
    let end_secs = parse_seconds(end)?;

    let diff = end_secs - start_secs;
    if diff >= 0.0 {
        Some(diff)
    } else {
        Some(diff + 86400.0) // crossed midnight
    }
}

/// Format seconds into a human-readable duration string.
fn format_duration(total_secs: f64) -> String {
    let total = total_secs as u64;
    let hours = total / 3600;
    let minutes = (total % 3600) / 60;
    let seconds = total % 60;

    if hours > 0 {
        format!("{}h {}m {}s", hours, minutes, seconds)
    } else if minutes > 0 {
        format!("{}m {}s", minutes, seconds)
    } else {
        format!("{}s", seconds)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(0.0), "0s");
        assert_eq!(format_duration(45.0), "45s");
        assert_eq!(format_duration(125.0), "2m 5s");
        assert_eq!(format_duration(3661.0), "1h 1m 1s");
    }

    #[test]
    fn test_estimate_duration() {
        let d = estimate_duration_secs("01-01-2024 10:00:00.000", "01-01-2024 10:05:30.000");
        assert_eq!(d, Some(330.0));
    }

    #[test]
    fn test_estimate_duration_midnight() {
        let d = estimate_duration_secs("01-01-2024 23:59:00.000", "01-02-2024 00:01:00.000");
        assert_eq!(d, Some(120.0));
    }

    #[test]
    fn build_timeline_sorts_by_parsed_timestamp() {
        let timeline = build_timeline(vec![
            IntuneEvent {
                id: 0,
                event_type: super::super::models::IntuneEventType::Win32App,
                name: "Later".to_string(),
                guid: None,
                status: IntuneStatus::InProgress,
                start_time: Some("12-31-2024 10:00:00.000".to_string()),
                end_time: None,
                duration_secs: None,
                error_code: None,
                detail: "later".to_string(),
                source_file: "b.log".to_string(),
                line_number: 2,
            },
            IntuneEvent {
                id: 1,
                event_type: super::super::models::IntuneEventType::Win32App,
                name: "Earlier".to_string(),
                guid: None,
                status: IntuneStatus::InProgress,
                start_time: Some("01-01-2024 10:00:00.000".to_string()),
                end_time: None,
                duration_secs: None,
                error_code: None,
                detail: "earlier".to_string(),
                source_file: "a.log".to_string(),
                line_number: 1,
            },
        ]);

        assert_eq!(timeline[0].name, "Earlier");
        assert_eq!(timeline[1].name, "Later");
    }

    #[test]
    fn build_timeline_deduplicates_exact_duplicate_events() {
        let event = IntuneEvent {
            id: 0,
            event_type: super::super::models::IntuneEventType::Win32App,
            name: "Duplicate".to_string(),
            guid: Some("a1b2c3d4-e5f6-7890-abcd-ef1234567890".to_string()),
            status: IntuneStatus::Success,
            start_time: Some("01-01-2024 10:00:00.000".to_string()),
            end_time: None,
            duration_secs: None,
            error_code: None,
            detail: "same".to_string(),
            source_file: "a.log".to_string(),
            line_number: 1,
        };

        let timeline = build_timeline(vec![event.clone(), event]);
        assert_eq!(timeline.len(), 1);
    }
}

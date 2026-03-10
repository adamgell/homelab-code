use super::models::{IntuneEvent, IntuneStatus};

/// Sort events chronologically and deduplicate paired events.
/// After `event_tracker::extract_events` has already paired start/end events,
/// this function cleans up the timeline by:
/// 1. Removing "end" events that were consumed by pairing
/// 2. Sorting by start_time
/// 3. Re-assigning sequential IDs
pub fn build_timeline(events: Vec<IntuneEvent>) -> Vec<IntuneEvent> {
    let mut timeline = deduplicate_paired(events);

    // Sort by start_time (lexicographic on the timestamp strings works for our format)
    timeline.sort_by(|a, b| {
        let a_time = a.start_time.as_deref().unwrap_or("");
        let b_time = b.start_time.as_deref().unwrap_or("");
        a_time.cmp(b_time)
    });

    // Re-assign sequential IDs
    for (i, event) in timeline.iter_mut().enumerate() {
        event.id = i as u64;
    }

    timeline
}

/// Remove duplicate events that were consumed during pairing.
/// When event_tracker pairs a start event with an end event, the end event's
/// information is merged into the start event. We remove end events that
/// have the same GUID/type as a paired start event (one that has an end_time).
fn deduplicate_paired(events: Vec<IntuneEvent>) -> Vec<IntuneEvent> {
    // Collect GUIDs that have been paired (start event has end_time set)
    let paired_keys: Vec<(Option<String>, String)> = events
        .iter()
        .filter(|e| e.end_time.is_some())
        .map(|e| (e.guid.clone(), format!("{:?}", e.event_type)))
        .collect();

    let mut result = Vec::new();
    for event in events {
        // Keep the event if:
        // 1. It's a paired start event (has end_time)
        // 2. It's an unpaired event (no matching paired start)
        // 3. It has no GUID (can't be deduplicated)
        if event.end_time.is_some() {
            // This is a start event that was paired — keep it
            result.push(event);
        } else if event.guid.is_none() {
            // No GUID — can't deduplicate, keep it
            result.push(event);
        } else {
            // Check if this is a "consumed" end event
            let key = (event.guid.clone(), format!("{:?}", event.event_type));
            let is_consumed_end = (event.status == IntuneStatus::Success
                || event.status == IntuneStatus::Failed)
                && paired_keys.contains(&key);

            if !is_consumed_end {
                result.push(event);
            }
        }
    }

    result
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
}

//! Format auto-detection.
//!
//! Examines the first non-empty lines of file content to determine
//! whether it uses CCM, Simple, or Plain text format.
//!
//! Detection strategy (matches CMTrace binary behavior):
//! - Check for `<![LOG[` marker → CCM format
//! - Check for `$$<` delimiter → Simple format
//! - Check for ` type="` substring → CCM format (fallback indicator)
//! - Otherwise → Plain text

use crate::models::log_entry::LogFormat;

/// Detect the log format from file content.
/// Examines up to the first 20 non-empty lines.
pub fn detect_format(content: &str) -> LogFormat {
    let sample_lines: Vec<&str> = content
        .lines()
        .filter(|l| !l.trim().is_empty())
        .take(20)
        .collect();

    let mut ccm_count = 0;
    let mut simple_count = 0;

    for line in &sample_lines {
        if line.contains("<![LOG[") && line.contains("]LOG]!>") {
            ccm_count += 1;
        } else if line.contains(" type=\"") && line.contains("component=\"") {
            // Fallback CCM detection from the binary's ` type="` check
            ccm_count += 1;
        } else if line.contains("$$<") {
            simple_count += 1;
        }
    }

    if ccm_count > 0 && ccm_count >= simple_count {
        LogFormat::Ccm
    } else if simple_count > 0 {
        LogFormat::Simple
    } else {
        LogFormat::Plain
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_ccm() {
        let content = r#"<![LOG[Test message]LOG]!><time="08:00:00.000+000" date="01-01-2024" component="Test" context="" type="1" thread="100" file="">
<![LOG[Another message]LOG]!><time="08:00:01.000+000" date="01-01-2024" component="Test" context="" type="1" thread="100" file="">"#;
        assert_eq!(detect_format(content), LogFormat::Ccm);
    }

    #[test]
    fn test_detect_simple() {
        let content = r#"Message one $$<Comp1><01-01-2024 08:00:00.000+000><thread=100>
Message two $$<Comp2><01-01-2024 08:00:01.000+000><thread=200>"#;
        assert_eq!(detect_format(content), LogFormat::Simple);
    }

    #[test]
    fn test_detect_plain() {
        let content = "Just some plain text\nAnother line\nNothing special here";
        assert_eq!(detect_format(content), LogFormat::Plain);
    }
}

use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use notify::{Config, EventKind, RecommendedWatcher, RecursiveMode, Watcher};

use crate::models::log_entry::{LogEntry, LogFormat};
use crate::parser::timestamped::DateOrder;

/// Manages incremental reading of a log file from a tracked byte offset.
pub struct TailReader {
    path: PathBuf,
    byte_offset: u64,
    format: LogFormat,
    next_id: u64,
    next_line: u32,
    /// Leftover partial line from previous read (file might be written mid-line)
    partial_line: String,
    /// Date field ordering for timestamped format
    date_order: DateOrder,
}

impl TailReader {
    /// Create a new TailReader starting after the initial parse.
    pub fn new(
        path: PathBuf,
        byte_offset: u64,
        format: LogFormat,
        next_id: u64,
        next_line: u32,
        date_order: DateOrder,
    ) -> Self {
        Self {
            path,
            byte_offset,
            format,
            next_id,
            next_line,
            partial_line: String::new(),
            date_order,
        }
    }

    /// Read new content from the file since last read, parse into entries.
    /// Returns new entries and updates internal byte_offset.
    pub fn read_new_entries(&mut self) -> Result<Vec<LogEntry>, String> {
        let mut file = std::fs::File::open(&self.path)
            .map_err(|e| format!("Failed to open file for tailing: {}", e))?;

        let metadata = file
            .metadata()
            .map_err(|e| format!("Failed to read metadata: {}", e))?;

        let file_size = metadata.len();

        // File was truncated (e.g. log rotation) — reset to beginning
        if file_size < self.byte_offset {
            self.byte_offset = 0;
            self.partial_line.clear();
        }

        // No new data
        if file_size == self.byte_offset {
            return Ok(vec![]);
        }

        // Seek to our byte offset
        file.seek(SeekFrom::Start(self.byte_offset))
            .map_err(|e| format!("Failed to seek: {}", e))?;

        let bytes_to_read = file_size - self.byte_offset;
        let mut buffer = vec![0u8; bytes_to_read as usize];
        file.read_exact(&mut buffer)
            .map_err(|e| format!("Failed to read new bytes: {}", e))?;

        // Decode (UTF-8 with Windows-1252 fallback)
        let new_text = match std::str::from_utf8(&buffer) {
            Ok(s) => s.to_string(),
            Err(_) => {
                let (cow, _, _) = encoding_rs::WINDOWS_1252.decode(&buffer);
                cow.into_owned()
            }
        };

        // Prepend any partial line from last read
        let full_text = if self.partial_line.is_empty() {
            new_text
        } else {
            let combined = format!("{}{}", self.partial_line, new_text);
            self.partial_line.clear();
            combined
        };

        // Split into lines — last "line" might be partial if file is still being written
        let ends_with_newline = full_text.ends_with('\n') || full_text.ends_with("\r\n");
        let mut lines: Vec<&str> = full_text.lines().collect();

        if !ends_with_newline && !lines.is_empty() {
            // Last line is incomplete — save it for next read
            self.partial_line = lines.pop().unwrap_or("").to_string();
        }

        if lines.is_empty() {
            self.byte_offset = file_size;
            return Ok(vec![]);
        }

        // Parse the new complete lines
        let path_str = self.path.to_string_lossy().to_string();
        let (mut entries, _) = match self.format {
            LogFormat::Ccm => crate::parser::ccm::parse_lines(&lines, &path_str),
            LogFormat::Simple => crate::parser::simple::parse_lines(&lines, &path_str),
            LogFormat::Plain => crate::parser::plain::parse_lines(&lines, &path_str),
            LogFormat::Timestamped => {
                crate::parser::timestamped::parse_lines(&lines, &path_str, self.date_order)
            }
        };

        // Update IDs and line numbers to be sequential from where we left off
        for entry in &mut entries {
            entry.id = self.next_id;
            entry.line_number = self.next_line;
            self.next_id += 1;
            self.next_line += 1;
        }

        // Update byte offset (subtract the partial line bytes we kept)
        self.byte_offset = file_size - self.partial_line.len() as u64;

        Ok(entries)
    }
}

/// Represents an active tail-watching session
pub struct TailSession {
    /// Flag to signal the watcher thread to stop
    stop_flag: Arc<AtomicBool>,
    /// Flag to pause emitting events (file is still tracked)
    paused: Arc<AtomicBool>,
}

impl TailSession {
    pub fn set_paused(&self, paused: bool) {
        self.paused.store(paused, Ordering::Relaxed);
    }

    pub fn stop(&self) {
        self.stop_flag.store(true, Ordering::Relaxed);
    }
}

/// Start watching a file for changes.
/// Spawns a background thread that monitors the file and calls `on_new_entries`
/// whenever new log entries appear.
pub fn start_tail_session<F>(
    path: PathBuf,
    byte_offset: u64,
    format: LogFormat,
    next_id: u64,
    next_line: u32,
    date_order: DateOrder,
    on_new_entries: F,
) -> Result<TailSession, String>
where
    F: Fn(Vec<LogEntry>) + Send + 'static,
{
    let stop_flag = Arc::new(AtomicBool::new(false));
    let paused = Arc::new(AtomicBool::new(false));

    let stop_flag_clone = stop_flag.clone();
    let paused_clone = paused.clone();
    let watch_path = path.clone();

    std::thread::spawn(move || {
        let mut tail_reader =
            TailReader::new(path, byte_offset, format, next_id, next_line, date_order);

        // Create a channel for notify events
        let (tx, rx) = std::sync::mpsc::channel();

        let mut watcher = match RecommendedWatcher::new(tx, Config::default()) {
            Ok(w) => w,
            Err(e) => {
                log::error!("Failed to create file watcher: {}", e);
                return;
            }
        };

        // Watch the parent directory (some systems don't notify on file-level watch
        // when the file is recreated/rotated)
        let watch_dir = watch_path.parent().unwrap_or(Path::new("."));
        if let Err(e) = watcher.watch(watch_dir, RecursiveMode::NonRecursive) {
            log::error!("Failed to start watching {}: {}", watch_dir.display(), e);
            return;
        }

        log::info!("Tail watcher started for {}", watch_path.display());

        // Also do a periodic poll as a fallback (some editors/log writers
        // may not trigger filesystem events reliably)
        let poll_interval = std::time::Duration::from_millis(500);

        loop {
            if stop_flag_clone.load(Ordering::Relaxed) {
                log::info!("Tail watcher stopped for {}", watch_path.display());
                break;
            }

            // Wait for a notify event or poll timeout
            match rx.recv_timeout(poll_interval) {
                Ok(Ok(event)) => {
                    // Only react to modify/create events for our file
                    match event.kind {
                        EventKind::Modify(_) | EventKind::Create(_) => {
                            if event.paths.iter().any(|p| p == &watch_path) {
                                if !paused_clone.load(Ordering::Relaxed) {
                                    if let Ok(entries) = tail_reader.read_new_entries() {
                                        if !entries.is_empty() {
                                            on_new_entries(entries);
                                        }
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                }
                Ok(Err(e)) => {
                    log::warn!("Watcher error: {}", e);
                }
                Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                    // Periodic poll — check for changes even without FS event
                    if !paused_clone.load(Ordering::Relaxed) {
                        if let Ok(entries) = tail_reader.read_new_entries() {
                            if !entries.is_empty() {
                                on_new_entries(entries);
                            }
                        }
                    }
                }
                Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                    log::info!("Watcher channel disconnected");
                    break;
                }
            }
        }
    });

    Ok(TailSession { stop_flag, paused })
}

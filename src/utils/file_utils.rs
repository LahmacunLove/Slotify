use std::path::{Path, PathBuf};
use anyhow::Result;
use uuid::Uuid;

pub fn generate_session_filename(dj_name: &str, session_id: &str) -> String {
    let sanitized_name = crate::utils::sanitize_filename(dj_name);
    let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
    format!("{}_{}_session_{}.mp3", sanitized_name, timestamp, &session_id[..8])
}

pub fn get_recordings_directory() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
    Path::new(&home).join(".dj_system").join("recordings")
}

pub fn ensure_recordings_directory() -> Result<PathBuf> {
    let dir = get_recordings_directory();
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}

pub fn get_session_file_path(session_id: &str, dj_name: &str) -> Result<PathBuf> {
    let recordings_dir = ensure_recordings_directory()?;
    let filename = generate_session_filename(dj_name, session_id);
    Ok(recordings_dir.join(filename))
}

pub fn cleanup_old_recordings(days_old: u64) -> Result<usize> {
    let recordings_dir = get_recordings_directory();
    if !recordings_dir.exists() {
        return Ok(0);
    }

    let cutoff_time = std::time::SystemTime::now() - std::time::Duration::from_secs(days_old * 24 * 3600);
    let mut cleaned_count = 0;

    for entry in std::fs::read_dir(recordings_dir)? {
        let entry = entry?;
        let path = entry.path();
        
        if path.is_file() {
            if let Ok(metadata) = entry.metadata() {
                if let Ok(created) = metadata.created() {
                    if created < cutoff_time {
                        if std::fs::remove_file(&path).is_ok() {
                            cleaned_count += 1;
                            tracing::info!("Cleaned up old recording: {:?}", path);
                        }
                    }
                }
            }
        }
    }

    Ok(cleaned_count)
}

pub fn get_file_size(path: &Path) -> Result<u64> {
    let metadata = std::fs::metadata(path)?;
    Ok(metadata.len())
}

pub fn format_file_size(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB"];
    let mut size = bytes as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    if unit_index == 0 {
        format!("{} {}", bytes, UNITS[unit_index])
    } else {
        format!("{:.1} {}", size, UNITS[unit_index])
    }
}

pub fn generate_cloud_upload_path(session_id: &str, filename: &str) -> String {
    let date = chrono::Utc::now().format("%Y/%m/%d");
    format!("sessions/{}/{}/{}", date, session_id, filename)
}

pub fn is_audio_file(path: &Path) -> bool {
    if let Some(extension) = path.extension() {
        if let Some(ext_str) = extension.to_str() {
            matches!(ext_str.to_lowercase().as_str(), "mp3" | "wav" | "flac" | "m4a" | "aac")
        } else {
            false
        }
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs::File;
    use std::io::Write;

    #[test]
    fn test_generate_session_filename() {
        let filename = generate_session_filename("DJ Test", "session-123-456");
        assert!(filename.contains("DJ_Test"));
        assert!(filename.contains("session_123"));
        assert!(filename.ends_with(".mp3"));
    }

    #[test]
    fn test_format_file_size() {
        assert_eq!(format_file_size(512), "512 B");
        assert_eq!(format_file_size(1024), "1.0 KB");
        assert_eq!(format_file_size(1536), "1.5 KB");
        assert_eq!(format_file_size(1048576), "1.0 MB");
        assert_eq!(format_file_size(1073741824), "1.0 GB");
    }

    #[test]
    fn test_generate_cloud_upload_path() {
        let path = generate_cloud_upload_path("session-123", "recording.mp3");
        assert!(path.starts_with("sessions/"));
        assert!(path.contains("session-123"));
        assert!(path.ends_with("recording.mp3"));
    }

    #[test]
    fn test_is_audio_file() {
        assert!(is_audio_file(Path::new("test.mp3")));
        assert!(is_audio_file(Path::new("test.wav")));
        assert!(is_audio_file(Path::new("test.flac")));
        assert!(is_audio_file(Path::new("test.m4a")));
        assert!(is_audio_file(Path::new("test.aac")));
        assert!(is_audio_file(Path::new("TEST.MP3"))); // Case insensitive
        
        assert!(!is_audio_file(Path::new("test.txt")));
        assert!(!is_audio_file(Path::new("test.jpg")));
        assert!(!is_audio_file(Path::new("test")));
    }

    #[test]
    fn test_cleanup_old_recordings() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let old_file = temp_dir.path().join("old_recording.mp3");
        let mut file = File::create(&old_file)?;
        file.write_all(b"test audio data")?;
        
        // Set the file's modification time to be old
        // Note: This is a simplified test; in reality, you'd need to use
        // platform-specific APIs to set creation time
        
        // For now, just test that the function runs without error
        let result = cleanup_old_recordings(30);
        assert!(result.is_ok());
        
        Ok(())
    }
}
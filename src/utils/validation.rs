// use regex::Regex;

pub fn is_valid_email(email: &str) -> bool {
    // Simple email validation without regex dependency for now
    email.contains('@') && email.contains('.') && email.len() > 5
}

pub fn is_valid_dj_name(name: &str) -> bool {
    !name.trim().is_empty() && name.len() <= 100
}

pub fn sanitize_filename(filename: &str) -> String {
    let invalid_chars = ['/', '\\', ':', '*', '?', '"', '<', '>', '|'];
    let mut sanitized = filename.to_string();
    
    for ch in invalid_chars {
        sanitized = sanitized.replace(ch, "_");
    }
    
    // Limit length
    if sanitized.len() > 255 {
        sanitized.truncate(255);
    }
    
    sanitized
}

pub fn validate_session_duration(duration_minutes: i32) -> bool {
    duration_minutes > 0 && duration_minutes <= 480 // Max 8 hours
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_email_validation() {
        assert!(is_valid_email("test@example.com"));
        assert!(is_valid_email("user.name+tag@domain.co.uk"));
        assert!(!is_valid_email("invalid-email"));
        assert!(!is_valid_email("@example.com"));
        assert!(!is_valid_email("test@"));
        assert!(!is_valid_email(""));
    }

    #[test]
    fn test_dj_name_validation() {
        assert!(is_valid_dj_name("Valid DJ Name"));
        assert!(is_valid_dj_name("DJ123"));
        assert!(!is_valid_dj_name(""));
        assert!(!is_valid_dj_name("   "));
        assert!(!is_valid_dj_name(&"a".repeat(101)));
    }

    #[test]
    fn test_filename_sanitization() {
        assert_eq!(sanitize_filename("normal_file.mp3"), "normal_file.mp3");
        assert_eq!(sanitize_filename("file/with\\bad:chars"), "file_with_bad_chars");
        assert_eq!(sanitize_filename("file*with?bad\"chars"), "file_with_bad_chars");
        
        let long_name = "a".repeat(300);
        let sanitized = sanitize_filename(&long_name);
        assert!(sanitized.len() <= 255);
    }

    #[test]
    fn test_session_duration_validation() {
        assert!(validate_session_duration(60));
        assert!(validate_session_duration(120));
        assert!(validate_session_duration(480));
        assert!(!validate_session_duration(0));
        assert!(!validate_session_duration(-10));
        assert!(!validate_session_duration(500));
    }
}
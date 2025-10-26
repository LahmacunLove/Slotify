use chrono::{DateTime, Utc, Duration, Timelike};

pub fn format_duration(duration: Duration) -> String {
    let total_seconds = duration.num_seconds();
    let hours = total_seconds / 3600;
    let minutes = (total_seconds % 3600) / 60;
    let seconds = total_seconds % 60;

    if hours > 0 {
        format!("{}:{:02}:{:02}", hours, minutes, seconds)
    } else {
        format!("{}:{:02}", minutes, seconds)
    }
}

pub fn format_relative_time(datetime: DateTime<Utc>) -> String {
    let now = Utc::now();
    let diff = now.signed_duration_since(datetime);

    if diff.num_seconds() < 60 {
        "just now".to_string()
    } else if diff.num_minutes() < 60 {
        let minutes = diff.num_minutes();
        if minutes == 1 {
            "1 minute ago".to_string()
        } else {
            format!("{} minutes ago", minutes)
        }
    } else if diff.num_hours() < 24 {
        let hours = diff.num_hours();
        if hours == 1 {
            "1 hour ago".to_string()
        } else {
            format!("{} hours ago", hours)
        }
    } else {
        let days = diff.num_days();
        if days == 1 {
            "1 day ago".to_string()
        } else {
            format!("{} days ago", days)
        }
    }
}

pub fn estimate_end_time(start_time: DateTime<Utc>, duration_minutes: i32) -> DateTime<Utc> {
    start_time + Duration::minutes(duration_minutes as i64)
}

pub fn calculate_queue_wait_time(position: i32, average_session_minutes: i32) -> Duration {
    Duration::minutes((position - 1) as i64 * average_session_minutes as i64)
}

pub fn is_late_arrival(registered_at: DateTime<Utc>, cutoff_hour: u32) -> bool {
    let hour = registered_at.hour();
    hour >= cutoff_hour
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(Duration::seconds(30)), "0:30");
        assert_eq!(format_duration(Duration::minutes(5)), "5:00");
        assert_eq!(format_duration(Duration::minutes(65)), "1:05:00");
        assert_eq!(format_duration(Duration::seconds(3661)), "1:01:01");
    }

    #[test]
    fn test_format_relative_time() {
        let now = Utc::now();
        
        assert_eq!(format_relative_time(now - Duration::seconds(30)), "just now");
        assert_eq!(format_relative_time(now - Duration::minutes(1)), "1 minute ago");
        assert_eq!(format_relative_time(now - Duration::minutes(5)), "5 minutes ago");
        assert_eq!(format_relative_time(now - Duration::hours(1)), "1 hour ago");
        assert_eq!(format_relative_time(now - Duration::hours(3)), "3 hours ago");
        assert_eq!(format_relative_time(now - Duration::days(1)), "1 day ago");
        assert_eq!(format_relative_time(now - Duration::days(3)), "3 days ago");
    }

    #[test]
    fn test_estimate_end_time() {
        let start = Utc.with_ymd_and_hms(2024, 1, 1, 12, 0, 0).unwrap();
        let end = estimate_end_time(start, 60);
        
        assert_eq!(end, Utc.with_ymd_and_hms(2024, 1, 1, 13, 0, 0).unwrap());
    }

    #[test]
    fn test_calculate_queue_wait_time() {
        let wait_time = calculate_queue_wait_time(3, 60); // 3rd in queue, 60min sessions
        assert_eq!(wait_time, Duration::minutes(120)); // Wait for 2 sessions
        
        let wait_time = calculate_queue_wait_time(1, 60); // First in queue
        assert_eq!(wait_time, Duration::minutes(0)); // No wait
    }

    #[test]
    fn test_is_late_arrival() {
        let late_time = Utc.with_ymd_and_hms(2024, 1, 1, 23, 30, 0).unwrap();
        let early_time = Utc.with_ymd_and_hms(2024, 1, 1, 20, 0, 0).unwrap();
        
        assert!(is_late_arrival(late_time, 22));
        assert!(!is_late_arrival(early_time, 22));
    }
}
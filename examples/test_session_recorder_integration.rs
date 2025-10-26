// Test example for Session-Recorder Integration

use session_recorder_addon::services::{SessionRecorderService, SessionRecorderConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎵 Testing Session-Recorder Integration");
    println!("=====================================");

    // Initialize the session recorder service
    let config = SessionRecorderConfig {
        minio_endpoint: "http://localhost:9000".to_string(),
        minio_access_key: "admin".to_string(),
        minio_secret_key: "password123".to_string(),
        bucket_name: "session-recorder".to_string(),
        public_endpoint: "http://localhost:9000".to_string(),
    };

    println!("\n1. Connecting to MinIO at {}", config.minio_endpoint);
    
    let session_recorder = match SessionRecorderService::new(config).await {
        Ok(service) => {
            println!("   ✅ Successfully connected to Session-Recorder MinIO");
            service
        }
        Err(e) => {
            println!("   ❌ Failed to connect: {}", e);
            println!("   💡 Make sure the Session-Recorder system is running:");
            println!("      docker-compose up");
            println!("      or");
            println!("      ./start-dev.sh");
            return Ok(());
        }
    };

    // Test 2: Get available recorders
    println!("\n2. Getting available recorders:");
    match session_recorder.get_recorders().await {
        Ok(recorders) => {
            if recorders.is_empty() {
                println!("   📝 No recorders found (this is normal if no recordings exist yet)");
            } else {
                println!("   📋 Found {} recorder(s):", recorders.len());
                for (i, recorder_id) in recorders.iter().enumerate() {
                    println!("      {}. {}", i + 1, recorder_id);
                }
            }
        }
        Err(e) => {
            println!("   ❌ Failed to get recorders: {}", e);
        }
    }

    // Test 3: Get recent sessions
    println!("\n3. Getting recent sessions (last 24 hours):");
    match session_recorder.get_recent_sessions(24).await {
        Ok(sessions) => {
            if sessions.is_empty() {
                println!("   📝 No recent sessions found");
                println!("   💡 Start a recording in the Session-Recorder system to see sessions here");
            } else {
                println!("   🎵 Found {} recent session(s):", sessions.len());
                for (i, session) in sessions.iter().enumerate() {
                    println!("      {}. {} - {} ({})", 
                        i + 1, 
                        session.name, 
                        session.start_time.format("%Y-%m-%d %H:%M:%S"),
                        if session.is_closed { "Closed" } else { "Recording" }
                    );
                    
                    if let Some(ref ogg_url) = session.files.ogg_url {
                        println!("         OGG: {}", ogg_url);
                    }
                    if let Some(ref flac_url) = session.files.flac_url {
                        println!("         FLAC: {}", flac_url);
                    }
                }
            }
        }
        Err(e) => {
            println!("   ❌ Failed to get recent sessions: {}", e);
        }
    }

    // Test 4: Demonstrate session matching
    println!("\n4. Testing session matching:");
    let test_time = chrono::Utc::now() - chrono::Duration::minutes(10);
    match session_recorder.find_matching_session(test_time, 15).await {
        Ok(Some(matching_session)) => {
            println!("   🎯 Found matching session: {} ({})", 
                matching_session.name, matching_session.id);
        }
        Ok(None) => {
            println!("   📝 No matching session found for test time");
        }
        Err(e) => {
            println!("   ❌ Error finding matching session: {}", e);
        }
    }

    println!("\n✅ Session-Recorder integration test completed!");
    println!("\n🔗 Integration Features Available:");
    println!("   • Automatic session linking based on timing");
    println!("   • Direct download URLs for OGG, FLAC, and Waveform files");
    println!("   • Real-time session discovery from MinIO");
    println!("   • Presigned URLs for secure file access");
    println!("\n🌐 API Endpoints Added:");
    println!("   • GET  /api/session-recorder/available-sessions");
    println!("   • POST /api/session-recorder/link/:session_id/:recorder_id/:recorder_session_id");
    println!("   • POST /api/session-recorder/auto-link/:session_id");
    println!("   • GET  /api/session-recorder/download/:session_id/:format");
    println!("   • GET  /api/session-recorder/session/:session_id");

    Ok(())
}
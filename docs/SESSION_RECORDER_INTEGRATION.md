# Session-Recorder Integration

This document describes the integration between the DJ Session Recorder & Lottery System and the existing [Session-Recorder](https://github.com/pascalhuerst/session-recorder) system.

## Overview

The DJ System can now automatically connect to an existing Session-Recorder MinIO instance to:
- Discover available recording sessions
- Automatically link DJ sessions to recorder sessions based on timing
- Provide direct download links for recordings (OGG, FLAC, Waveform)
- Access session metadata and files

## Architecture

```
┌─────────────────────┐    ┌─────────────────────┐    ┌─────────────────────┐
│   DJ System API    │    │  Session-Recorder   │    │      MinIO S3       │
│   (Port 3000)      │◄──►│   (Port 8780)       │◄──►│   (Port 9000)       │
│                     │    │                     │    │                     │
│ • Session Management│    │ • Audio Recording   │    │ • File Storage      │
│ • Lottery System   │    │ • Real-time Stream  │    │ • Metadata          │
│ • DJ Queue         │    │ • gRPC API          │    │ • Presigned URLs    │
└─────────────────────┘    └─────────────────────┘    └─────────────────────┘
```

## Configuration

### Environment Variables

Add these to your `.env` file:

```bash
# Session-Recorder Integration
SESSION_RECORDER_ENABLED=true
SESSION_RECORDER_MINIO_ENDPOINT=http://localhost:9000
SESSION_RECORDER_MINIO_ACCESS_KEY=admin
SESSION_RECORDER_MINIO_SECRET_KEY=password123
SESSION_RECORDER_BUCKET_NAME=session-recorder
SESSION_RECORDER_PUBLIC_ENDPOINT=http://localhost:9000
SESSION_RECORDER_AUTO_LINK_TOLERANCE=5
```

### Default Configuration

| Setting | Default Value | Description |
|---------|---------------|-------------|
| `SESSION_RECORDER_ENABLED` | `true` | Enable/disable integration |
| `SESSION_RECORDER_MINIO_ENDPOINT` | `http://localhost:9000` | MinIO API endpoint |
| `SESSION_RECORDER_MINIO_ACCESS_KEY` | `admin` | MinIO access key |
| `SESSION_RECORDER_MINIO_SECRET_KEY` | `password123` | MinIO secret key |
| `SESSION_RECORDER_BUCKET_NAME` | `session-recorder` | S3 bucket name |
| `SESSION_RECORDER_PUBLIC_ENDPOINT` | `http://localhost:9000` | Public endpoint for file URLs |
| `SESSION_RECORDER_AUTO_LINK_TOLERANCE` | `5` | Minutes tolerance for auto-linking |

## Database Schema Changes

The integration adds the following columns to the `sessions` table:

```sql
ALTER TABLE sessions ADD COLUMN recorder_session_id TEXT;
ALTER TABLE sessions ADD COLUMN recorder_id TEXT;
ALTER TABLE sessions ADD COLUMN recorder_ogg_url TEXT;
ALTER TABLE sessions ADD COLUMN recorder_flac_url TEXT;
ALTER TABLE sessions ADD COLUMN recorder_waveform_url TEXT;
```

## API Endpoints

### Get Available Recorder Sessions

```http
GET /api/session-recorder/available-sessions
```

**Response:**
```json
[
  {
    "id": "session-uuid",
    "recorder_id": "recorder-uuid",
    "name": "Session Name",
    "start_time": "2024-01-01T12:00:00Z",
    "end_time": "2024-01-01T13:00:00Z",
    "duration_seconds": 3600,
    "is_closed": true,
    "keep": true,
    "files": {
      "ogg_url": "http://localhost:9000/session-recorder/recorder-id/sessions/session-id/data.ogg",
      "flac_url": "http://localhost:9000/session-recorder/recorder-id/sessions/session-id/data.flac",
      "waveform_url": "http://localhost:9000/session-recorder/recorder-id/sessions/session-id/waveform.dat",
      "overview_png_url": "http://localhost:9000/session-recorder/recorder-id/sessions/session-id/overview.png",
      "metadata_url": "http://localhost:9000/session-recorder/recorder-id/sessions/session-id/metadata.json"
    }
  }
]
```

### Link DJ Session to Recorder Session

```http
POST /api/session-recorder/link/{session_id}/{recorder_id}/{recorder_session_id}
```

**Response:** `200 OK` on success

### Auto-Link DJ Session

```http
POST /api/session-recorder/auto-link/{session_id}?tolerance_minutes=5
```

**Response:**
```json
{
  "success": true,
  "message": "Session successfully auto-linked to recorder session"
}
```

### Get Recording Download URL

```http
GET /api/session-recorder/download/{session_id}/{format}
```

**Supported formats:** `ogg`, `flac`, `waveform`

**Response:**
```json
{
  "url": "https://presigned-url-to-file",
  "expires_in_seconds": 3600,
  "format": "ogg"
}
```

### Get Session with Recorder Info

```http
GET /api/session-recorder/session/{session_id}
```

**Response:**
```json
{
  "id": "dj-session-uuid",
  "dj_id": "dj-uuid",
  "dj_name": "DJ Name",
  "started_at": "2024-01-01T12:00:00Z",
  "ended_at": "2024-01-01T13:00:00Z",
  "duration_minutes": 60,
  "download_link": "recorder-ogg-url",
  "upload_status": "uploaded",
  "session_type": "solo",
  "recording_urls": {
    "ogg_url": "https://presigned-ogg-url",
    "flac_url": "https://presigned-flac-url",
    "waveform_url": "https://presigned-waveform-url"
  }
}
```

## Session Mapping Workflow

### Automatic Linking

1. DJ starts a session in the DJ System
2. System automatically searches for recorder sessions within the tolerance window
3. If found, the sessions are automatically linked
4. Download URLs are updated to point to recorder files

### Manual Linking

1. Admin views available recorder sessions via API
2. Admin manually links a DJ session to a specific recorder session
3. System updates download URLs and metadata

### Session Discovery

The system continuously monitors the MinIO bucket for:
- New recorder IDs (new devices)
- New sessions for each recorder
- Session metadata changes
- File availability

## File Access

### Direct URLs

When sessions are linked, the DJ System provides direct access to:
- **OGG files**: Compressed audio for web playback
- **FLAC files**: Lossless audio for archival/DJ use
- **Waveform data**: For audio visualization
- **Overview images**: PNG waveform previews

### Presigned URLs

All file access uses presigned URLs for security:
- URLs expire after 1 hour by default
- No credentials needed in client applications
- Direct browser download support

## Testing

### Prerequisites

1. Session-Recorder system running:
   ```bash
   cd /path/to/session-recorder
   ./docker-build.sh up --build
   ```

2. MinIO accessible at `localhost:9000`
3. At least one recording session available

### Run Integration Test

```bash
cargo run --example test_session_recorder_integration
```

### Manual Testing

1. Start a recording in Session-Recorder system
2. Create a DJ session via API:
   ```bash
   curl -X POST http://localhost:3000/api/sessions/start \
     -H "Content-Type: application/json" \
     -d '{"dj_id": "your-dj-id"}'
   ```

3. Auto-link the session:
   ```bash
   curl -X POST http://localhost:3000/api/session-recorder/auto-link/{session-id}
   ```

4. Get download URLs:
   ```bash
   curl http://localhost:3000/api/session-recorder/download/{session-id}/ogg
   ```

## Troubleshooting

### Common Issues

1. **Connection Failed**: Verify MinIO is running and accessible
2. **No Sessions Found**: Check if recordings exist in the bucket
3. **Auto-link Failed**: Verify timing tolerance and session timestamps
4. **Download Failed**: Check MinIO credentials and bucket permissions

### Debug Commands

```bash
# Check MinIO connectivity
curl -u admin:password123 http://localhost:9000/session-recorder/

# List bucket contents
aws --endpoint-url http://localhost:9000 s3 ls s3://session-recorder/ --recursive

# Verify credentials
docker exec session-recorder-minio mc admin info minio
```

### Logs

Enable debug logging for detailed integration information:

```bash
RUST_LOG=session_recorder_addon::services::session_recorder_service=debug cargo run --bin server
```

## Performance Considerations

- Session discovery is cached for performance
- Presigned URLs reduce server load
- MinIO operations are async and non-blocking
- Large file downloads go directly to MinIO (not through DJ System)

## Security

- All file access uses presigned URLs
- MinIO credentials are configurable via environment variables
- No session files are stored in DJ System database
- File access URLs expire automatically

## Future Enhancements

- [ ] Real-time session notifications via WebSockets
- [ ] Automatic session cleanup based on DJ System retention policies  
- [ ] Advanced session matching algorithms (audio fingerprinting)
- [ ] Integration with multiple Session-Recorder instances
- [ ] Session quality metrics from recorder data
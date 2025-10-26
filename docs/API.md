# DJ Session Recorder API Documentation

## Base URL
```
http://localhost:3000/api
```

## Authentication
- Guest endpoints: No authentication required
- DJ endpoints: Session-based (planned)
- Admin endpoints: Admin authentication required

## Endpoints

### Health Check

#### GET /health
Check if the API is running.

**Response:**
```json
{
  "status": "ok",
  "message": "DJ Session Recorder API is running"
}
```

---

## DJ Management

### GET /api/djs
Get all DJs in the system.

**Response:**
```json
[
  {
    "id": "uuid",
    "name": "DJ Name",
    "email": "dj@example.com",
    "registered_at": "2024-01-01T12:00:00Z",
    "weight": 1.0,
    "is_active": true,
    "position_in_queue": null,
    "estimated_time": null
  }
]
```

### POST /api/djs/register
Register a new DJ for the lottery.

**Request:**
```json
{
  "name": "DJ Name",
  "email": "dj@example.com"  // optional
}
```

**Response:**
```json
{
  "id": "uuid",
  "name": "DJ Name",
  "email": "dj@example.com",
  "registered_at": "2024-01-01T12:00:00Z",
  "weight": 1.0,
  "is_active": true,
  "position_in_queue": null,
  "estimated_time": null
}
```

### GET /api/djs/pool
Get the current DJ pool with active DJs and queue information.

**Response:**
```json
{
  "active_djs": [...],
  "current_dj": {
    "id": "uuid",
    "name": "Current DJ",
    ...
  },
  "next_dj": {
    "id": "uuid", 
    "name": "Next DJ",
    ...
  },
  "total_count": 15
}
```

### GET /api/djs/{id}
Get a specific DJ by ID.

### PUT /api/djs/{id}
Update DJ information.

**Request:**
```json
{
  "name": "Updated Name",        // optional
  "email": "new@example.com",    // optional
  "weight": 1.5,                 // optional
  "is_active": false,            // optional
  "position_in_queue": 3         // optional
}
```

### DELETE /api/djs/{id}
Remove a DJ from the system.

### POST /api/djs/{id}/request
Submit a guest request for a DJ's set.

**Request:**
```json
{
  "guest_name": "Guest Name",
  "guest_email": "guest@example.com",
  "message": "Optional message",
  "target_dj_id": "dj-uuid"
}
```

---

## Lottery System

### POST /api/lottery/draw
Execute the lottery draw to select the next DJ.

**Response:**
```json
{
  "winner": {
    "id": "uuid",
    "name": "Winner DJ",
    ...
  },
  "participants": [
    {
      "dj": {...},
      "calculated_weight": 1.2,
      "selection_probability": 0.15
    }
  ],
  "drawn_at": "2024-01-01T12:00:00Z",
  "algorithm_used": "weighted_random"
}
```

### GET /api/lottery/queue
Get the current queue of DJs.

**Response:**
```json
[
  {
    "id": "uuid",
    "name": "DJ Name",
    "position_in_queue": 1,
    ...
  }
]
```

### GET /api/lottery/next
Get the next DJ in queue.

### GET /api/lottery/statistics
Get lottery statistics.

**Response:**
```json
{
  "total_draws": 25,
  "unique_winners": 15,
  "average_weight": 1.2,
  "fairness_score": 0.73
}
```

### POST /api/lottery/reset
Reset the lottery (admin only).

---

## Session Management

### GET /api/sessions
Get all sessions.

### GET /api/sessions/current
Get the currently active session.

**Response:**
```json
{
  "id": "uuid",
  "dj_id": "uuid",
  "dj_name": "DJ Name",
  "started_at": "2024-01-01T12:00:00Z",
  "ended_at": null,
  "duration_minutes": null,
  "download_link": null,
  "upload_status": "recording",
  "session_type": "solo"
}
```

### POST /api/sessions/start
Start a new recording session.

**Request:**
```json
{
  "dj_id": "uuid",
  "session_type": "solo"  // "solo", "b2b", "special"
}
```

### POST /api/sessions/end
End the current session.

**Request:**
```json
{
  "session_id": "uuid"
}
```

### GET /api/sessions/{id}
Get a specific session.

### GET /api/sessions/{id}/download
Get the download link for a session.

**Response:**
```json
"https://cloud-storage.example.com/sessions/uuid.mp3"
```

### GET /api/sessions/statistics
Get session statistics.

**Response:**
```json
{
  "total_sessions": 50,
  "active_sessions": 1,
  "average_duration_minutes": 67.5,
  "total_duration_hours": 156.8
}
```

---

## Admin Endpoints

### GET /api/admin/djs
Get all DJs (admin view with additional info).

### PUT /api/admin/djs/{id}
Update DJ (admin privileges).

### DELETE /api/admin/djs/{id}
Remove DJ (admin privileges).

### PUT /api/admin/djs/{id}/position
Move DJ to specific position in queue.

**Request:**
```json
{
  "new_position": 2
}
```

### GET /api/admin/queue
Get admin view of queue and lottery pool.

**Response:**
```json
{
  "lottery_pool": [...],
  "current_queue": [...],
  "statistics": {...}
}
```

### POST /api/admin/queue/reset
Reset the entire queue.

### POST /api/admin/sessions/b2b
Create a B2B session.

**Request:**
```json
{
  "dj_ids": ["uuid1", "uuid2"],
  "duration_minutes": 120
}
```

### GET /api/admin/statistics
Get comprehensive admin statistics.

### GET /api/admin/timetable
Get the current timetable with estimated times.

**Response:**
```json
[
  {
    "position": 1,
    "dj": {...},
    "estimated_start_time": "2024-01-01T13:00:00Z",
    "session_type": "solo"
  }
]
```

---

## Error Responses

All endpoints may return error responses in the following format:

### 400 Bad Request
```json
{
  "error": "Invalid request",
  "details": "Specific error message"
}
```

### 404 Not Found
```json
{
  "error": "Resource not found"
}
```

### 500 Internal Server Error
```json
{
  "error": "Internal server error"
}
```

---

## Rate Limiting

Currently no rate limiting is implemented, but it's recommended for production use.

## WebSocket Support

Future versions may include WebSocket support for real-time updates of:
- Queue changes
- Session status updates
- Lottery draws
- Admin notifications
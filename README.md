# DA Slotify

A touchscreen-based DJ queue management system with automated lottery selection, event session management, and timetable tracking.

## 🎯 Core Features

### Event Session Management
- **Event Control**: Start/stop DJ events with configurable settings
- **Slot Duration**: Set custom time slots for each DJ (default: 60 minutes)
- **Late Arrival Penalty**: Automatic penalty for DJs registering after cutoff time
- **Auto-Draw System**: Automatically draws next DJ at 50% of current set time
- **Custom Start Time**: Set specific event start time (e.g., "20:00" for 8 PM)
- **Real-time Timetable**: Live timetable showing past, current, and upcoming DJs

### DJ Lottery System
- **Fair Random Selection**: Weighted lottery system considering arrival time
- **Queue Management**: Visual display of all registered DJs
- **Position Tracking**: Shows DJ's position in the drawn queue
- **Late Registration Support**: DJs can register before AND during events
- **Automatic Weighting**: Earlier arrivals receive slight bonus, late arrivals get penalty

### User Interfaces

#### 📝 DJ Registration Tab
**Two-Column Layout:**
- **Left**: Registration form with name and email input
- **Right**: Live list of all registered DJs with draw status indicators
- **Features**:
  - Auto-refresh every 3 seconds
  - Shows which DJs have been drawn
  - "Next to play" indicator

#### 🎵 Session Tab
**Two-Column Layout:**
- **Left**:
  - Event status (active/inactive, elapsed time, current DJ)
  - Live timetable (start time | DJ name format)
  - Status indicators: ✅ Done, ▶️ Playing, ⏳ Upcoming
  - Fixed 400px height with scrolling
- **Right**:
  - QR code for guest requests
  - Request form for contacting DJs

#### ⚙️ Admin Mode
- **Event Controls**: Start/stop events with custom parameters
- **Lottery Management**: Manual draw button
- **Timetable View**: Complete event overview
- **Data Management**: Clear all data with confirmation
- **Auto-refresh**: Updates every 3 seconds
- **Features**:
  - Set slot duration (minutes)
  - Set late penalty cutoff (hours)
  - Set custom event start time (HH:MM format)

### Auto-Refresh System
All modes automatically refresh without manual intervention:
- **DJ Registration**: 3-second intervals
- **Session View**: 2-second intervals
- **Admin Mode**: 3-second intervals

## 🏗️ Technical Architecture

### Backend
- **Language**: Rust (Edition 2021)
- **Web Framework**: Axum 0.7 (async web framework)
- **Database**: SQLite with sqlx and migrations
- **API**: RESTful endpoints with JSON
- **Real-time**: Background task system for automatic lottery draws

### Frontend
- **GUI Framework**: egui/eframe 0.29 (immediate-mode GUI)
- **Image Support**: Custom logo integration with fallback
- **Target Platforms**: Linux (cross-platform compatible)
- **Interface**: Mouse/touchscreen optimized

### Key Components

#### Event Session System
- Manages overall event lifecycle
- Tracks current DJ and slot progress
- Calculates automatic draw times (50% of slot duration)
- Enforces late arrival penalties

#### Lottery Engine
- Weighted random selection algorithm
- Considers registration time for fairness
- Applies late arrival penalty (50% probability after cutoff)
- Maintains queue position tracking

#### Timetable Generation
- Shows complete DJ schedule in queue order
- Real-time status updates
- Displays actual session durations
- Formatted as "HH:MM | DJ Name"

## 🚀 Getting Started

### Prerequisites
- Rust 1.70+
- Linux/Windows/macOS (cross-platform)
- Optional: Logo file at `assets/logo.jpg` (auto-detected)

### Installation
```bash
# Clone the repository
git clone https://github.com/LahmacunLove/Slotify.git
cd slotify

# Build the project
cargo build --release

# Run the server (backend)
cargo run --bin server

# Run the GUI (frontend) - in separate terminal
cargo run --bin gui
```

### First Time Setup
1. Start the server: `cargo run --bin server`
2. Start the GUI: `cargo run --bin gui`
3. Go to Admin Mode (password: `admin123`)
4. Click "Start Event" with your desired settings
5. Register DJs in the "DJ Registration" tab
6. Monitor progress in the "Session" tab

## 📁 Project Structure

```
slotify/
├── src/
│   ├── main.rs                    # Server entry point
│   ├── api/                       # REST API endpoints
│   │   ├── dj_routes.rs          # DJ management endpoints
│   │   ├── lottery_routes.rs     # Lottery draw endpoints
│   │   ├── event_routes.rs       # Event session endpoints
│   │   └── admin_routes.rs       # Admin operations
│   ├── models/                    # Data models
│   │   ├── dj.rs                 # DJ entity and responses
│   │   ├── lottery.rs            # Lottery algorithm
│   │   ├── event_session.rs      # Event management
│   │   └── session.rs            # Individual DJ sessions
│   ├── services/                  # Business logic
│   │   ├── dj_service.rs         # DJ CRUD operations
│   │   ├── lottery_service.rs    # Lottery execution
│   │   ├── event_service.rs      # Event lifecycle
│   │   └── session_service.rs    # Session recording
│   └── gui/                       # User interface
│       ├── main.rs               # GUI entry point
│       ├── app.rs                # Main application
│       ├── api_client.rs         # HTTP client for backend
│       └── modes/                # Different view modes
│           ├── dj_mode.rs        # DJ Registration tab
│           ├── guest_mode.rs     # Session tab
│           └── admin_mode.rs     # Admin controls
├── migrations/                    # Database schema
├── assets/                        # Logo and resources
└── Cargo.toml                     # Rust dependencies
```

## 📖 API Documentation

### Event Endpoints

```http
POST /api/event/start
Content-Type: application/json

{
  "slot_duration_minutes": 60,
  "late_arrival_cutoff_hours": 2,
  "started_at": "2026-01-15T20:00:00Z"  // Optional
}
```

```http
GET /api/event/current
```

```http
POST /api/event/end
```

```http
GET /api/event/timetable
```

### DJ Management

```http
POST /api/djs/register
Content-Type: application/json

{
  "name": "DJ Name",
  "email": "dj@example.com"
}
```

```http
GET /api/djs
GET /api/djs/pool
DELETE /api/djs/{id}
```

### Lottery

```http
POST /api/lottery/draw
GET /api/lottery/queue
GET /api/lottery/statistics
POST /api/lottery/reset
```

### Session Control

```http
POST /api/sessions/start
Content-Type: application/json

{
  "dj_id": "uuid",
  "session_type": "Solo"
}
```

```http
POST /api/sessions/end?session_id={id}
GET /api/sessions/current
GET /api/sessions
```

## 🔧 Configuration

### Default Settings
- **Slot Duration**: 60 minutes
- **Late Arrival Cutoff**: 2 hours after event start
- **Auto-draw**: At 50% of slot time (e.g., 30 minutes into 60-minute slot)
- **API Server**: http://localhost:3000

### Lottery Configuration
```toml
[lottery]
base_weight = 1.0
late_arrival_penalty = 0.5  # 50% probability for late arrivals
time_block_hours = 2         # Cutoff time after event start
max_session_duration_minutes = 60
```

## 🎨 Customization

### Logo
Place your logo at `assets/logo.jpg` (JPEG format)
- Automatically detected and loaded
- Displayed at 150x150px in GUI header
- Falls back to text-only if not found

### Admin Password
Default: `admin123`
(Configured in admin_mode.rs - change for production use)

## 🧪 Testing

The system has been tested with:
- Multiple DJ registrations before event start
- Late DJ arrivals during event
- Automatic lottery draws
- Session start/end workflows
- Timetable generation and updates
- Clear all data functionality

## 📋 Recent Updates

### Latest Features (v0.1.0)
- ✅ Two-column layouts for DJ Registration and Session tabs
- ✅ Auto-refresh on all tabs (no manual refresh needed)
- ✅ Fixed timetable heights (400px Session, 250px Admin)
- ✅ Custom event start time support (HH:MM format)
- ✅ Improved "Registered DJs" display showing all active DJs
- ✅ Logo integration with automatic detection
- ✅ Enhanced clear data with error handling
- ✅ Timetable format: "Status | Time | DJ Name"

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/new-feature`)
3. Commit your changes with conventional commits
4. Push to the branch (`git push origin feature/new-feature`)
5. Open a Pull Request

## 📝 License

This project is open source and available under the [MIT License](LICENSE).

## 🙏 Acknowledgments

- Built with Rust for performance and safety
- egui for immediate-mode GUI
- Axum for high-performance async web framework
- SQLite for embedded database

---

**DA Slotify** - Fair, automated DJ queue management for your events 🎵

# DJ Session Recorder & Lottery System

A touchscreen-based DJ queue management system with automated lottery selection, session recording, and file sharing capabilities.

## 🎯 Core Features

### DJ Lottery System
- **Touchscreen Interface**: Easy registration and participation in DJ lottery
- **Pool Display**: Shows all registered DJs in real-time
- **Automated Selection**: Computer-based random selection with customizable weighting
- **Next DJ Display**: Shows upcoming DJ on login screen
- **Fair Scheduling**: Weighted lottery system considering arrival time and other factors

### Session Management
- **Session Recording**: Automatic recording of DJ sets
- **Cloud Upload**: Sessions uploaded to cloud storage upon completion
- **Share Links**: DJs receive automated download/share links for their sets
- **Session Control**: DJ-controlled start/stop functionality

### User Interfaces

#### DJ Mode
- Register for lottery participation
- View current position in queue
- Remove self from queue
- Start/end recording sessions

#### Guest Mode
- **QR Code Integration**: Scan to request sets from current/previous DJs
- **Anonymous Requests**: Contact DJs through the system without exposing personal data
- **Email Notifications**: DJs receive email requests and can respond with yes/no

#### Admin Mode
- **Manual Queue Management**: Add/remove DJs from pool
- **DJ Repositioning**: Move DJs within the queue
- **B2B Management**: Combine DJs for back-to-back sessions (1h/2h)
- **Timetable View**: Graphical overview of current schedule

## 🏗️ Technical Architecture

### Backend
- **Language**: Rust
- **Web Framework**: Axum (high-performance async web framework)
- **Database**: SQLite with migrations
- **API**: RESTful API endpoints

### Frontend
- **GUI Framework**: Native Rust GUI (egui or Tauri)
- **Target Platforms**: Linux (Raspberry Pi, Arch Linux)
- **Interface**: Touchscreen-optimized

### Weighting Algorithm
- **Base**: Random selection
- **Time-based Weighting**: Later arrivals have reduced probability
- **Blocking System**: Optional time-based blocking for fair rotation
- **Late Registration**: Support for mid-event registration

## 🚀 Getting Started

### Prerequisites
- Rust 1.70+
- Linux environment (Raspberry Pi or Arch Linux)
- Touchscreen display (optional, works with mouse/keyboard)

### Installation
```bash
# Clone the repository
git clone <repository-url>
cd sessionRecorderAddon

# Build the project
cargo build --release

# Run the application
cargo run
```

### Configuration
The system supports various configuration options:
- Weighting parameters for lottery selection
- Session duration settings
- Cloud storage configuration
- Email notification setup

## 📁 Project Structure

```
sessionRecorderAddon/
├── src/
│   ├── main.rs              # Application entry point
│   ├── api/                 # REST API endpoints
│   ├── models/              # Data models and database schema
│   ├── services/            # Business logic
│   │   ├── lottery.rs       # Lottery algorithm implementation
│   │   ├── session.rs       # Session management
│   │   └── auth.rs          # Authentication and authorization
│   ├── gui/                 # User interface components
│   └── utils/               # Utility functions
├── tests/                   # Unit and integration tests
├── docs/                    # Additional documentation
├── migrations/              # Database migrations
└── static/                  # Static assets (if needed)
```

## 🧪 Testing

```bash
# Run all tests
cargo test

# Run with coverage
cargo test --coverage

# Run specific test suite
cargo test lottery_tests
```

## 📖 API Documentation

### Core Endpoints

#### DJ Management
- `POST /api/djs/register` - Register new DJ
- `DELETE /api/djs/{id}` - Remove DJ from pool
- `GET /api/djs/pool` - Get current DJ pool
- `POST /api/djs/draw` - Execute lottery draw

#### Session Control
- `POST /api/sessions/start` - Start recording session
- `POST /api/sessions/end` - End current session
- `GET /api/sessions/{id}/download` - Get session download link

#### Admin Operations
- `PUT /api/admin/djs/{id}/position` - Move DJ in queue
- `POST /api/admin/djs/merge` - Create B2B session
- `GET /api/admin/timetable` - Get current schedule

### Authentication
The system uses role-based access control:
- **Guest**: Read-only access, can register for lottery
- **DJ**: Can manage own registration and sessions
- **Admin**: Full system control

## 🔧 Configuration

### Environment Variables
```bash
DATABASE_URL=sqlite:///data/dj_system.db
CLOUD_STORAGE_URL=https://your-cloud-storage.com
EMAIL_SMTP_SERVER=smtp.your-provider.com
EMAIL_FROM=noreply@your-domain.com
```

### Lottery Configuration
```toml
[lottery]
base_weight = 1.0
late_arrival_penalty = 0.5
time_block_hours = 2
max_session_duration_minutes = 60
```

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## 📋 Development Roadmap

- [ ] Core lottery system implementation
- [ ] Basic GUI with touchscreen support
- [ ] Session recording integration
- [ ] Cloud storage integration
- [ ] Email notification system
- [ ] Admin interface enhancements
- [ ] Mobile web interface
- [ ] Advanced analytics and reporting

## 📝 License

This project is open source and available under the [MIT License](LICENSE).

## 🙏 Acknowledgments

- Built with Rust for performance and reliability
- Designed for Linux environments, especially Raspberry Pi
- Optimized for touchscreen interfaces and event environments
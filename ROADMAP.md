# SLOTIFY - DJ Session Recorder & Lottery System
# Development Roadmap

---

## Current Project Status

**✅ Completed Components:**
- Core backend architecture (Rust + Axum)
- Database structure (SQLite with migrations)
- API endpoints (DJ, Session, Lottery, Admin, Session-Recorder routes)
- Service layer (DJ, Session, Lottery, Email, Session-Recorder services)
- Session-Recorder MinIO integration
- Native GUI foundation (egui/eframe)
- Basic data models

**⚠️ Status Unknown / Needs Verification:**
- GUI completeness (modes, components)
- Lottery weighting algorithm implementation
- Email service functionality
- Cloud storage service
- Web interface (HTML-based registration)
- Statistics/analytics module

---

## Phase 1: Core Functionality Completion 🎯

### 1.1 Lottery System Enhancement
- **Verify & complete weighting algorithm**
  - Time-based weighting for late arrivals
  - Historical participation tracking
  - "Joker" system for DJs not drawn in previous sessions
  - Configurable weighting parameters
- **Late registration support**
- **First draw timing** (20 minutes before session start)
- **Next DJ selection** (within minutes after current DJ starts)

### 1.2 Configuration System
- **Create comprehensive config module**
  - Session duration (playtime)
  - Weighting factors (late arrival penalty, joker multiplier)
  - Time tolerance for auto-linking
  - Email templates
  - Storage settings
- **Support both TOML/ENV configuration**
- **Runtime config reload capability**

### 1.3 Web Interface Development 🌐
- **HTML/CSS/JavaScript frontend**
  - DJ registration page (alias + optional email)
  - Current pool display (real-time updates)
  - Next DJ display
  - Guest mode (QR code functionality)
  - Admin panel (web-based)
- **Alternative to/complement of native GUI**
- **Responsive design for mobile/tablet**
- **WebSocket support for live updates**

---

## Phase 2: Data Management & Statistics 📊

### 2.1 Statistics Module
- **Lottery analytics**
  - Draw history per DJ
  - Waiting time statistics
  - Weighting effectiveness analysis
  - Session participation rates
- **Session analytics**
  - Total sessions per event
  - Average session duration
  - Peak times
  - DJ popularity metrics

### 2.2 Historical Session Tracking
- **Per-DJ session history**
  - All past sessions with timestamps
  - Draw success rate
  - Total playtime
  - Joker status tracking
- **Event-based session grouping**
- **Export capabilities** (CSV, JSON)

### 2.3 Database Enhancement
- **Implement CSV fallback option** (as per Projektziele)
  - CSV export/import for sessions
  - Timestamped folder structure for sets
  - Easy migration path to full database
- **Data retention policies**
- **Backup/restore functionality**

---

## Phase 3: Communication & Storage 📧☁️

### 3.1 Email Service Completion
- **Complete email module implementation**
  - Session link delivery to DJs
  - Guest request forwarding
  - Admin notifications
  - Template system
- **Email queue management**
- **Retry logic for failed sends**
- **Email verification/validation**

### 3.2 Cloud Storage Integration
- **Storage abstraction layer**
  - Support for multiple backends (MinIO, S3, local)
  - Direct upload from Session-Recorder
  - Automatic file organization
- **Link generation service**
  - Presigned URLs with expiration
  - Share link customization
- **Storage quota management**

### 3.3 Session-Recorder Direct Integration
- **Eliminate intermediate storage**
  - Direct cloud upload from Session-Recorder
  - Stream processing capabilities
  - Real-time progress updates

---

## Phase 4: User Experience & Interface 🎨

### 4.1 GUI Completion (Native)
- **Complete all three modes**
  - DJ Mode: Registration, queue view, session control
  - Guest Mode: QR scanning, request submission
  - Admin Mode: Full control panel, manual adjustments
- **Touchscreen optimization**
  - Large touch targets
  - Gesture support
  - Virtual keyboard integration

### 4.2 QR Code System
- **QR generation for active sessions**
- **Guest request workflow**
  - Anonymous contact through system
  - DJ email notification
  - Simple yes/no response mechanism
- **Request tracking and history**

### 4.3 Timetable Visualization
- **Graphical schedule view**
  - Current and upcoming DJs
  - Session durations
  - Pool status
- **Drag-and-drop for admin**
- **B2B session visualization**

---

## Phase 5: Advanced Features 🚀

### 5.1 Multi-Mode Architecture
- **External web server option**
  - Remote registration (not just local touchscreen)
  - Multiple access points
  - Load balancing support

### 5.2 B2B Session Management
- **Fusion mode for 1h/2h combined sets**
- **Shared session recording**
- **Split download links for both DJs**

### 5.3 Advanced Admin Features
- **Manual DJ pool manipulation**
  - Add/remove DJs
  - Reorder queue
  - Priority overrides
- **Emergency controls**
  - Session pause/resume
  - Quick lottery redraw
  - Block/unblock DJs

---

## Phase 6: Platform & Deployment 🖥️

### 6.1 Platform Support
- **Verify Raspberry Pi compatibility**
- **Arch Linux optimization**
- **Cross-compilation pipeline**
- **ARM binary distribution**

### 6.2 Deployment Tools
- **Docker containerization**
- **Systemd service files**
- **Auto-update mechanism**
- **Health monitoring**

### 6.3 Documentation
- **User manual** (DJ, Guest, Admin perspectives)
- **Setup guide** (Raspberry Pi specific)
- **API documentation completion**
- **Troubleshooting guide**

---

## Phase 7: Testing & Quality Assurance ✅

### 7.1 Test Coverage
- **Unit tests for all modules**
- **Integration tests for workflows**
- **End-to-end GUI tests**
- **Performance testing**

### 7.2 Real-World Testing
- **Simulated event scenarios**
- **Load testing (concurrent DJs)**
- **Network interruption handling**
- **Data corruption recovery**

---

## Architectural Overview 🏗️

Based on **Projektziele.md**, maintain strict modularity:

```
┌─────────────────────────────────────────────┐
│            Web Interface (HTML)              │
│          + Native GUI (egui/eframe)          │
└─────────────────┬───────────────────────────┘
                  │
┌─────────────────▼───────────────────────────┐
│              API Layer (Axum)                │
└─────────────────┬───────────────────────────┘
                  │
        ┌─────────┴─────────┐
        │                   │
┌───────▼─────┐   ┌────────▼──────┐
│  Services   │   │   Database    │
│  - Lottery  │   │  (SQLite/CSV) │
│  - Session  │   │               │
│  - Email    │   └───────────────┘
│  - Storage  │
│  - Stats    │
│  - Recorder │
└─────────────┘
```

### Module Responsibilities

**GUI Module** (Native + Web)
- User interface for DJ/Guest/Admin modes
- Touchscreen optimization
- Real-time updates via WebSocket

**Statistics Module**
- Lottery analytics and reporting
- Session tracking and analysis
- Data export functionality

**Database Module**
- SQLite primary storage
- CSV fallback/export option
- Data migration utilities

**Communication Module**
- Email delivery system
- Template management
- Request forwarding

**Storage Module**
- Cloud upload/download
- Link generation
- Multiple backend support

**Session-Recorder Integration**
- API communication with external recorder
- MinIO/S3 integration
- Automatic session linking

**Registration Module**
- DJ pool management
- Entry validation
- Real-time pool updates

---

## Priority Recommendations ⭐

### High Priority (Do First)
1. Verify lottery weighting algorithm works correctly
2. Build web interface for registration
3. Complete statistics module
4. Finish email service implementation
5. Test Session-Recorder integration thoroughly

### Medium Priority
6. CSV fallback system
7. Complete native GUI
8. QR code system for guests
9. Historical tracking and joker system
10. Cloud storage abstraction

### Nice to Have
11. Advanced admin features
12. B2B session management
13. Multi-server deployment
14. Advanced analytics dashboard

---

## Development Guidelines

### Code Organization
- Keep modules strictly separated
- Use service layer for all business logic
- API layer only handles HTTP concerns
- Models define data structures only

### Extensibility Requirements
- All modules must be independently replaceable
- Configuration-driven behavior where possible
- Plugin architecture for future extensions
- Clear interface boundaries

### Testing Strategy
- Unit tests for each service
- Integration tests for workflows
- API endpoint tests
- GUI interaction tests

---

## External Dependencies

### Session-Recorder (Reference Only - DO NOT MODIFY)
Located in `/home/ffx/Projekte/slotify/session-recorder/`

**Purpose:** Reference implementation for:
- API communication patterns
- MinIO/S3 integration
- Session metadata structure
- File organization

**Integration Points:**
- MinIO bucket access (session-recorder bucket)
- Session metadata reading
- File URL generation
- Auto-linking based on timestamps

---

## Next Steps

1. Review and prioritize roadmap items
2. Assess current implementation completeness
3. Identify critical gaps
4. Begin with Phase 1 core functionality
5. Establish testing framework
6. Document API contracts

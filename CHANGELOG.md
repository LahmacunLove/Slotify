# Changelog

All notable changes to DA Slotify will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2026-01-15

### Added
- **Event Session Management System**
  - Start/stop events with configurable parameters
  - Custom slot duration (default: 60 minutes)
  - Late arrival cutoff configuration (default: 2 hours)
  - Custom event start time support (HH:MM format)
  - Automatic first DJ draw when event starts
  - Real-time event status tracking

- **Automatic Lottery Drawing**
  - Background task that draws next DJ at 50% of slot time
  - Example: 60-minute slot → next DJ drawn at 30 minutes
  - Configurable through slot duration setting

- **Late Arrival Penalty System**
  - DJs registering after cutoff time receive 50% probability penalty
  - Automatic penalty application based on event start time
  - Configurable cutoff hours

- **Timetable Generation**
  - Complete event timetable showing all DJs (past, current, upcoming)
  - Format: "Status | Time | DJ Name"
  - Status indicators: ✅ Done, ▶️ Playing, ⏳ Upcoming
  - Real-time updates as event progresses
  - Fixed 400px height in Session tab
  - Fixed 250px height in Admin view

- **GUI Reorganization**
  - **DJ Registration Tab**: Two-column layout
    - Left: Registration form
    - Right: List of all registered DJs
  - **Session Tab**: Two-column layout
    - Left: Event status and timetable
    - Right: Guest request features (QR code, form)
  - Removed redundant headings and cards
  - Improved visual consistency

- **Auto-Refresh System**
  - DJ Registration: 3-second intervals
  - Session View: 2-second intervals
  - Admin Mode: 3-second intervals
  - No manual refresh needed

- **Logo Integration**
  - Custom logo support at `assets/logo.jpg`
  - Automatic detection and loading
  - Fallback to text-only if logo not found
  - Displayed at 150x150px in GUI header

- **Program Rebranding**
  - Changed name from "DJ Session Recorder & Lottery System" to "DA Slotify"
  - Updated all documentation and GUI

### Changed
- **DJ Registration Queue Display**
  - Now shows ALL registered DJs, not just drawn ones
  - Renamed "Current Queue" to "Registered DJs"
  - Added "(drawn: #X)" indicator for DJs in lottery queue
  - Added scrollable area for long lists

- **Timetable Format**
  - Changed from position-based to time-based display
  - Shows start time in HH:MM format
  - Simplified layout: Status icon | Time | DJ Name

- **DJ Registration Tab**
  - Removed "Start Session" button (admin-only function)
  - Split into two columns for better layout
  - Auto-refresh every 3 seconds

- **Event Elapsed Time Calculation**
  - Now calculates from event start time set by admin
  - Supports custom start times in the past or future

### Fixed
- **Clear All Data Functionality**
  - Improved error handling and reporting
  - Now properly deletes all DJs from database
  - Shows specific error messages if deletion fails
  - Refreshes GUI state after clearing

- **DJ Queue Display Issue**
  - Second registered DJ now properly appears in list
  - All active DJs shown regardless of draw status
  - Fixed refresh logic to fetch all DJs, not just drawn ones

- **Timetable Order**
  - First drawn DJ now appears at top (not bottom)
  - DJs shown in queue position order
  - Consistent ordering across all views

### Removed
- Test result documentation files (API_TEST_RESULTS.md, DJ_MODE_TESTING.md, etc.)
- Consolidated into CHANGELOG.md and README.md

## [Initial] - 2026-01-10

### Added
- Basic DJ lottery system with weighted random selection
- SQLite database with migrations
- REST API with Axum framework
- Native GUI with egui/eframe
- DJ registration and management
- Session recording infrastructure
- Guest mode with QR code support
- Admin mode with queue management

---

## Upcoming Features

See [ROADMAP.md](ROADMAP.md) for planned features and improvements.

## Contributing

Please see [README.md](README.md) for contribution guidelines.

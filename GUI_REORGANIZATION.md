# GUI Reorganization - Complete!

## Changes Made

### 1. Tab Names Updated ✅
- **"DJ Mode"** → **"📝 DJ Registration"**
- **"Guest Mode"** → **"🎵 Session"**
- **"Admin Mode"** → **"⚙️ Admin Mode"** (unchanged)

### 2. DJ Registration Tab (formerly DJ Mode) ✅
**Changes:**
- Removed event status display
- Focused solely on DJ registration functionality
- Shows:
  - Registration form
  - Queue status for registered DJ
  - "Start Session" button

### 3. Session Tab (formerly Guest Mode) ✅
**Major Reorganization - Two Column Layout:**

#### LEFT COLUMN: Event & Timetable
- **Event Status Panel:**
  - Active/Inactive indicator
  - Event elapsed time
  - Currently playing DJ with progress bar
  - Slot duration information
  - Refresh button

- **Timetable Panel:**
  - Complete list of all DJs
  - Position numbers
  - DJ names
  - Status indicators:
    - ✅ Done (green) - Completed sets
    - ▶️ Playing (yellow) - Currently playing
    - ⏳ Upcoming (gray) - Waiting in queue
  - Duration for completed sets
  - Scrollable list
  - Refresh button

#### RIGHT COLUMN: Guest Requests
- **Now Playing Panel** (unchanged)
  - Current DJ info
  - Previous DJ info

- **QR Code Panel** (unchanged)
  - QR code for requesting sets
  - Simulate scan button

- **Request Form** (unchanged)
  - Guest name/email inputs
  - Message field
  - Send request button

## How to Use

### Start the GUI:
```bash
cd /home/ffx/Projekte/slotify
cargo run --bin gui
```

### Workflow:

1. **DJ Registration Tab:**
   - DJs register for the lottery
   - See their position in queue

2. **Session Tab:**
   - **Left side:** Monitor event progress and see complete timetable
   - **Right side:** Guests can request DJ sets via QR code

3. **Admin Mode:**
   - Start/end events
   - Draw DJs
   - Full control panel

## Features

### Timetable Updates Dynamically:
- Shows all DJs as they are drawn
- Updates status as DJs play
- Shows completed durations
- Real-time progress bars

### Event Status Shows:
- Whether event is running
- Current DJ and progress
- Slot duration
- Elapsed time

## Testing

To test the new layout:
1. Start server: `cargo run --bin server`
2. Start GUI: `cargo run --bin gui`
3. Go to Admin → Start Event
4. Register some DJs in DJ Registration tab
5. Go to Session tab → See them in timetable (left side)
6. When DJ starts session → Status changes to "Playing"
7. Progress bar shows slot completion

## File Changes:
- `/src/gui/app.rs` - Updated tab names
- `/src/gui/modes/dj_mode.rs` - Removed event status, updated heading
- `/src/gui/modes/guest_mode.rs` - Complete reorganization with two-column layout, added event status and timetable

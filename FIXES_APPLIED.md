# Fixes Applied

## Issue 1: No Automatic First Draw
**Problem:** When starting an event, the first DJ was not automatically drawn.

**Fix:** Modified `EventService::start_event()` to automatically draw the first DJ when the event starts.

**Result:**
- When you click "Start Event" in Admin Mode, the first DJ is now automatically drawn
- You'll see a message in the server logs: "Automatically drew first DJ for new event: [DJ Name]"

## Issue 2: Empty Timetable
**Problem:** Timetable only showed DJs who had started sessions, not DJs waiting in queue.

**Fix:** Modified `EventService::get_timetable()` to include:
1. DJs who have completed their sessions (with duration)
2. DJs currently playing (in progress)
3. **DJs in the queue waiting to play (upcoming)** ← NEW

**Result:**
- Timetable now shows ALL DJs: past, current, and upcoming
- Upcoming DJs show status "Upcoming" and appear in queue order

## How to Test

### Terminal 1 - Start Server:
```bash
cd /home/ffx/Projekte/slotify
cargo run --bin server
```

### Terminal 2 - Start GUI:
```bash
cd /home/ffx/Projekte/slotify
cargo run --bin gui
```

### Test Flow:
1. **Register DJs** (DJ Mode):
   - Register 3-4 DJs with different names

2. **Start Event** (Admin Mode):
   - Login: `admin123`
   - Set slot duration: 60 minutes
   - Set late penalty: 2 hours
   - Click "Start Event"
   - **Check server logs** - should see: "Automatically drew first DJ for new event: [Name]"

3. **View Timetable** (Admin Mode):
   - Click "View Timetable"
   - Should see:
     - Position 1: [First DJ name] - Status: Upcoming
     - Position 2: [Second DJ name] - Status: Upcoming
     - Position 3: [Third DJ name] - Status: Upcoming

4. **Start First DJ Session** (DJ Mode or Admin):
   - First DJ clicks "Start Session"
   - After 30 minutes (50% of slot), second DJ will be automatically drawn

## Expected Behavior

✅ **Event starts** → First DJ automatically drawn
✅ **View timetable** → Shows all queued DJs as "Upcoming"
✅ **DJ starts session** → Status changes to "InProgress"
✅ **At 50% of slot** → Next DJ automatically drawn
✅ **DJ finishes** → Status changes to "Completed" with duration

## Notes

- The automatic draw at 50% happens in the background (every 10 seconds check)
- For testing 50% draw: Set slot duration to 2 minutes, and the next draw happens at 1 minute

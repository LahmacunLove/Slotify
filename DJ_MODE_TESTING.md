# DJ Mode Testing Guide

## ✅ What's Implemented

### DJ Mode Features (ALL CONNECTED TO REAL API)

1. **Register for Lottery** ✅
   - API: `POST /api/djs/register`
   - Input: Name + optional email
   - Returns DJ ID and confirmation

2. **View Queue Status** ✅
   - API: `GET /api/lottery/queue`
   - Shows all DJs in queue with positions
   - Displays "Next DJ"
   - Shows your position if you're in queue

3. **Remove from Lottery** ✅
   - API: `DELETE /api/djs/{id}`
   - Removes DJ from pool
   - Clears registration status

4. **Start Session** ✅
   - API: `POST /api/sessions/start`
   - Creates a new DJ session
   - Records start time

5. **Refresh Queue** ✅
   - Manual refresh button
   - Loads latest queue from API

## 🧪 How to Test DJ Mode

### Prerequisites
- API server running on `localhost:3000` ✅
- GUI application running ✅

### Test Workflow

#### Test 1: Basic Registration
1. Open GUI → Click "🎧 DJ Mode" tab
2. Enter name: "Test DJ 1"
3. Enter email: "test1@example.com" (optional)
4. Click "🎲 Register for Lottery"
5. **Expected**: Green success message "Successfully registered as 'Test DJ 1'!"
6. **Verify in terminal**:
   ```bash
   curl http://localhost:3000/api/djs | jq '.[] | {name, email}'
   ```

#### Test 2: View Queue
1. After registering, click "🔄 Refresh" button
2. **Expected**: Queue section shows all registered DJs
3. **Expected**: If queue is empty, shows "No DJs in queue yet"

#### Test 3: Lottery Draw (Use Admin Mode)
1. Switch to Admin Mode (⚙️)
2. Login with password: `admin123`
3. Click "🎲 Draw Next DJ"
4. Switch back to DJ Mode
5. Click "🔄 Refresh"
6. **Expected**: DJ who was drawn appears in queue with position number
7. **Expected**: Shows "🎵 Next DJ: [name]"

#### Test 4: Check Position Status
1. After being drawn, the registered DJ should see:
   - **If position 1**: "🎉 You're next up!"
   - **Otherwise**: "⏳ Waiting in lottery pool..."

#### Test 5: Start Session
1. Register as a DJ
2. Get drawn (use Admin Mode)
3. Click "🎵 Start Session" button
4. **Expected**: Green success "Session started! Have a great set!"
5. **Verify in terminal**:
   ```bash
   curl http://localhost:3000/api/sessions/current | jq '.'
   ```

#### Test 6: Remove from Lottery
1. Register as a DJ
2. Click "🗑️ Remove from Lottery"
3. **Expected**:
   - Green message "'[name]' removed from lottery"
   - Returns to registration form
   - Name/email fields cleared

#### Test 7: Error Handling
1. Try registering with empty name
2. **Expected**: Red error "❌ DJ name cannot be empty"
3. Stop API server
4. Try to register
5. **Expected**: Red error about connection failure

#### Test 8: Multi-DJ Workflow
1. **Terminal 1**: Register "DJ One"
   ```bash
   curl -X POST http://localhost:3000/api/djs/register \
     -H 'Content-Type: application/json' \
     -d '{"name": "DJ One", "email": "one@test.com"}'
   ```
2. **Terminal 2**: Register "DJ Two"
   ```bash
   curl -X POST http://localhost:3000/api/djs/register \
     -H 'Content-Type: application/json' \
     -d '{"name": "DJ Two", "email": "two@test.com"}'
   ```
3. **GUI - Admin Mode**: Draw both DJs
4. **GUI - DJ Mode**: Click refresh
5. **Expected**: Both DJs appear in queue with positions 1 and 2

## 📊 Data Flow

```
DJ Mode (GUI)
     ↓
ApiClient (reqwest blocking)
     ↓
API Server (localhost:3000)
     ↓
SQLite Database (/tmp/dj_system.db)
```

## 🎯 Success Criteria

- [x] Can register DJs via GUI
- [x] Registration appears in API immediately
- [x] Queue refreshes show real data
- [x] Session start works
- [x] Remove from lottery works
- [x] Error messages display correctly
- [x] Success messages display correctly
- [x] Position detection works ("You're next up!")

## 🐛 Known Limitations (For Future)

- No automatic queue refresh (must click button)
- Session end not implemented in DJ Mode (only Admin can end)
- No session status display (is a session running?)
- No validation for duplicate names

## 🔧 Debug Tips

### Check API Server Logs
The API server outputs to stdout. Watch for:
```
POST /api/djs/register - 200 OK
GET /api/lottery/queue - 200 OK
POST /api/sessions/start - 200 OK
```

### Check Database Directly
```bash
sqlite3 /tmp/dj_system.db "SELECT name, email, is_active, position_in_queue FROM djs;"
```

### Reset Database for Testing
```bash
rm /tmp/dj_system.db
# Restart API server - it will recreate the database
```

### GUI Not Updating?
Click the "🔄 Refresh" button in DJ Mode to manually pull latest data.

## 📝 Notes

- DJ Mode doesn't auto-draw DJs - that's Admin Mode's job
- Registering adds you to the pool, not the queue
- You appear in queue only after being drawn
- Email is optional for registration
- All data is persistent (SQLite)
- Server restart doesn't lose data

## 🚀 Next Steps (Future Features)

- [ ] Auto-refresh every 5 seconds
- [ ] WebSocket for live updates
- [ ] Session timer display
- [ ] End session button for current DJ
- [ ] View session history
- [ ] Download recorded sets
- [ ] Request set from other DJs (Guest Mode feature)

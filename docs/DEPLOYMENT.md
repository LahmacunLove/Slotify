# DJ Session Recorder - Deployment Guide

## System Requirements

### Minimum Requirements
- **CPU**: ARM64 (Raspberry Pi 4) or x86_64
- **RAM**: 1GB minimum, 2GB recommended
- **Storage**: 4GB for system, additional space for recordings
- **OS**: Linux (Debian/Ubuntu/Arch Linux)
- **Network**: WiFi or Ethernet connection

### Recommended Hardware
- **Raspberry Pi 4** (4GB RAM model)
- **Touchscreen**: 7" or larger capacitive touchscreen
- **Audio Interface**: USB audio interface for recording
- **Storage**: Class 10 SD card (32GB+) or USB SSD

## Installation Methods

### 1. Binary Installation (Recommended)

```bash
# Download the latest release
wget https://github.com/your-org/session-recorder-addon/releases/latest/download/dj-system.tar.gz

# Extract
tar -xzf dj-system.tar.gz
cd dj-system

# Make executable
chmod +x server gui

# Copy to system directories
sudo cp server /usr/local/bin/dj-system-server
sudo cp gui /usr/local/bin/dj-system-gui

# Copy configuration
cp .env.example /etc/dj-system/.env
```

### 2. Build from Source

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Clone repository
git clone https://github.com/your-org/session-recorder-addon.git
cd session-recorder-addon

# Build
make build

# Install
make install
```

## Configuration

### Environment Variables

Create `/etc/dj-system/.env`:

```bash
# Database
DATABASE_URL=/var/lib/dj-system/dj_system.db

# Email (for notifications)
EMAIL_SMTP_SERVER=smtp.gmail.com
EMAIL_SMTP_PORT=587
EMAIL_USERNAME=your-email@gmail.com
EMAIL_PASSWORD=your-app-password
EMAIL_FROM=dj-system@your-domain.com

# Cloud Storage (optional)
CLOUD_STORAGE_URL=https://your-storage-provider.com

# Security
ADMIN_PASSWORD=your-secure-admin-password

# Application
LOG_LEVEL=info
MAX_SESSION_DURATION=480
RECORDINGS_DIRECTORY=/var/lib/dj-system/recordings
```

### Database Setup

```bash
# Create directories
sudo mkdir -p /var/lib/dj-system/recordings
sudo mkdir -p /var/log/dj-system

# Set permissions
sudo chown -R dj-system:dj-system /var/lib/dj-system
sudo chown -R dj-system:dj-system /var/log/dj-system

# Initialize database
sudo -u dj-system dj-system-server --migrate
```

## Systemd Service Setup

### Create Service User

```bash
sudo useradd --system --no-create-home --shell /bin/false dj-system
sudo usermod -a -G audio dj-system  # For audio recording
```

### Server Service

Create `/etc/systemd/system/dj-system-server.service`:

```ini
[Unit]
Description=DJ Session Recorder API Server
After=network.target
Wants=network.target

[Service]
Type=simple
User=dj-system
Group=dj-system
WorkingDirectory=/var/lib/dj-system
Environment=RUST_LOG=info
EnvironmentFile=/etc/dj-system/.env
ExecStart=/usr/local/bin/dj-system-server
Restart=always
RestartSec=5
StandardOutput=journal
StandardError=journal

# Security settings
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/var/lib/dj-system /var/log/dj-system

[Install]
WantedBy=multi-user.target
```

### GUI Service (for kiosk mode)

Create `/etc/systemd/system/dj-system-gui.service`:

```ini
[Unit]
Description=DJ Session Recorder GUI
After=graphical-session.target
Wants=graphical-session.target

[Service]
Type=simple
User=dj-system
Group=dj-system
Environment=DISPLAY=:0
Environment=RUST_LOG=info
EnvironmentFile=/etc/dj-system/.env
ExecStart=/usr/local/bin/dj-system-gui
Restart=always
RestartSec=5

[Install]
WantedBy=graphical.target
```

### Enable Services

```bash
sudo systemctl daemon-reload
sudo systemctl enable dj-system-server
sudo systemctl enable dj-system-gui
sudo systemctl start dj-system-server
sudo systemctl start dj-system-gui
```

## Raspberry Pi Specific Setup

### 1. Enable Touchscreen

Edit `/boot/config.txt`:

```ini
# Enable touchscreen
dtoverlay=vc4-kms-v3d
dtoverlay=rpi-ft5406
max_framebuffers=2

# Rotate display if needed
display_rotate=2

# GPU memory
gpu_mem=128
```

### 2. Configure Audio

```bash
# Install audio packages
sudo apt update
sudo apt install alsa-utils pulseaudio

# Configure audio device
sudo usermod -a -G audio dj-system

# Test audio recording
arecord -l  # List recording devices
```

### 3. Kiosk Mode Setup

Create `/etc/systemd/system/kiosk.service`:

```ini
[Unit]
Description=Kiosk Mode
After=graphical.target

[Service]
Type=simple
User=pi
Environment=DISPLAY=:0
ExecStart=/usr/local/bin/dj-system-gui
Restart=always

[Install]
WantedBy=graphical.target
```

### 4. Auto-start X11

Add to `/etc/rc.local`:

```bash
# Start X11 for touchscreen
sudo -u pi startx &
```

## Network Configuration

### WiFi Setup

```bash
# Configure WiFi
sudo wpa_passphrase "YourWiFiName" "YourPassword" >> /etc/wpa_supplicant/wpa_supplicant.conf

# Enable WiFi
sudo systemctl enable wpa_supplicant
```

### Static IP (Optional)

Edit `/etc/dhcpcd.conf`:

```bash
interface wlan0
static ip_address=192.168.1.100/24
static routers=192.168.1.1
static domain_name_servers=8.8.8.8
```

## Security Considerations

### Firewall

```bash
# Install UFW
sudo apt install ufw

# Allow SSH (if needed)
sudo ufw allow ssh

# Allow DJ System
sudo ufw allow 3000/tcp

# Enable firewall
sudo ufw enable
```

### SSL/TLS (Production)

For production deployment, use a reverse proxy like nginx:

```nginx
server {
    listen 443 ssl;
    server_name dj-system.your-domain.com;
    
    ssl_certificate /path/to/cert.pem;
    ssl_certificate_key /path/to/key.pem;
    
    location / {
        proxy_pass http://localhost:3000;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
    }
}
```

## Monitoring and Maintenance

### Log Monitoring

```bash
# View logs
sudo journalctl -u dj-system-server -f
sudo journalctl -u dj-system-gui -f

# Log rotation
sudo logrotate /etc/logrotate.d/dj-system
```

### Automatic Updates

Create `/etc/cron.d/dj-system-update`:

```bash
# Update system weekly
0 2 * * 0 root apt update && apt upgrade -y

# Clean old recordings monthly
0 3 1 * * dj-system find /var/lib/dj-system/recordings -mtime +30 -delete
```

### Health Checks

Create monitoring script `/usr/local/bin/health-check.sh`:

```bash
#!/bin/bash
curl -f http://localhost:3000/health || systemctl restart dj-system-server
```

Add to crontab:
```bash
*/5 * * * * /usr/local/bin/health-check.sh
```

## Backup and Recovery

### Database Backup

```bash
#!/bin/bash
# Backup script
DATE=$(date +%Y%m%d_%H%M%S)
sqlite3 /var/lib/dj-system/dj_system.db ".backup /var/backups/dj_system_$DATE.db"

# Keep only 7 days of backups
find /var/backups -name "dj_system_*.db" -mtime +7 -delete
```

### Recordings Backup

```bash
# Sync to cloud storage
rsync -av /var/lib/dj-system/recordings/ user@backup-server:/backups/dj-recordings/
```

## Troubleshooting

### Common Issues

1. **Permission denied errors**
   ```bash
   sudo chown -R dj-system:dj-system /var/lib/dj-system
   ```

2. **Audio not working**
   ```bash
   # Check audio devices
   arecord -l
   
   # Test recording
   arecord -d 5 test.wav
   ```

3. **GUI not starting**
   ```bash
   # Check X11 permission
   xhost +local:
   
   # Check DISPLAY variable
   echo $DISPLAY
   ```

4. **Database locked**
   ```bash
   sudo systemctl stop dj-system-server
   sudo fuser -k /var/lib/dj-system/dj_system.db
   sudo systemctl start dj-system-server
   ```

### Performance Tuning

For Raspberry Pi:

```bash
# Increase swap
sudo dphys-swapfile swapoff
sudo sed -i 's/CONF_SWAPSIZE=100/CONF_SWAPSIZE=1024/' /etc/dphys-swapfile
sudo dphys-swapfile setup
sudo dphys-swapfile swapon

# Optimize for SD card
echo 'vm.swappiness=1' >> /etc/sysctl.conf
echo 'vm.dirty_background_ratio=15' >> /etc/sysctl.conf
echo 'vm.dirty_ratio=25' >> /etc/sysctl.conf
```

## Production Checklist

- [ ] Change default admin password
- [ ] Configure email notifications
- [ ] Set up SSL/TLS
- [ ] Configure firewall
- [ ] Set up monitoring
- [ ] Configure backups
- [ ] Test audio recording
- [ ] Test touchscreen functionality
- [ ] Configure automatic updates
- [ ] Document admin procedures
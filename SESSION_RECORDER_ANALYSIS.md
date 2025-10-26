# Session Recorder System - Analyse und API Dokumentation

## GitHub Repository
- Repository: https://github.com/pascalhuerst/session-recorder
- Local Path: /home/ffx/Projekt/session-recorder

## System Übersicht

Das Session-Recorder System ist ein verteiltes Audio-Aufnahmesystem mit drei Hauptkomponenten:

### Architektur
1. **C++ Chunk Source Client**: Nimmt Audio von ALSA-Geräten auf und streamt Chunks via gRPC
2. **Go Backend Server**: Empfängt Audio-Chunks, verwaltet Sessions, stellt API bereit  
3. **Vue.js Web Interface**: Benutzeroberfläche zur Verwaltung von Aufnahmen

### Service Ports
- **Web Interface**: http://localhost:3000
- **MinIO Console**: http://localhost:9090 (admin/password123)
- **Go Backend Services**:
  - ChunkSink gRPC: localhost:8779
  - SessionSource gRPC: localhost:8780
- **gRPC-Web Proxy**: http://localhost:8080
- **MinIO API**: http://localhost:9000

## Storage-System (MinIO S3)

### Bucket: `session-recorder`
### Ordnerstruktur:
```
bucket/
├── metadata.json                    # System-Metadaten
├── {recorder-id}/
│   ├── metadata.json               # Recorder-Metadaten
│   └── sessions/
│       └── {session-id}/
│           ├── metadata.json       # Session-Metadaten
│           ├── data.ogg           # Komprimierte Audio-Datei
│           ├── data.flac          # Verlustfreie Audio-Datei  
│           ├── data.raw           # Rohe Audio-Daten
│           ├── waveform.dat       # Waveform-Daten für Visualisierung
│           └── overview.png       # Waveform-Übersicht als Bild
```

### Audio-Spezifikationen
- **Format**: 48kHz, 16-bit, 2-Kanal (Stereo)
- **Raw Data**: PCM Little Endian
- **Chunk Size**: Minimum 5MB für S3-Kompatibilität

## API-Zugriffsmöglichkeiten

### 1. gRPC SessionSource API (Port 8780)

#### Proto-Definitionen
- File: `protocols/proto/sessionsource.proto`
- File: `protocols/proto/chunksink.proto`
- File: `protocols/proto/common.proto`

#### Hauptfunktionen:
```protobuf
service SessionSource {
    // Streaming
    rpc StreamRecorders(StreamRecordersRequest) returns (stream Recorder);
    rpc StreamSessions(StreamSessionRequest) returns (stream Session);

    // Unary
    rpc SetKeepSession(SetKeepSessionRequest) returns (common.Respone);
    rpc DeleteSession(DeleteSessionRequest) returns (common.Respone);
    rpc SetName(SetNameRequest) returns (common.Respone);
    rpc CreateSegment(CreateSegmentRequest) returns (common.Respone);
    rpc DeleteSegment(DeleteSegmentRequest) returns (common.Respone);
    rpc RenderSegment(RenderSegmentRequest) returns (common.Respone);
    rpc UpdateSegment(UpdateSegmentRequest) returns (common.Respone);
    rpc CutSession(CutSessionRequest) returns (common.Respone);
}
```

#### Session Info Struktur:
```protobuf
message SessionInfo {
    message Files {
        string ogg = 1;
        string flac = 2;
        string waveform = 3;
    }

    google.protobuf.Timestamp timeCreated = 2;
    google.protobuf.Timestamp timeFinished = 3;
    google.protobuf.Duration lifetime = 4;
    string name = 5;
    bool keep = 8;
    SessionState state = 9;
    repeated Segment segments = 10;
    Files inlineFiles = 11;      // URLs für direkten Zugriff
    Files downloadFiles = 12;    // URLs für Download
}
```

### 2. Web Interface (Port 3000)
- Vue.js basierte Benutzeroberfläche
- Zugriff über gRPC-Web Proxy (Port 8080)
- Code: `web/src/`

### 3. Direkte MinIO/S3 API (Port 9000)
**Credentials:** admin/password123
- Standard S3-kompatible REST API
- Direkter Zugriff auf alle Storage-Operationen

### 4. Presigned URLs
Das System generiert signierte URLs für sicheren Dateizugriff:

**Implementation:** `go/cmd/chunk_sink/session-source-handler.go:68-84`
```go
GetPresignedURL(AssetOptions, SigningOptions) string
```

**AssetOptions:**
- RecorderID uuid.UUID
- SessionID uuid.UUID  
- Filename (FILENAME_OGG, FILENAME_FLAC, FILENAME_WAVEFORM)

**SigningOptions:**
- Expires time.Duration
- Download bool
- DownloadFilename string

## Möglichkeiten für externe Programme

### Methode 1: gRPC API Integration (Empfohlen für Live-Monitoring)
```go
// Verbindung zum SessionSource Server
conn, err := grpc.Dial("localhost:8780", grpc.WithInsecure())
client := sessionsource.NewSessionSourceClient(conn)

// Alle Recorder abrufen
stream, err := client.StreamRecorders(context.Background(), &sessionsource.StreamRecordersRequest{})

// Sessions für Recorder abrufen  
sessionStream, err := client.StreamSessions(context.Background(), &sessionsource.StreamSessionRequest{
    RecorderID: recorderID,
})

// Download-URLs aus SessionInfo verwenden:
// session.DownloadFiles.Ogg
// session.DownloadFiles.Flac  
// session.DownloadFiles.Waveform
```

### Methode 2: Direkte MinIO/S3 Integration (Empfohlen für Batch-Processing)
```go
import "github.com/minio/minio-go/v7"

// MinIO Client erstellen
client, err := minio.New("localhost:9000", &minio.Options{
    Creds: credentials.NewStaticV4("admin", "password123", ""),
    Secure: false,
})

// Alle Recorder auflisten
recorders := client.ListObjects(context.Background(), "session-recorder", minio.ListObjectsOptions{
    Prefix: "",
    Recursive: false,
})

// Sessions für Recorder auflisten
sessions := client.ListObjects(context.Background(), "session-recorder", minio.ListObjectsOptions{
    Prefix: fmt.Sprintf("%s/sessions/", recorderID),
    Recursive: true,
})

// Dateien herunterladen
object, err := client.GetObject(context.Background(), "session-recorder", objectName, minio.GetObjectOptions{})
```

### Methode 3: HTTP REST API
```bash
# Alle Objekte auflisten
curl -u admin:password123 \
  "http://localhost:9000/session-recorder/"

# Datei herunterladen
curl -u admin:password123 \
  "http://localhost:9000/session-recorder/{recorder-id}/sessions/{session-id}/data.ogg" \
  -o recording.ogg

# Session Metadaten abrufen
curl -u admin:password123 \
  "http://localhost:9000/session-recorder/{recorder-id}/sessions/{session-id}/metadata.json"
```

## Verfügbare Dateiformate pro Session

### Audio-Dateien:
- **data.ogg**: Komprimierte Audio-Datei (für Streaming/Web-Playback)
- **data.flac**: Verlustfreie Audio-Datei (für Archivierung/Qualität)  
- **data.raw**: Rohe PCM-Daten (48kHz, 16-bit, 2-Kanal, Little Endian)

### Visualisierung:
- **waveform.dat**: Binäre Waveform-Daten für detaillierte Visualisierung
- **overview.png**: Waveform-Übersicht als PNG-Bild

### Metadaten:
- **metadata.json**: Session-Metadaten (Start/End-Zeit, Dauer, Name, Keep-Flag, etc.)

## Storage Interface (Go)

### Wichtige Konstanten:
```go
const (
    FILENAME_OGG      = Filename("data.ogg")
    FILENAME_FLAC     = Filename("data.flac") 
    FILENAME_WAVEFORM = Filename("waveform.dat")
    FILENAME_METADATA = Filename("metadata.json")
)
```

### Storage Interface Methods:
```go
type Storage interface {
    GetRecorders() map[uuid.UUID]Recorder
    GetSessions(recorderID uuid.UUID) map[uuid.UUID]Session
    GetSession(recorderID, sessionID uuid.UUID) (Session, error)
    DeleteSession(ctx context.Context, recorderID, sessionID uuid.UUID) error
    SetKeepSession(ctx context.Context, recorderID, sessionID uuid.UUID, keep bool) error
    GetPresignedURL(ctx context.Context, asset AssetOptions, options SigningOptions) (string, error)
}
```

### Session Struct:
```go
type Session struct {
    ID         uuid.UUID `json:"id"`
    RecorderID uuid.UUID `json:"recorder_id"`
    Name       string    `json:"name"`
    StartTime  time.Time `json:"start_time"`
    EndTime    time.Time `json:"end_time"`
    Duration   time.Duration `json:"duration"`
    IsClosed   bool `json:"is_closed"`
    Keep       bool `json:"keep"`
    Segments   map[uuid.UUID]Segment `json:"segments"`
}
```

## Development Setup

### Mit Docker:
```bash
./docker-build.sh up --build  # Alle Services starten
./docker-build.sh logs        # Logs verfolgen
./docker-build.sh down        # Services stoppen
```

### Development (ohne Docker):
```bash
./start-dev.sh                # Envoy & MinIO starten

# Go Backend
cd go/cmd/chunk_sink
S3_ENDPOINT=localhost:9000 S3_PUBLIC_ENDPOINT=localhost:9000 \
S3_ACCESS_KEY=admin S3_SECRET_KEY=password123 go run .

# Web Interface  
cd web
npm start

./stop-dev.sh                 # Envoy & MinIO stoppen
```

## Empfehlung für externe Programme

**Für Batch-Processing/Archivierung:** Methode 2 (Direkte MinIO Integration)
- Einfachste Implementation
- Vollständiger Zugriff auf alle Dateien
- Standard S3-API (gut dokumentiert)
- Keine gRPC-Komplexität

**Für Live-Monitoring:** Methode 1 (gRPC API)
- Real-time Updates über Streams
- Strukturierte Daten über Protobuf
- Integrierte Presigned URL Generation

## Wichtige Code-Dateien

### Backend (Go):
- `go/storage/storage.go` - Storage Interface Definition
- `go/storage/minio.go` - MinIO Implementation  
- `go/cmd/chunk_sink/session-source-handler.go` - gRPC SessionSource Handler
- `go/cmd/chunk_sink/main.go` - Hauptserver

### Protocol Buffers:
- `protocols/proto/sessionsource.proto` - SessionSource Service Definition
- `protocols/proto/chunksink.proto` - ChunkSink Service Definition
- `protocols/proto/common.proto` - Gemeinsame Typen

### Frontend (Vue.js):
- `web/src/grpc/procedures/` - gRPC Client Procedures
- `web/src/store/` - Vuex Stores für Sessions/Recorders
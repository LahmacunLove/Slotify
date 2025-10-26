# DJ Session Recorder - Development Guide

## Development Environment Setup

### Prerequisites

- **Rust**: 1.70+ (latest stable recommended)
- **SQLite**: 3.35+
- **Git**: For version control
- **Text Editor**: VS Code, Vim, or your preferred editor

### Install Rust

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Install additional components
rustup component add clippy rustfmt
cargo install cargo-watch
```

### Clone and Setup

```bash
# Clone the repository
git clone https://github.com/your-org/session-recorder-addon.git
cd session-recorder-addon

# Copy environment file
cp .env.example .env

# Install dependencies and build
make dev-setup
make build
```

## Project Structure

```
session-recorder-addon/
├── src/
│   ├── main.rs              # Server entry point
│   ├── api/                 # REST API routes
│   │   ├── mod.rs
│   │   ├── dj_routes.rs
│   │   ├── session_routes.rs
│   │   ├── lottery_routes.rs
│   │   └── admin_routes.rs
│   ├── models/              # Data models
│   │   ├── mod.rs
│   │   ├── dj.rs
│   │   ├── session.rs
│   │   └── lottery.rs
│   ├── services/            # Business logic
│   │   ├── mod.rs
│   │   ├── dj_service.rs
│   │   ├── session_service.rs
│   │   ├── lottery_service.rs
│   │   └── email_service.rs
│   ├── gui/                 # GUI application
│   │   ├── main.rs
│   │   ├── app.rs
│   │   ├── modes/
│   │   └── components/
│   └── utils/               # Utility functions
├── tests/                   # Unit and integration tests
├── migrations/              # Database migrations
├── docs/                    # Documentation
├── static/                  # Static assets
├── Cargo.toml              # Rust dependencies
├── Makefile                # Build automation
└── README.md               # Project overview
```

## Development Workflow

### Running the Application

```bash
# Run the API server
make run-server
# or
cargo run --bin server

# Run the GUI (in another terminal)
make run-gui
# or
cargo run --bin gui

# Run with auto-reload
make dev        # for server
make dev-gui    # for GUI
```

### Testing

```bash
# Run all tests
make test

# Run with verbose output
make test-verbose

# Run specific test
cargo test lottery_tests

# Run tests with coverage
cargo test --coverage

# Watch tests
make test-watch
```

### Code Quality

```bash
# Format code
make format

# Lint code
make lint

# Check without building
make check

# Run all quality checks
make format && make lint && make test
```

## Architecture Overview

### Backend Architecture

The backend follows a layered architecture:

1. **API Layer** (`src/api/`): HTTP endpoints using Axum
2. **Service Layer** (`src/services/`): Business logic and orchestration
3. **Model Layer** (`src/models/`): Data structures and database schema
4. **Utility Layer** (`src/utils/`): Helper functions and utilities

### Database Design

```sql
-- Core entities
djs              -- DJ registration and information
sessions         -- Recording sessions
lottery_draws    -- Lottery history
guest_requests   -- Guest set requests
system_config    -- System configuration
```

### Frontend Architecture

The GUI uses egui for cross-platform native UI:

1. **App** (`src/gui/app.rs`): Main application state
2. **Modes** (`src/gui/modes/`): DJ/Guest/Admin interfaces
3. **Components** (`src/gui/components/`): Reusable UI components

## Adding New Features

### 1. Adding a New API Endpoint

1. **Define the route** in appropriate route file:

```rust
// src/api/dj_routes.rs
async fn new_endpoint(
    State(app_state): State<Arc<AppState>>,
    Json(request): Json<NewRequest>,
) -> Result<Json<NewResponse>, StatusCode> {
    // Implementation
}

// Add to router
.route("/new-endpoint", post(new_endpoint))
```

2. **Add service method**:

```rust
// src/services/dj_service.rs
impl DjService {
    pub async fn new_functionality(&self, request: NewRequest) -> Result<NewResponse> {
        // Business logic
    }
}
```

3. **Create tests**:

```rust
// tests/dj_service_tests.rs
#[tokio::test]
async fn test_new_functionality() {
    // Test implementation
}
```

### 2. Adding Database Migrations

```bash
# Create new migration
make migration
# Enter migration name when prompted

# Edit the generated SQL file
# migrations/YYYYMMDDHHMMSS_your_migration.sql
```

Example migration:
```sql
-- Add new column
ALTER TABLE djs ADD COLUMN phone_number TEXT;

-- Create new table
CREATE TABLE dj_preferences (
    id TEXT PRIMARY KEY,
    dj_id TEXT NOT NULL REFERENCES djs(id),
    preference_key TEXT NOT NULL,
    preference_value TEXT NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);
```

### 3. Adding GUI Components

1. **Create component**:

```rust
// src/gui/components/new_component.rs
pub struct NewComponent {
    state: ComponentState,
}

impl NewComponent {
    pub fn new() -> Self {
        Self {
            state: ComponentState::default(),
        }
    }

    pub fn render(&mut self, ui: &mut egui::Ui) {
        ui.heading("New Component");
        // Component UI logic
    }
}
```

2. **Integrate into mode**:

```rust
// src/gui/modes/dj_mode.rs
use crate::gui::components::NewComponent;

impl DjMode {
    fn render_new_section(&mut self, ui: &mut egui::Ui) {
        self.new_component.render(ui);
    }
}
```

## Testing Strategy

### Unit Tests

```rust
// Test individual functions
#[test]
fn test_calculate_weight() {
    let dj = create_test_dj();
    let weight = dj.calculate_weight(0.5);
    assert!(weight > 0.0);
}
```

### Integration Tests

```rust
// Test service interactions
#[tokio::test]
async fn test_dj_registration_flow() {
    let app_state = setup_test_db().await;
    let service = DjService::new(app_state);
    
    let dj = service.register_dj(request).await.unwrap();
    assert_eq!(dj.name, "Test DJ");
}
```

### API Tests

```rust
// Test HTTP endpoints
#[tokio::test]
async fn test_register_dj_endpoint() {
    let app = create_test_app().await;
    
    let response = app
        .post("/api/djs/register")
        .json(&request)
        .send()
        .await;
        
    assert_eq!(response.status(), 200);
}
```

## Database Development

### Working with SQLx

```rust
// Query examples
let djs = sqlx::query_as::<_, Dj>(
    "SELECT * FROM djs WHERE is_active = ?"
)
.bind(true)
.fetch_all(&db)
.await?;

// Transaction example
let mut tx = db.begin().await?;

sqlx::query("INSERT INTO djs (...) VALUES (...)")
    .execute(&mut *tx)
    .await?;

tx.commit().await?;
```

### Database Testing

```rust
async fn setup_test_db() -> Arc<AppState> {
    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
    sqlx::migrate!("./migrations").run(&pool).await.unwrap();
    // Return AppState with test database
}
```

## GUI Development

### egui Patterns

```rust
// Responsive layout
ui.horizontal(|ui| {
    ui.vertical(|ui| {
        // Left column
    });
    ui.separator();
    ui.vertical(|ui| {
        // Right column
    });
});

// Touch-friendly buttons
if ui.add_sized([120.0, 50.0], egui::Button::new("Touch Me")).clicked() {
    // Handle click
}

// State management
if ui.button("Toggle").clicked() {
    self.state = !self.state;
}
```

### Async Operations in GUI

```rust
// Using runtime handle
let rt = self.rt.clone();
let api_url = self.api_url.clone();

rt.spawn(async move {
    let response = reqwest::get(&api_url).await;
    // Handle response
});
```

## Performance Considerations

### Database Optimization

```rust
// Use prepared statements
let stmt = sqlx::query_as::<_, Dj>(
    "SELECT * FROM djs WHERE id = ?"
);

// Batch operations
let mut tx = db.begin().await?;
for dj in djs {
    stmt.bind(&dj.id).execute(&mut *tx).await?;
}
tx.commit().await?;
```

### Memory Management

```rust
// Use Arc for shared state
let app_state = Arc::new(AppState::new().await?);

// Limit collection sizes
if self.cache.len() > MAX_CACHE_SIZE {
    self.cache.clear();
}
```

## Error Handling

### Service Layer

```rust
use anyhow::{Result, Context};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DjError {
    #[error("DJ not found: {id}")]
    NotFound { id: String },
    #[error("DJ already exists: {name}")]
    AlreadyExists { name: String },
}

pub async fn register_dj(&self, request: CreateDjRequest) -> Result<DjResponse> {
    let dj = Dj::new(request.name, request.email);
    
    sqlx::query("INSERT INTO djs ...")
        .execute(&self.db)
        .await
        .context("Failed to insert DJ")?;
        
    Ok(dj.into())
}
```

### API Layer

```rust
async fn handle_dj_error(err: DjError) -> StatusCode {
    match err {
        DjError::NotFound { .. } => StatusCode::NOT_FOUND,
        DjError::AlreadyExists { .. } => StatusCode::CONFLICT,
    }
}
```

## Contributing Guidelines

### Code Style

- Use `rustfmt` for formatting
- Follow Rust naming conventions
- Write documentation for public APIs
- Add tests for new functionality

### Commit Messages

```
feat: add lottery weighting algorithm
fix: resolve session overlap issue
docs: update API documentation
test: add integration tests for DJ service
refactor: simplify queue management logic
```

### Pull Request Process

1. Create feature branch from `main`
2. Implement changes with tests
3. Run quality checks: `make format && make lint && make test`
4. Update documentation if needed
5. Create pull request with clear description

## Debugging

### Logging

```rust
use tracing::{info, warn, error, debug};

info!("DJ registered: {}", dj.name);
warn!("Late arrival penalty applied: {}", penalty);
error!("Failed to process session: {}", err);
debug!("Queue state: {:?}", queue);
```

### Environment Variables

```bash
# Set log level
export RUST_LOG=debug

# Enable SQL query logging
export RUST_LOG=sqlx=debug

# Run with logging
RUST_LOG=debug cargo run
```

### Database Debugging

```bash
# Connect to SQLite directly
sqlite3 data/dj_system.db

# View tables
.tables

# Check schema
.schema djs

# Query data
SELECT * FROM djs WHERE is_active = 1;
```

## Deployment Testing

### Local Testing

```bash
# Build release version
make release

# Test with production-like config
cp .env.production .env
./target/release/server
```

### Cross-compilation for Raspberry Pi

```bash
# Add target
rustup target add aarch64-unknown-linux-gnu

# Install cross-compiler
sudo apt install gcc-aarch64-linux-gnu

# Configure Cargo
cat >> ~/.cargo/config.toml << EOF
[target.aarch64-unknown-linux-gnu]
linker = "aarch64-linux-gnu-gcc"
EOF

# Build for Pi
cargo build --target aarch64-unknown-linux-gnu --release
```

## Resources

- [Rust Documentation](https://doc.rust-lang.org/)
- [Axum Guide](https://docs.rs/axum/latest/axum/)
- [SQLx Documentation](https://docs.rs/sqlx/latest/sqlx/)
- [egui Documentation](https://docs.rs/egui/latest/egui/)
- [Tokio Tutorial](https://tokio.rs/tokio/tutorial)
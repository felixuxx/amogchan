# Architecture Overview

## 🏗️ System Architecture

The Encrypted Social Platform is built with a modular Rust architecture that combines 4chan-style anonymity with WhatsApp-style security.

```
┌─────────────────────────────────────────────────────────────┐
│                    Web Interface                            │
│                  (HTML + JavaScript)                       │
└─────────────────────┬───────────────────────────────────────┘
                      │ HTTP API
┌─────────────────────▼───────────────────────────────────────┐
│                   Axum Web Server                          │
│              (Routes + Middleware)                         │
└─────────────────────┬───────────────────────────────────────┘
                      │
┌─────────────────────▼───────────────────────────────────────┐
│                 Core Services                              │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────────────┐    │
│  │ Auth Service│ │Board Service│ │   Chat Service      │    │
│  └─────────────┘ └─────────────┘ └─────────────────────┘    │
└─────────────────────┬───────────────────────────────────────┘
                      │
┌─────────────────────▼───────────────────────────────────────┐
│              Infrastructure Layer                          │
│ ┌───────────┐ ┌─────────────┐ ┌───────────────────────┐     │
│ │  SQLite   │ │Matrix Client│ │  Crypto Service       │     │
│ │ Database  │ │             │ │  (AES-256 + Argon2)  │     │
│ └───────────┘ └─────────────┘ └───────────────────────┘     │
└─────────────────────────────────────────────────────────────┘
```

## 📁 Module Structure

### Core (`src/core/`)
- **App**: Main application orchestrator
- **Config**: Environment configuration management
- **Types**: Shared data structures and DTOs
- **Error**: Centralized error handling

### Authentication (`src/auth/`)
- **Service**: User registration, login, session management
- **Middleware**: JWT token validation and user extraction

### Board System (`src/board/`)
- **Service**: 4chan-style boards, threads, and posts
- Features: Anonymous posting, threaded discussions, image support

### Chat System (`src/chat/`)
- **Service**: WhatsApp-style private and group messaging
- Features: End-to-end encryption, group management, media sharing

### Matrix Integration (`src/matrix/`)
- **Client**: Matrix protocol client for real-time communication
- **Events**: Event handling and synchronization

### Cryptography (`src/crypto/`)
- **Service**: AES-256 encryption, Argon2 password hashing, secure tokens

### Web Layer (`src/web/`)
- **Routes**: HTTP API endpoint definitions
- **Handlers**: Request/response processing
- **Middleware**: Authentication, CORS, logging

### Storage (`src/storage/`)
- **Database**: SQLite connection and migration management
- **Repositories**: Data access patterns (placeholder)

## 🔐 Security Features

### Encryption
- **At Rest**: AES-256 encryption for sensitive data
- **In Transit**: Matrix protocol provides E2E encryption
- **Passwords**: Argon2 hashing with configurable cost

### Authentication
- **Session-based**: Secure token generation and validation
- **Anonymous Mode**: Support for anonymous users
- **Rate Limiting**: Protection against abuse

### Privacy
- **No Tracking**: No user analytics or tracking
- **Anonymous Posting**: Optional anonymous mode
- **Data Minimization**: Only essential data collected

## 🌐 API Design

### RESTful Architecture
- **Consistent patterns**: Standard HTTP methods and status codes
- **JSON communication**: Structured request/response format
- **Error handling**: Standardized error responses

### Endpoint Categories
- **Auth**: `/api/auth/*` - User management
- **Boards**: `/api/boards/*` - 4chan-style features
- **Chats**: `/api/chats/*` - WhatsApp-style messaging
- **Users**: `/api/users/*` - User profiles

## 📊 Data Models

### User Management
```rust
User {
    id: Uuid,
    username: String,
    email: Option<String>,
    matrix_user_id: String,
    is_anonymous: bool,
    created_at: DateTime<Utc>,
}
```

### Board System
```rust
Board -> Thread -> Post
```
- Hierarchical structure like traditional forums
- Support for images and rich content
- Threaded discussions with reply chains

### Chat System
```rust
Chat -> Message
Chat -> Participants
```
- Direct messages and group chats
- End-to-end encrypted messages
- Media attachment support

## 🔄 Data Flow

### Board Posting Flow
1. User creates thread/post via HTTP API
2. Content stored in SQLite database
3. Message sent to Matrix room for real-time updates
4. Other users receive updates via Matrix sync

### Chat Messaging Flow
1. User sends message via HTTP API
2. Message encrypted and stored locally
3. Sent to Matrix room (Matrix handles E2E encryption)
4. Recipients get real-time updates

### Authentication Flow
1. User registers/logs in via HTTP API
2. Password hashed with Argon2
3. Session token generated and stored
4. Token used for subsequent API calls

## 🏃 Performance Considerations

### Database
- **SQLite**: Embedded database for simplicity
- **Migrations**: Automatic schema management
- **Indexing**: Optimized queries for performance

### Caching
- **In-memory**: Session and user data caching
- **Matrix**: Event caching for offline support

### Scalability
- **Stateless design**: Easy horizontal scaling
- **Matrix federation**: Distributed architecture support
- **Modular services**: Independent scaling of components

## 🛠️ Development Workflow

### Build System
- **Cargo**: Rust package manager and build tool
- **Dependencies**: Curated set of stable crates
- **Tests**: Unit and integration testing

### Deployment
- **Docker**: Containerized deployment
- **Native**: Direct compilation and execution
- **Environment**: Configuration via environment variables

## 🔮 Future Extensions

### Planned Features
- **File uploads**: Media attachment system
- **Moderation tools**: Content filtering and management
- **Federation**: Multi-server communication
- **Mobile clients**: Native mobile applications

### Architecture Evolution
- **Microservices**: Split into independent services
- **Message queues**: Async processing with Redis/RabbitMQ
- **Load balancing**: Multiple server instances
- **CDN integration**: Static asset distribution

## 🎯 Design Principles

1. **Security First**: All data encrypted, secure by default
2. **Privacy Focused**: Minimal data collection, anonymous options
3. **Modular Design**: Clean separation of concerns
4. **Type Safety**: Rust's type system prevents many bugs
5. **Performance**: Efficient memory usage and fast execution
6. **Simplicity**: Easy to deploy and maintain
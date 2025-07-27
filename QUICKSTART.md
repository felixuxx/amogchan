# Quick Start Guide

Get your encrypted social platform running in minutes!

## 🚀 Option 1: Native Setup (Recommended)

### Prerequisites
- Rust 1.70+ (will be installed automatically)
- OpenSSL (for key generation)

### Steps
1. **Clone and setup**
```bash
git clone <your-repo>
cd encrypted-social-platform
./setup.sh
```

2. **Configure environment** (optional)
```bash
# Edit .env file with your Matrix credentials
nano .env
```

3. **Run the application**
```bash
cargo run --release
```

4. **Access the application**
- Open http://localhost:3000 in your browser
- API available at http://localhost:3000/api/

## 🐳 Option 2: Docker (Easy Deployment)

### Prerequisites
- Docker & Docker Compose

### Steps
1. **Clone repository**
```bash
git clone <your-repo>
cd encrypted-social-platform
```

2. **Set environment variables**
```bash
cp .env.example .env
# Generate encryption key
export ENCRYPTION_KEY=$(openssl rand -base64 32)
# Edit other variables as needed
```

3. **Run with Docker**
```bash
docker-compose up -d
```

4. **Access the application**
- Open http://localhost:3000

## 📱 First Steps

### 1. Register a User
```bash
curl -X POST http://localhost:3000/api/auth/register \
  -H "Content-Type: application/json" \
  -d '{
    "username": "testuser",
    "email": "test@example.com",
    "password": "securepassword",
    "is_anonymous": false
  }'
```

### 2. Create a Board
```bash
curl -X POST http://localhost:3000/api/boards \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer YOUR_TOKEN" \
  -d '{
    "name": "general",
    "title": "General Discussion",
    "description": "General discussion board",
    "is_nsfw": false,
    "is_private": false
  }'
```

### 3. Start a Chat
```bash
curl -X POST http://localhost:3000/api/chats \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer YOUR_TOKEN" \
  -d '{
    "name": "Test Chat",
    "is_group": true,
    "participants": []
  }'
```

## 🔧 Configuration

### Matrix Setup (Optional)
1. Create a Matrix account on your preferred homeserver
2. Generate an access token
3. Update your `.env` file:
```bash
MATRIX_HOMESERVER_URL=https://your-homeserver.org
MATRIX_USER_ID=@yourbot:your-homeserver.org
MATRIX_ACCESS_TOKEN=your_access_token_here
```

### Security Settings
- `ENCRYPTION_KEY`: 32-byte base64 key for data encryption
- `SESSION_SECRET`: Secret for session signing
- `BCRYPT_COST`: Password hashing cost (default: 12)

## 🌐 API Endpoints

### Authentication
- `POST /api/auth/register` - Register user
- `POST /api/auth/login` - Login user
- `GET /api/auth/me` - Get current user
- `POST /api/auth/logout` - Logout user

### Boards (4chan-style)
- `GET /api/boards` - List boards
- `POST /api/boards` - Create board
- `GET /api/boards/:name/threads` - List threads
- `POST /api/boards/:name/threads` - Create thread
- `POST /api/threads/:id/posts` - Reply to thread

### Chats (WhatsApp-style)
- `GET /api/chats` - List user chats
- `POST /api/chats` - Create chat
- `GET /api/chats/:id/messages` - Get messages
- `POST /api/chats/:id/messages` - Send message

## 🛠️ Development

### Building from Source
```bash
# Debug build
cargo build

# Release build
cargo build --release

# Run tests
cargo test

# Run with logging
RUST_LOG=debug cargo run
```

### Database Migrations
Migrations run automatically on startup. Manual control:
```bash
# Install sqlx-cli
cargo install sqlx-cli

# Create migration
sqlx migrate add your_migration_name

# Run migrations
sqlx migrate run
```

## 🔍 Troubleshooting

### Common Issues

**Port already in use**
```bash
# Change port in .env
PORT=3001
```

**Database errors**
```bash
# Remove database to reset
rm data.db
```

**Matrix connection issues**
- Check homeserver URL is correct
- Verify access token is valid
- Ensure Matrix user exists

### Getting Help

1. Check the logs: `RUST_LOG=debug cargo run`
2. Verify configuration in `.env`
3. Test API endpoints with curl
4. Check Matrix homeserver connectivity

## 🚦 Health Check

The application provides a health endpoint:
```bash
curl http://localhost:3000/health
# Should return: OK
```

## 📚 Next Steps

- Read the full [README.md](README.md) for detailed documentation
- Explore the API with the web interface
- Set up Matrix integration for real-time features
- Configure reverse proxy for production deployment
- Set up SSL/TLS certificates for HTTPS
# Encrypted Social Platform

A secure encrypted social platform built with Rust that combines features from 4chan and WhatsApp, using the Matrix protocol for end-to-end encrypted communications.

## Features

### 4chan-style Features
- **Anonymous Boards**: Create and browse topic-based boards
- **Threaded Discussions**: Start threads and reply to posts
- **Image Support**: Share images in threads and posts  
- **Board Management**: Create custom boards with descriptions
- **Real-time Updates**: Live updates via Matrix protocol

### WhatsApp-style Features
- **Private Messaging**: Secure direct messages between users
- **Group Chats**: Create and manage encrypted group conversations
- **End-to-End Encryption**: All messages are encrypted
- **User Presence**: See when users were last active
- **Media Sharing**: Share images, files, audio, and video

### Security Features
- **Matrix Protocol**: Leverages Matrix for secure, federated communication
- **AES-256 Encryption**: Strong encryption for stored data
- **Argon2 Password Hashing**: Secure password storage
- **Session Management**: Secure token-based authentication
- **Anonymous Mode**: Support for anonymous users

## Architecture

The application follows a modular architecture:

```
src/
├── core/           # Core types, configuration, and application logic
├── matrix/         # Matrix protocol integration
├── board/          # 4chan-style board functionality  
├── chat/           # WhatsApp-style messaging
├── auth/           # Authentication and authorization
├── crypto/         # Encryption and cryptographic services
├── web/            # HTTP API and web interface
└── storage/        # Database and data persistence
```

## Quick Start

### Prerequisites

- Rust 1.70+ 
- SQLite
- Matrix homeserver access (or use matrix.org)

### Setup

1. **Clone the repository**
```bash
git clone <repo-url>
cd encrypted-social-platform
```

2. **Configure environment**
```bash
cp .env.example .env
# Edit .env with your configuration
```

3. **Generate encryption keys**
```bash
# Generate a 32-byte encryption key
openssl rand -base64 32

# Add this to your .env file as ENCRYPTION_KEY
```

4. **Set up Matrix bot** (optional)
- Create a Matrix account on your homeserver
- Generate an access token
- Add the credentials to your .env file

5. **Build and run**
```bash
cargo build --release
cargo run
```

The server will start on `http://localhost:3000`

## API Endpoints

### Authentication
- `POST /api/auth/register` - Register a new user
- `POST /api/auth/login` - Login user
- `POST /api/auth/logout` - Logout user  
- `GET /api/auth/me` - Get current user info

### Boards (4chan-style)
- `GET /api/boards` - List all boards
- `POST /api/boards` - Create a new board
- `GET /api/boards/:name` - Get board details
- `GET /api/boards/:name/threads` - List threads in board
- `POST /api/boards/:name/threads` - Create new thread
- `GET /api/threads/:id` - Get thread details
- `GET /api/threads/:id/posts` - List posts in thread
- `POST /api/threads/:id/posts` - Reply to thread

### Chats (WhatsApp-style)  
- `GET /api/chats` - List user's chats
- `POST /api/chats` - Create new chat/DM
- `GET /api/chats/:id` - Get chat details
- `GET /api/chats/:id/messages` - List messages in chat
- `POST /api/chats/:id/messages` - Send message
- `POST /api/chats/:id/participants` - Add user to group chat
- `DELETE /api/chats/:id/participants/:user_id` - Remove user from chat

### Users
- `GET /api/users/:id` - Get user profile

## Configuration

### Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `HOST` | Server bind address | `0.0.0.0` |
| `PORT` | Server port | `3000` |
| `DATABASE_URL` | SQLite database path | `sqlite:./data.db` |
| `MATRIX_HOMESERVER_URL` | Matrix homeserver URL | `https://matrix.org` |
| `MATRIX_USER_ID` | Bot Matrix user ID | Required for Matrix features |
| `MATRIX_ACCESS_TOKEN` | Bot access token | Required for Matrix features |
| `ENCRYPTION_KEY` | Base64 encryption key | Required |
| `SESSION_SECRET` | Session signing secret | Required |

### Matrix Setup

To enable full Matrix integration:

1. Create a Matrix account for your bot
2. Generate an access token using matrix-js-sdk or similar
3. Add the credentials to your environment

Without Matrix credentials, the app will still work but Matrix features will be limited.

## Database Schema

The application uses SQLite with the following main tables:

- `users` - User accounts and profiles
- `boards` - 4chan-style boards  
- `threads` - Discussion threads
- `posts` - Thread replies
- `chats` - Private/group chats
- `messages` - Chat messages
- `sessions` - User sessions

## Security Considerations

- All sensitive data is encrypted at rest
- Passwords are hashed with Argon2
- Session tokens are securely generated
- Matrix provides transport encryption
- Anonymous posting is supported
- Rate limiting prevents abuse

## Development

### Project Structure

The codebase is organized into logical modules:

- **Core**: Shared types, configuration, error handling
- **Matrix**: Matrix protocol client and event handling  
- **Board**: 4chan-style board functionality
- **Chat**: WhatsApp-style messaging
- **Auth**: User authentication and sessions
- **Crypto**: Encryption services
- **Web**: HTTP API and routing
- **Storage**: Database abstraction

### Building

```bash
# Development build
cargo build

# Release build  
cargo build --release

# Run tests
cargo test

# Run with logging
RUST_LOG=debug cargo run
```

### Database Migrations

Migrations are handled automatically on startup using sqlx-migrate.

To create new migrations:
```bash
sqlx migrate add <migration_name>
```

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests
5. Submit a pull request

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Security Notice

This is experimental software. While it implements strong encryption and security practices, it has not undergone professional security auditing. Use at your own risk in production environments.
-- Users table
CREATE TABLE users (
    id TEXT PRIMARY KEY NOT NULL,
    username TEXT UNIQUE NOT NULL,
    email TEXT UNIQUE,
    password_hash TEXT,
    matrix_user_id TEXT UNIQUE NOT NULL,
    avatar_url TEXT,
    is_anonymous BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    last_seen TEXT
);

-- Boards table (4chan-style boards)
CREATE TABLE boards (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT UNIQUE NOT NULL,
    title TEXT NOT NULL,
    description TEXT,
    matrix_room_id TEXT UNIQUE NOT NULL,
    is_nsfw BOOLEAN NOT NULL DEFAULT FALSE,
    is_private BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    created_by TEXT NOT NULL,
    FOREIGN KEY (created_by) REFERENCES users(id)
);

-- Threads table (4chan-style threads)
CREATE TABLE threads (
    id TEXT PRIMARY KEY NOT NULL,
    board_id TEXT NOT NULL,
    title TEXT,
    content TEXT NOT NULL,
    image_url TEXT,
    matrix_event_id TEXT UNIQUE NOT NULL,
    is_pinned BOOLEAN NOT NULL DEFAULT FALSE,
    is_locked BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    created_by TEXT NOT NULL,
    reply_count INTEGER NOT NULL DEFAULT 0,
    last_reply_at TEXT,
    FOREIGN KEY (board_id) REFERENCES boards(id),
    FOREIGN KEY (created_by) REFERENCES users(id)
);

-- Posts table (replies to threads)
CREATE TABLE posts (
    id TEXT PRIMARY KEY NOT NULL,
    thread_id TEXT,
    board_id TEXT NOT NULL,
    content TEXT NOT NULL,
    image_url TEXT,
    matrix_event_id TEXT UNIQUE NOT NULL,
    reply_to TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    created_by TEXT NOT NULL,
    FOREIGN KEY (thread_id) REFERENCES threads(id),
    FOREIGN KEY (board_id) REFERENCES boards(id),
    FOREIGN KEY (reply_to) REFERENCES posts(id),
    FOREIGN KEY (created_by) REFERENCES users(id)
);

-- Chats table (WhatsApp-style private/group chats)
CREATE TABLE chats (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT,
    matrix_room_id TEXT UNIQUE NOT NULL,
    is_group BOOLEAN NOT NULL DEFAULT FALSE,
    is_encrypted BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    created_by TEXT NOT NULL,
    FOREIGN KEY (created_by) REFERENCES users(id)
);

-- Chat participants table
CREATE TABLE chat_participants (
    chat_id TEXT NOT NULL,
    user_id TEXT NOT NULL,
    joined_at TEXT NOT NULL DEFAULT (datetime('now')),
    is_admin BOOLEAN NOT NULL DEFAULT FALSE,
    PRIMARY KEY (chat_id, user_id),
    FOREIGN KEY (chat_id) REFERENCES chats(id),
    FOREIGN KEY (user_id) REFERENCES users(id)
);

-- Messages table (chat messages)
CREATE TABLE messages (
    id TEXT PRIMARY KEY NOT NULL,
    chat_id TEXT NOT NULL,
    content TEXT NOT NULL,
    message_type TEXT NOT NULL DEFAULT 'text',
    matrix_event_id TEXT UNIQUE NOT NULL,
    reply_to TEXT,
    is_encrypted BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    created_by TEXT NOT NULL,
    FOREIGN KEY (chat_id) REFERENCES chats(id),
    FOREIGN KEY (reply_to) REFERENCES messages(id),
    FOREIGN KEY (created_by) REFERENCES users(id)
);

-- Sessions table for authentication
CREATE TABLE sessions (
    id TEXT PRIMARY KEY NOT NULL,
    user_id TEXT NOT NULL,
    token_hash TEXT UNIQUE NOT NULL,
    expires_at TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (user_id) REFERENCES users(id)
);

-- Indexes for performance
CREATE INDEX idx_threads_board_id ON threads(board_id);
CREATE INDEX idx_threads_created_at ON threads(created_at DESC);
CREATE INDEX idx_posts_thread_id ON posts(thread_id);
CREATE INDEX idx_posts_board_id ON posts(board_id);
CREATE INDEX idx_posts_created_at ON posts(created_at DESC);
CREATE INDEX idx_messages_chat_id ON messages(chat_id);
CREATE INDEX idx_messages_created_at ON messages(created_at DESC);
CREATE INDEX idx_sessions_user_id ON sessions(user_id);
CREATE INDEX idx_sessions_token_hash ON sessions(token_hash);
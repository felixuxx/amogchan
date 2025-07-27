#!/bin/bash

set -e

echo "🔧 Setting up Encrypted Social Platform..."

# Check if Rust is installed
if ! command -v cargo &> /dev/null; then
    echo "📦 Installing Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source ~/.cargo/env
fi

# Update Rust to latest stable
echo "🔄 Updating Rust toolchain..."
rustup update

# Create environment file if it doesn't exist
if [ ! -f .env ]; then
    echo "📝 Creating environment file..."
    cp .env.example .env
    echo "⚠️  Please edit .env with your configuration before running!"
fi

# Generate a secure encryption key
if command -v openssl &> /dev/null; then
    echo "🔐 Generating encryption key..."
    ENCRYPTION_KEY=$(openssl rand -base64 32)
    sed -i "s/your-base64-encoded-32-byte-encryption-key-here/$ENCRYPTION_KEY/" .env
    echo "✅ Encryption key generated and added to .env"
fi

# Create necessary directories
mkdir -p static migrations

echo "🏗️  Building project..."
cargo build --release

echo "✅ Setup complete!"
echo ""
echo "🚀 To run the application:"
echo "   cargo run --release"
echo ""
echo "🌐 The server will be available at:"
echo "   http://localhost:3000"
echo ""
echo "📚 API documentation:"
echo "   http://localhost:3000 (see the API endpoints section)"
echo ""
echo "⚠️  Remember to:"
echo "   1. Edit .env with your Matrix credentials (optional)"
echo "   2. Set strong passwords for ENCRYPTION_KEY and SESSION_SECRET"
echo "   3. Configure your Matrix homeserver if needed"
#!/bin/bash
# First-time setup for Vast.ai GPU instance
# Run this once after creating a new instance
#
# Usage: ./vast-setup.sh [SSH_HOST] [SSH_PORT]
# Or: SSH_HOST=ssh6.vast.ai SSH_PORT=12345 ./vast-setup.sh

set -e

# Get instance info if not provided
if [ -z "$SSH_HOST" ] || [ -z "$SSH_PORT" ]; then
  export VAST_API_KEY=$(cat ~/.config/vastai/vast_api_key)
  INSTANCE_INFO=$(~/.local/bin/vastai show instances --raw | jq -r '.[] | select(.actual_status == "running") | "\(.ssh_host) \(.ssh_port)"' | head -1)
  
  if [ -z "$INSTANCE_INFO" ]; then
    echo "❌ No running instance found. Create one first:"
    echo "   vastai create instance OFFER_ID --image nvidia/cuda:12.2.0-runtime-ubuntu22.04 --disk 20 --ssh"
    exit 1
  fi
  
  SSH_HOST=$(echo $INSTANCE_INFO | cut -d' ' -f1)
  SSH_PORT=$(echo $INSTANCE_INFO | cut -d' ' -f2)
fi

SSH_CMD="ssh -o StrictHostKeyChecking=no -o ConnectTimeout=30 -p $SSH_PORT root@$SSH_HOST"

echo "🚀 Setting up instance at $SSH_HOST:$SSH_PORT..."
echo "   This takes ~5-10 minutes on first run"

$SSH_CMD 'bash -s' << 'EOF'
set -e

echo "📦 Installing system deps..."
apt-get update
apt-get install -y curl wget git xvfb build-essential libclang-dev pkg-config cmake

echo "📦 Installing Node.js 22..."
curl -fsSL https://deb.nodesource.com/setup_22.x | bash -
apt-get install -y nodejs

echo "📦 Installing Rust..."
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source ~/.cargo/env

echo "📦 Installing Chrome..."
wget -q https://dl.google.com/linux/direct/google-chrome-stable_current_amd64.deb
apt-get install -y ./google-chrome-stable_current_amd64.deb
rm google-chrome-stable_current_amd64.deb

echo "📦 Installing Puppeteer..."
cd /root && npm install puppeteer

echo "📦 Cloning Mittens..."
cd /root
git clone https://github.com/DonSqualo/Mittens.git
cd Mittens

echo "🔨 Building Rust server (this takes a while)..."
cd server && ~/.cargo/bin/cargo build --release && cd ..

echo "📦 Installing renderer deps..."
cd renderer && npm install && cd ..

echo "🖥️ Starting Xvfb..."
Xvfb :99 -screen 0 1920x1080x24 &
sleep 1

echo "✅ Setup complete!"
echo ""
echo "To start services:"
echo "  export DISPLAY=:99"
echo "  export LD_LIBRARY_PATH=\$(find /root/Mittens -name 'libmanifoldc.so' -path '*/out/lib/*' | head -1 | xargs dirname)"
echo "  cd /root/Mittens && ./server/target/release/scriptcad-server examples/multiphysics/pure_acoustics.lua &"
echo "  cd /root/Mittens/renderer && npm run dev &"
EOF

echo ""
echo "✅ Instance ready! Now run:"
echo "   ~/clawd/Mittens/scripts/vast-screenshot.sh"

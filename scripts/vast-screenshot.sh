#!/bin/bash
# Take a screenshot from running Vast.ai instance
# Usage: ./vast-screenshot.sh [--restart] [output.png]
#
# Options:
#   --restart   Kill and restart server/renderer (needed for TS/Rust changes)
#               Not needed for Lua changes (server has file watcher)
#
# Examples:
#   ./vast-screenshot.sh                     # Just screenshot
#   ./vast-screenshot.sh my_feature.png      # Screenshot with custom name
#   ./vast-screenshot.sh --restart           # Restart services then screenshot
#   ./vast-screenshot.sh --restart new.png   # Both

set -e

RESTART=false
OUTPUT="screenshot_$(date +%H%M%S).png"

# Parse args
while [[ $# -gt 0 ]]; do
  case $1 in
    --restart) RESTART=true; shift ;;
    *) OUTPUT="$1"; shift ;;
  esac
done

# Get instance SSH info
export VAST_API_KEY=$(cat ~/.config/vastai/vast_api_key)
INSTANCE_INFO=$(~/.local/bin/vastai show instances --raw | jq -r '.[] | select(.actual_status == "running") | "\(.ssh_host) \(.ssh_port)"' | head -1)

if [ -z "$INSTANCE_INFO" ]; then
  echo "❌ No running Vast.ai instance found"
  exit 1
fi

SSH_HOST=$(echo $INSTANCE_INFO | cut -d' ' -f1)
SSH_PORT=$(echo $INSTANCE_INFO | cut -d' ' -f2)
SSH_CMD="ssh -o StrictHostKeyChecking=no -o ConnectTimeout=15 -p $SSH_PORT root@$SSH_HOST"

echo "📡 Connecting to $SSH_HOST:$SSH_PORT..."

if [ "$RESTART" = true ]; then
  echo "🔄 Restarting services..."
  $SSH_CMD 'bash -s' << 'EOF'
pkill -f scriptcad-server 2>/dev/null || true
pkill -f vite 2>/dev/null || true
sleep 1

export LD_LIBRARY_PATH=$(find /root/Mittens -name "libmanifoldc.so" -path "*/out/lib/*" 2>/dev/null | head -1 | xargs dirname)
cd /root/Mittens
git pull origin Electron-MRI

./server/target/release/scriptcad-server examples/multiphysics/pure_acoustics.lua > /tmp/server.log 2>&1 &
cd renderer && npm run dev > /tmp/renderer.log 2>&1 &
sleep 4
echo "Services restarted"
EOF
fi

echo "📸 Taking screenshot..."
$SSH_CMD 'bash -s' << 'EOF'
export DISPLAY=:99
cd /root
node -e "
const puppeteer = require('puppeteer');
(async () => {
  const browser = await puppeteer.launch({
    headless: 'new',
    args: ['--no-sandbox', '--disable-setuid-sandbox', '--disable-dev-shm-usage']
  });
  const page = await browser.newPage();
  await page.setViewport({ width: 1920, height: 1080 });
  await page.goto('http://localhost:3000', { waitUntil: 'load', timeout: 15000 });
  await new Promise(r => setTimeout(r, 5000));
  await page.screenshot({ path: '/tmp/vast_screenshot.png' });
  await browser.close();
  console.log('Done');
})();
"
EOF

echo "📥 Downloading..."
scp -o StrictHostKeyChecking=no -P $SSH_PORT root@$SSH_HOST:/tmp/vast_screenshot.png ~/clawd/Mittens/screenshots/$OUTPUT

echo "✅ Saved to ~/clawd/Mittens/screenshots/$OUTPUT"

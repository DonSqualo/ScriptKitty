# Visual Test Agent - Vast.ai GPU Rendering

Instructions for taking screenshots of Mittens/ScriptKitty renders using a Vast.ai GPU instance.

## Prerequisites

- Vast.ai API key stored at `~/.config/vastai/vast_api_key`
- SSH key uploaded to Vast.ai (check with `vastai ssh-keys list`)
- `vastai` CLI installed: `pip3 install --user vastai`

## Quick Start

### 1. Find and Create Instance

```bash
# Set API key
export VAST_API_KEY=$(cat ~/.config/vastai/vast_api_key)

# Find cheapest GPU instance
~/.local/bin/vastai search offers 'gpu_ram>=4 reliability>0.95 dph<0.15 inet_down>100' --order 'dph' --raw | head -3

# Create instance (replace OFFER_ID)
~/.local/bin/vastai create instance OFFER_ID \
  --image nvidia/cuda:12.2.0-runtime-ubuntu22.04 \
  --disk 20 \
  --ssh \
  --raw

# Wait for running status
~/.local/bin/vastai show instance INSTANCE_ID --raw | jq '{ssh_host, ssh_port, actual_status}'
```

### 2. Install Dependencies on Instance

```bash
# SSH connection (adjust port)
ssh -p PORT root@ssh6.vast.ai

# Install everything
apt-get update && apt-get install -y curl wget git xvfb build-essential libclang-dev pkg-config cmake

# Node.js 22
curl -fsSL https://deb.nodesource.com/setup_22.x | bash - && apt-get install -y nodejs

# Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source ~/.cargo/env

# Chrome
wget -q https://dl.google.com/linux/direct/google-chrome-stable_current_amd64.deb
apt-get install -y ./google-chrome-stable_current_amd64.deb

# Puppeteer for proper screenshots
npm install puppeteer
```

### 3. Clone and Build Mittens

```bash
cd /root
git clone https://github.com/DonSqualo/ScriptKitty.git Mittens
cd Mittens
git checkout Electron-MRI

# Build Rust server
cd server && cargo build --release && cd ..

# Install renderer deps
cd renderer && npm install && cd ..
```

### 4. Start Services

```bash
# Start virtual display
Xvfb :99 -screen 0 1920x1080x24 &
export DISPLAY=:99

# Find the manifold library path
MANIFOLD_LIB=$(find /root -name "libmanifoldc.so" -path "*/out/lib/*" 2>/dev/null | head -1 | xargs dirname)
export LD_LIBRARY_PATH=$MANIFOLD_LIB

# Start server (positional arg, not --file!)
cd /root/Mittens
nohup ./server/target/release/scriptcad-server examples/multiphysics/pure_acoustics.lua > /tmp/server.log 2>&1 &

# Start renderer
cd renderer && nohup npm run dev > /tmp/renderer.log 2>&1 &

# Verify
sleep 3
cat /tmp/server.log  # Should show "Server: http://0.0.0.0:3001"
cat /tmp/renderer.log  # Should show "VITE ready"
```

**Ports:**
- Renderer (Vite): `localhost:3000`
- Server (Rust): `localhost:3001`

### 5. Take Screenshot with Puppeteer

Create `/root/screenshot.js`:
```javascript
const puppeteer = require("puppeteer");
(async () => {
  const browser = await puppeteer.launch({
    headless: "new",
    args: ["--no-sandbox", "--disable-setuid-sandbox"]
  });
  const page = await browser.newPage();
  await page.setViewport({ width: 1920, height: 1080 });
  await page.goto("http://localhost:3000", { waitUntil: "networkidle0" });
  // Wait for WebGL content to render (adjust selector as needed)
  await new Promise(r => setTimeout(r, 5000));
  await page.screenshot({ path: "/tmp/screenshot.png" });
  await browser.close();
})();
```

Run: `node /root/screenshot.js`

### 6. Retrieve Screenshot

```bash
# From gateway server
scp -P PORT root@ssh6.vast.ai:/tmp/screenshot.png ~/clawd/Mittens/screenshots/

# Send to user via Telegram
# Use message tool with filePath parameter
```

### 7. Cleanup

```bash
# Destroy instance when done (~$0.04/hr)
export VAST_API_KEY=$(cat ~/.config/vastai/vast_api_key)
~/.local/bin/vastai destroy instance INSTANCE_ID
```

## Troubleshooting

### Server won't start
- Check `LD_LIBRARY_PATH` includes the manifold lib directory
- Use positional argument for lua file, not `--file`

### Screenshot shows "CONNECTING..."
- WebSocket hasn't connected yet
- Increase wait time in puppeteer script
- Check server is running: `curl http://localhost:3001/`

### WebGL not working
- Use `--headless=new` flag (not just `--headless`)
- Ensure Xvfb is running with DISPLAY=:99

## Instance Info Template

When an instance is running, note:
- Instance ID: `30636543`
- SSH: `ssh -p 36542 root@ssh6.vast.ai`
- GPU: GTX 1080 @ ~$0.04/hr
- Ports: renderer=3000, server=3001

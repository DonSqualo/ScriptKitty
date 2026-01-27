# Visual Test Agent - Vast.ai GPU Screenshots

Take WebGL screenshots using a Vast.ai GPU instance (~$0.04/hr).

## Quick Reference

```bash
# Check for running instance
export VAST_API_KEY=$(cat ~/.config/vastai/vast_api_key)
vastai show instances --raw | jq '.[] | select(.actual_status=="running")'

# Take screenshot (if instance already set up)
~/clawd/Mittens/scripts/vast-screenshot.sh
~/clawd/Mittens/scripts/vast-screenshot.sh my_feature.png
~/clawd/Mittens/scripts/vast-screenshot.sh --restart  # for TS/Rust changes
```

## New Instance Setup

### 1. Create Instance

```bash
# Find cheap GPU
vastai search offers 'gpu_ram>=4 reliability>0.95 dph<0.15' --order 'dph'

# Create it
vastai create instance OFFER_ID \
  --image nvidia/cuda:12.2.0-runtime-ubuntu22.04 \
  --disk 20 --ssh

# Wait for "running" status
vastai show instances
```

### 2. Run Setup Script

```bash
~/clawd/Mittens/scripts/vast-setup.sh
```

This installs Node, Rust, Chrome, Puppeteer, clones Mittens, and builds everything. Takes ~5-10 min.

### 3. Start Services

```bash
~/clawd/Mittens/scripts/vast-screenshot.sh --restart
```

## Notes

- **Lua changes**: Auto-reload, no restart needed
- **TS/Rust changes**: Use `--restart` flag
- **Never destroy instances** — let them idle out
- Renderer: `localhost:3000`, Server: `localhost:3001`

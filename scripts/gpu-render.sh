#!/bin/bash
# GPU Render Session - Spin up Vast.ai instance for screenshots
# Usage: ./gpu-render.sh [command]
#   start  - spin up instance
#   ssh    - connect to running instance
#   shot   - take screenshot of localhost:3000
#   stop   - destroy instance
#   status - show running instances

set -e

VASTAI="$HOME/.local/bin/vastai"
INSTANCE_FILE="/tmp/vastai_instance_id"
SSH_KEY="$HOME/.ssh/id_ed25519"

# Ensure API key is set
export VAST_API_KEY=$(cat ~/.secrets/vastai_key 2>/dev/null || cat ~/.config/vastai/vast_api_key 2>/dev/null)

case "${1:-status}" in
  start)
    echo "🔍 Finding cheapest GPU instance..."
    # Find cheapest reliable GPU instance
    OFFER=$($VASTAI search offers 'gpu_ram>=4 reliability>0.95 dph<0.15 inet_down>100' \
      --order 'dph' --raw 2>/dev/null | head -2 | tail -1)
    
    OFFER_ID=$(echo "$OFFER" | jq -r '.id')
    PRICE=$(echo "$OFFER" | jq -r '.dph_total')
    GPU=$(echo "$OFFER" | jq -r '.gpu_name')
    
    echo "📦 Selected: $GPU @ \$$PRICE/hr (ID: $OFFER_ID)"
    echo "🚀 Creating instance..."
    
    # Create instance with Ubuntu + CUDA
    RESULT=$($VASTAI create instance $OFFER_ID \
      --image nvidia/cuda:12.2.0-runtime-ubuntu22.04 \
      --disk 20 \
      --ssh \
      --raw)
    
    INSTANCE_ID=$(echo "$RESULT" | jq -r '.new_contract')
    echo "$INSTANCE_ID" > "$INSTANCE_FILE"
    
    echo "⏳ Waiting for instance $INSTANCE_ID to start..."
    for i in {1..60}; do
      STATUS=$($VASTAI show instance $INSTANCE_ID --raw 2>/dev/null | jq -r '.actual_status' || echo "starting")
      if [ "$STATUS" = "running" ]; then
        echo "✅ Instance running!"
        $VASTAI show instance $INSTANCE_ID
        break
      fi
      sleep 5
      echo -n "."
    done
    ;;
    
  ssh)
    if [ ! -f "$INSTANCE_FILE" ]; then
      echo "❌ No instance running. Run: $0 start"
      exit 1
    fi
    INSTANCE_ID=$(cat "$INSTANCE_FILE")
    
    # Get SSH connection info
    INFO=$($VASTAI show instance $INSTANCE_ID --raw)
    HOST=$(echo "$INFO" | jq -r '.ssh_host')
    PORT=$(echo "$INFO" | jq -r '.ssh_port')
    
    echo "🔗 Connecting to $HOST:$PORT..."
    ssh -o StrictHostKeyChecking=no -p "$PORT" root@"$HOST"
    ;;
    
  stop)
    if [ ! -f "$INSTANCE_FILE" ]; then
      echo "❌ No instance file found"
      exit 1
    fi
    INSTANCE_ID=$(cat "$INSTANCE_FILE")
    
    echo "🛑 Destroying instance $INSTANCE_ID..."
    $VASTAI destroy instance $INSTANCE_ID
    rm -f "$INSTANCE_FILE"
    echo "✅ Instance destroyed"
    ;;
    
  status)
    echo "📊 Running instances:"
    $VASTAI show instances
    ;;
    
  *)
    echo "Usage: $0 {start|ssh|stop|status}"
    exit 1
    ;;
esac

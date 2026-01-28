#!/bin/bash
# Start Ralph in a tmux session for monitoring
# Usage: ./start-ralph.sh [max_iterations]

MAX_ITER="${1:-50}"
SESSION="ralph"

# Kill existing session if any
tmux kill-session -t "$SESSION" 2>/dev/null || true

# Start new session with Ralph
cd "$(dirname "$0")/.."
tmux new-session -d -s "$SESSION" -n "ralph" "bash -c './ralph.sh $MAX_ITER 2>&1 | tee ralph.log; echo \"[Ralph exited with code \$?]\"; sleep infinity'"

echo "Ralph started in tmux session '$SESSION' with max $MAX_ITER iterations"
echo "Monitor: tmux attach -t $SESSION"
echo "Logs: tail -f ~/clawd/Mittens/ralph.log"
echo "Progress: cat ~/clawd/Mittens/progress.txt"

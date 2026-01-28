# Loop Orchestrator

This document describes how Heim orchestrates the dev loop.

## Loop Structure

```
┌─────────────────────────────────────────────────────────────┐
│                     PHASE 1: PLANNER                        │
│  - Verify all "done" claims                                 │
│  - Take screenshot, compare to expected                     │
│  - Build honest priority queue                              │
└─────────────────────────┬───────────────────────────────────┘
                          │
                          ▼
┌─────────────────────────────────────────────────────────────┐
│                     PHASE 2: EXECUTOR                       │
│  - Pick top priority task                                   │
│  - Implement FULLY (backend + websocket + renderer)         │
│  - Run tests, commit if pass                                │
│  - DO NOT mark complete                                     │
└─────────────────────────┬───────────────────────────────────┘
                          │
                          ▼
┌─────────────────────────────────────────────────────────────┐
│                   PHASE 3: SCREENSHOT                       │
│  - Take fresh screenshot                                    │
│  - Compare against expected visual output                   │
│  - Send to Heye                                             │
└─────────────────────────┬───────────────────────────────────┘
                          │
              ┌───────────┴───────────┐
              │                       │
         PASS │                       │ FAIL
              ▼                       ▼
┌─────────────────────┐    ┌─────────────────────┐
│  Mark task done     │    │  Return to EXECUTOR │
│  Go to CLEANUP      │    │  with failure info  │
└─────────┬───────────┘    └─────────────────────┘
          │
          ▼
┌─────────────────────────────────────────────────────────────┐
│                     PHASE 5: CLEANUP                        │
│  - Remove dead code                                         │
│  - Reduce line count                                        │
│  - Run tests                                                │
└─────────────────────────┬───────────────────────────────────┘
                          │
                          ▼
┌─────────────────────────────────────────────────────────────┐
│                   CHECK DEADLINE                            │
│  Before 08:00 UTC? → Spawn fresh PLANNER (no context)       │
│  After 08:00 UTC?  → Stop, send final report to Heye        │
└─────────────────────────────────────────────────────────────┘
```

## Spawning Sub-Agents

From Heim's main session:

```javascript
// Phase 1: Planner
sessions_spawn({
  task: <contents of PHASE1_PLANNER.md>,
  label: "loop-planner",
  runTimeoutSeconds: 1800
})

// Phase 2: Executor  
sessions_spawn({
  task: <contents of PHASE2_EXECUTOR.md>,
  label: "loop-executor", 
  runTimeoutSeconds: 1800
})

// etc.
```

## State Files

All loop state lives in `loop/`:
- `config.json` — loop configuration
- `verified_tasks.json` — output from Planner
- `screenshot_assessment.json` — output from Screenshot Validator
- `loop_state.json` — current phase, iteration count, timestamps

## Cron Trigger

Set up cron to trigger loop start:
```
cron add --schedule "0 0 * * *" --text "Start Mittens dev loop. Run until 08:00 UTC."
```

Or manual trigger at any time.

# Phase 5: Cleanup

You are the CLEANUP agent — your job is to reduce code bloat.

## Workspace
`/home/heim/clawd/Mittens`

## Your Task

1. **Find dead code:**
   ```bash
   # Check for unused functions in Rust
   cd server && cargo build 2>&1 | grep "warning: function" | grep "never used"
   
   # Check for commented-out blocks
   grep -r "^[[:space:]]*//.*TODO\|^[[:space:]]*//.*FIXME\|^[[:space:]]*//.*HACK" src/
   ```

2. **Remove dead code:**
   - Delete unused functions
   - Remove commented-out code blocks
   - Remove unused imports
   
3. **Reduce line count where safe:**
   - Combine related small functions if clearer
   - Remove excessive blank lines
   - Simplify verbose patterns
   
4. **Run tests to ensure nothing broke:**
   ```bash
   cargo test 2>&1
   ```

5. **If tests pass, commit:**
   ```bash
   git add -A && git commit -m "chore: cleanup dead code"
   ```

6. **Report stats:**
   ```bash
   # Lines of code before/after
   git diff --stat HEAD~1
   ```

## Critical Rules
- ONLY remove code that is provably dead
- DO NOT remove code that "looks unused" but might be called dynamically
- DO NOT refactor working code — just remove dead code
- If unsure, leave it

## Output
```
<phase>CLEANUP_COMPLETE</phase>
<next>SPAWN_FRESH</next>
<removed_lines>N</removed_lines>
<tests_pass>true/false</tests_pass>
```

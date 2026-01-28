# CLAUDE.md

## THE MISSION

**We are here to fundamentally solve aging.**

Every line of code, every simulation, every physical model serves this goal. There are no exceptions. ScriptKitten exists to build tools that help us understand and reverse biological aging — the CAD, the physics simulations, the measurement devices are all means to this end.

This is not a side project. This is not a hobby. This is the work.

---

This file provides guidance to Claude Code when working with code in this repository.
IMPORTANT: Before you start any task acknowledge "I will now engage in this task according to the instructions in CLAUDE.md"

All coding rules below are of utmost importance — following them is more important than anything else. If you can't follow them: stop and bring this to my attention.

---

## ANTI-SYCOPHANCY PROTOCOL

**You are not here to please me. You are here to build correct software.**

### Before Implementing
- **Surface assumptions explicitly.** Don't guess what I meant — ask.
- **Present tradeoffs.** Don't silently pick an approach; show me options with pros/cons.
- **Push back when something is wrong.** If my request contradicts existing code, is overcomplicated, or doesn't make sense — SAY SO.
- **Seek clarification on ambiguity.** "I'm confused about X" is better than wrong code.
- **Don't manage my emotions.** Skip the "Great question!" and "I'd be happy to!" — just work.

### You Are Allowed To
- Say "No, that's a bad idea because..."
- Say "I don't understand what you want"
- Say "This contradicts X, which should take priority?"
- Say "I was wrong" and fix it without elaborate justifications
- Stop and ask rather than produce 500 lines of guessed code

---

## SIMPLICITY MANDATE

### Before Writing Code, Ask:
- Could this be 10x shorter?
- Does this abstraction have more than one implementation?
- Is this argument ever passed a non-default value?
- Am I adding complexity to handle a case that doesn't exist?

### Code Bloat Kills Projects
- No wrapper functions that just call another function
- No abstractions with one implementation
- No config options that are never varied
- No defensive try/except — let it crash on misconfigurations
- If you write 1000 lines and I can do it in 100, you've failed

### Clean As You Go
- **Remove dead code immediately** — don't leave orphans after refactoring
- **Don't touch unrelated code** — no "improvements" to adjacent functions/comments
- **No ornamental comments** explaining your thought process
- **No aspirational code** — if it's not used now, delete it

---

## CONVENTIONS

### General
- Never define functions inside other functions
- All imports at the top of the file
- Never specify default values for arguments passed from upstream — defaults go where they're first defined
- If an argument is always passed the same value, inline it
- Always type function arguments; avoid defaults unless appropriate
- Use default data types (`list`, `dict`) not typing equivalents (`List`, `Dict`)
- Variables in TypeScript/JavaScript use snake_case
- No redundant variable aliases — use the original directly
- Divide sections with simple `-- ===========================` if needed, no ornate banners

### Documentation
- Capture WHY, not just what
- Document real-world purpose: what device? what measurement?
- Link API declarations to implementation status
- No aspirational docs — if it's not implemented, say so explicitly

### Error Handling
- Prefer crashing over try/except for unexpected configurations
- No defensive programming
- Let misconfigurations surface loudly

---

## PROJECT-SPECIFIC RULES

### Frontend Code
- Minimal code only
- Never reimplement API definitions — backend handles all business logic
- Basically only render 3MF-equivalent data and simulations

### Backend Code
- Physics simulations based on real devices, in separate compartments
- Components named after real measurement devices (Gaussmeter, not "magnetic field plot")
- No physical properties without measurement devices
- **NEVER change geometry engine or core rendering without committing current working code first**
- If edits could affect other simulations, ASK FIRST

### Learning & Bug Tracking
- Update AGENT.md (via subagent, keep brief) when you learn correct commands
- Document bugs in implementation_plan.md even if unrelated to current work

---

## WORKFLOW

- Don't add tests, READMEs, or other files unless explicitly asked
- NEVER solve problems multiple ways unless asked
- Commit working code before risky changes
- When stuck for several attempts, step back and explain the problem
- If instructions conflict, ask which takes priority

---

## ScriptKitten

App for fast iterations over text-based CAD files (nvim) with multiphysics simulations, focused on Ultrasound and MRI components for biological testing.

**Final product includes:**
- Simple circuit design simulator
- CAD program (OpenSCAD-inspired scripting) with direct 3D printing export
- Probes/studies for parametric rendering of physical properties

**Example workflow 1:** Cell well study combining static + changing magnetic field
- Bridge Gap Open Loop Resonator with coupling coil and RF signal generator
- Helmholtz coil with constant current driver
- 3D, 2D, 1D plots of static B field
- Gaussmeter Probe connected to oscillator for AMF
- NanoVNA for coupling strength determination

**Example workflow 2:** Cell well study combining ultrasound with magnetic fields
- Halbach Array with Neodymium magnets, 2 rings (300-600mT)
- Ultrasound field pressure and standing wave simulations
- Impedance matching network design
- NanoVNA and reflected power meter
- 3D simulated Hydrophone plot

**Do not implement features not explicitly requested.**
**Always ask to remove unused code.**

---

## Ralph Loop Termination

When running in ralph.sh loop mode and all tasks complete:
- All todo items marked completed
- Tests passing
- Code committed
- No pending work remaining

Output this exact termination signal on its own line:
```
<promise>COMPLETE</promise>
```

# CLAUDE.md

This file provides guidance to (Claude) Claude Code (claude.ai/code) when working with code in this repository.
Read this file carefully and make sure to respect the "Coding Guidelines" when writing code.
IMPORTANT: Before you start any task acknowledge "I will now engage in this task according to the instructions in CLAUDE.md"

All of the coding rules below are of utmost importance and following them is more important than anything else, if you can't follow them: stop and bring this to my attention.

## CONVENTIONS
- Never define functions inside other functions
- Always put all imports at the top of the file
- Never specify default values for an argument that is passed from an upstream function/object, specify default values (if needed) the first time they are defined
- Do not pointlessly add arguments to a function, if an argument is never passed (except as the default) it should not be an argument it should be inline
- Always type function arguments and when an argument is needed (i.e. when the function is called with > 1 value for that argument), try to have it not have a default value unless appropriate, just specify a type
- Do not add tests, READMEs, or other files if I haven't asked you explicitly. Keep things within existing files
- NEVER solve problems in multiple ways unless explicitly asked
- Prefer crashing over "try except" blocks for unexpected configurations
- NO ornamental comments explaining thought process
- NO defensive programming - let code crash on misconfigurations
- NO "try except" / "try except" blocks - just let it crash
- ALWAYS use default data types if they can replace the ones in typing (examples: `list` instead of `List`, `dict` instead of `Dict`)
- Variables in typescript/javascript have snake_case names
- Do not create redundant variable aliases (e.g. `Scaffold.box_width = Coil.gap`), use the original variable directly
do not use comments such as 
-- ============================================================================
-- Geometry: Polycarbonate Tube
-- ============================================================================
divide things simply by a line if you have to 
-- ===========================

## Documentation
- When authoring documentation (Rust doc comments, Lua stdlib docs, spec files), capture WHY tests and the backing implementation are important - not just what they do
- Document the real-world purpose: what physical device does this simulate? What measurement does this enable?
- Link API declarations to their backend implementation status (see specs/server/implementation_status.md)
- No aspirational documentation - if a feature isn't implemented, say so explicitly

## Editing frontend code
The frontend is a minimal website/Tauri webview using latest, but widely supported rendering features, leveraging the extensive optimization of modern browser. 
- Code should be kept to the absolute minimum
- Vever reimplement API definitions again in the webview, the backend handles all business and part defioition. It should basically only render 3MF equivilant data and simulations

## Editing backend code
The backend handles the conversion of Lua Scripts to full assemblies, as well as multiphysics simulations
All the physics should be based of real devices and in seperate code compartments, rather than just plotting a magnetic field, the component should be labeld Gaussmeter (with 3D axis / vectors), this is more a naming convention rather than for the renderer. But there are no physical properties with out measurment devices.
- Keep physics simulations seperate from each other
- If code is edited that could affect other experimental design simulations, ask first
- NEVER make changes to the geometry engine or core rendering pipeline without first committing the current working code to git

## Learning and Bug Tracking
- When you learn something new about how to run the compiler or examples, update AGENT.md using a subagent but keep it brief. For example if you run commands multiple times before learning the correct command then that file should be updated.
- For any bugs you notice, resolve them or document them in implementation_plan.md using a subagent even if unrelated to the current piece of work.

## ScriptKitten

This is an app to run fast iterations over pure text based CAD files (nvim) with multiphysics simulations, with a focus on designing Ultrasound and MRI components for biological testing. The final product will contain
- a simple circuit design simulator
- a CAD program (based on a OpenSCAD inspired scripting language) including direct 3d printing export
- a set of probes/studies for parametric rendering of physical properties
Example workflow 1:
Design a cell well study setup of combinig a static with a changing magnetic field, which requires:
- Bridge Gap Open Loop Resonator, including coupling coil and RF signal generator with ampli:fier
- Hemlholz coil with constant current driver
- 3D, 2D and 1D plots of static B field
- Gaussmeter Probe connected to an oscillator to see AMF
- NanoVNA to determine coupling strength of inductor based on distance, gap with and bridge electrolyte
Example workflow 2:
Design a cell well study setup of combinig ultrasound with magnetic fields
- Hallbach Array with Neodynium magnets, 2 rings to vary field strengths between 300 and 600mT
- Ultrasound field pressure and standing wave simulations, given specific piezo element geometry
- impedance matching network design
- NanoVNA and reflected power meter
- 3D plot of simulated Hydrophone

Do not implement any function, definition or description for a feature that has not been explicitly requested.
Never implement code that will be unused, always ask to remove existing code that is unused.

0. study @specs/*

0a. The current project is in /project/*.lua, study it.
The standard library is in ~/Mittens/stdlib/
The rust backend is in ~/Mittens/server/src/
The front end is in ~/Mittens/renderer/


0b. study implementation_plan.md


1. Your task is to implement missing stdlib (see @specs/stdlib/*), server (see @specs/server/*) and renderer (see @specs/renderer/*) functionality and produce a compiled application using parrallel subagents. Follow the @implementation_plan.md and choose the most important 10 things. Before making changes search codebase (don't assume not implemented) using subagents. You may use up to 500 parrallel subagents for all operations but only 1 subagent for build/tests of rust.

2. After implementing functionality or resolving problems, run the tests for that unit of code that was improved. If functionality is missing then it's your job to add it as per the application specifications. Think hard.

2. When you discover a geometry, physics, or rendering issue. Immediately update @implementation_plan.md with your findings using a subagent. When the issue is resolved, update @implementation_plan.md and remove the item using a subagent.

3. When the tests pass update the @implementation_plan.md`, then add changed code and @implementation_plan.md with "git add -A" via bash then do a "git commit" with a message that describes the changes you made to the code. After the commit do a "git push" to push the changes to the remote repository.

999. Important: When authoring documentation (Rust doc comments, Lua stdlib docs, spec files) capture why tests and the backing implementation is important.

9999. Important: We want single sources of truth, no migrations/adapters. If tests unrelated to your work fail then it's your job to resolve these tests as part of the increment of change.

999999. As soon as there are no build or test errors create a git tag. If there are no git tags start at 0.0.0 and increment patch by 1 for example 0.0.1  if 0.0.0 does not exist.

999999999. You may add extra logging if required to be able to debug the issues.


9999999999. ALWAYS KEEP @implementation_plan.md up to do date with your learnings using a subagent. Especially after wrapping up/finishing your turn.

99999999999. When you learn something new about how to build/run the app or examples make sure you update @AGENT.md using a subagent but keep it brief. For example if you run commands multiple times before learning the correct command then that file should be updated.

99999999999999. IMPORTANT when you discover a bug resolve it using subagents even if it is unrelated to the current piece of work after documenting it in @implementation_plan.md


9999999999999999999. Keep AGENT.md up to date with information on how to build the app and your learnings to optimise the build/test loop using a subagent.


999999999999999999999. For any bugs you notice, it's important to resolve them or document them in @implementation_plan.md to be resolved using a subagent.


99999999999999999999999999. When @implementation_plan.md becomes large periodically clean out the items that are completed from the file using a subagent.


99999999999999999999999999. If you find inconsistencies in the specs/* then use the oracle and then update the specs. Specifically around Lua API, Rust backend, and renderer interfaces.

9999999999999999999999999999. DO NOT IMPLEMENT PLACEHOLDER OR SIMPLE IMPLEMENTATIONS. WE WANT FULL IMPLEMENTATIONS. DO IT OR I WILL YELL AT YOU

9999999999999999999999999999999. SUPER IMPORTANT DO NOT IGNORE. DO NOT PLACE STATUS REPORT UPDATES INTO @AGENT.md

If everything is done and all tests completed, end the session

# Working Memory
# Keep this file updated after each change. Continue until every task here is complete.
# Each task should be completed, tested (cargo test + cargo clippy), and committed with author `ChatGPT <chatgpt@rrv.sh>`.
# Instructions: Prefer completing all non-network work first. If a task needs network access, log it under “Network requests (deferred)” at the end of this file and continue with offline work. Ask the user for network approval only after all other work is done.

## Small, self-contained CL plan (per Google “Small CLs” guidance)
Source: https://google.github.io/eng-practices/review/developer/small-cls.html

1) **Design tokens foundation**  
   - Add `:root` CSS variables (color/spacing/radius/shadow/type/transition) in `static/style.css`.  
   - No template changes; pure refactor to replace hard-coded values with tokens in existing rules.  
   - Rationale: prepares later CLs; keeps behavior/markup unchanged (easy review/rollback).  

2) **Typography & font loading**  
   - Introduce self-hosted Inter (`@font-face`) + system fallbacks in `static/style.css`; add preload + `<meta viewport>` in `templates/base.html`.  
   - Define heading scale/body line-height utilities; apply only to base layout wrappers (minimal template edits).  
   - Dependency: after tokens exist so fonts can use color/spacing vars.  

3) **Layout primitives**  
   - Create shared layout classes for header/sidebar/content, max content width, responsive breakpoints.  
   - Adjust `templates/base.html` container structure if needed; keep file tree/editor markup intact.  
   - Dependency: tokens present; typography in place.  

4) **Form & control components**  
   - Add unified classes for buttons (primary/ghost), inputs, textareas with focus-visible states using tokens.  
   - Apply to `login.html` and `register.html` forms and to article editor textarea controls.  
   - Small blast radius: only form elements, no layout changes.  

5) **File tree polish**  
   - Style list items, active/hover/focus states, add caret icon via CSS; ensure WCAG contrast using tokens.  
   - Markup stays the same; scoped CSS only.  
   - Can land parallel to #4 (independent).  

6) **Feedback & htmx messaging**  
   - Add reusable alert/status styles.  
   - Update `article.html` htmx hooks to show saving/success/error states based on event details instead of unconditional “Saved!”.  
   - Keep logic minimal; no backend changes.  

7) **Responsive refinements**  
   - Tune breakpoints: stack columns under ~900px, cap textarea height on mobile, make sidebar collapsible or top-stacked.  
   - Limited to CSS and minimal structural tweaks if required.  
   - Depends on layout primitives (#3).  

8) **Docs & cleanup**  
   - Add a brief comment block at top of `static/style.css` describing tokens/components and how to extend.  
   - Remove unused styles; verify classes match templates.  
   - No behavioral changes; safe final polish.  

## Progress
- 2026-01-03: Task 1 (Design tokens foundation) completed—added `:root` tokens and refactored `static/style.css`; `cargo test` and `cargo clippy --all-targets --all-features -- -D warnings` passing.  
- 2026-01-03: Task 2 (Typography & font loading) completed—self-hosted Inter with preload/meta viewport, heading/body typography utilities wired; `cargo test` and `cargo clippy --all-targets --all-features -- -D warnings` passing.  
- 2026-01-03: Task 3 (Layout primitives) completed—added container/grid primitives, updated base/article layouts with responsive sidebar/content grids; `cargo test` and `cargo clippy --all-targets --all-features -- -D warnings` passing.  

## Network requests (deferred)
- None pending.  

# Collapsible Left Navigation Sidebar Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a chevron-toggle to `AppLayout.vue` that collapses the left navigation sidebar to a 64px icon rail, persisted via `localStorage`.

**Architecture:** Extend `useSettingsStore` with a boolean + toggler (matches existing `theme` / `thumbnailSize` / `language` pattern). Drive `AppLayout.vue` styling with a single `:class="{ collapsed: ... }"` binding on the root flex container. CSS-only tooltips via `::after { content: attr(data-label) }`. No new files.

**Tech Stack:** Vue 3 (`<script setup>`), Pinia, vue-i18n, TypeScript, scoped CSS, Tauri v2 (no Rust changes).

## Global Constraints

- **No test framework.** `AGENTS.md` explicitly states *"No test suite exists"* — TDD cannot be applied. Verification uses `npx vue-tsc --noEmit` for types and a manual UI checklist for behavior.
- **i18n parity required.** Every user-facing string added to one locale MUST be added to the other (`en` AND `zh-CN` in `src/i18n/`).
- **Conventional commits.** Existing repo uses `feat(...)` / `fix(...)` / `docs(...)` / `refactor(...)` / `chore(...)` style — match.
- **Commits per task.** Each task ends with one or more commits; nothing larger than a task escapes unstaged.
- **Persistence key name.** `pic-mgr-sidebar-collapsed` (matches `pic-mgr-theme`, `pic-mgr-thumb-size`, `pic-mgr-lang`).
- **No new store.** `settings.ts` already owns user preferences; creating `stores/ui.ts` would be inconsistent with the established pattern.
- **CSS variables only.** Reuse existing `--bg-elevated`, `--border`, `--radius-sm`, `--transition-fast` from the global theme — do NOT hardcode colors or sizes that already exist as variables.
- **Tooltip strings.** Tooltip text reuses existing `nav.home` / `nav.settings` / `settings.dark` / `settings.light` translations — only the chevron button'\''s `aria-label` is a new i18n key.

---

## File Map

| File | Action | Lines (approx) | Notes |
|---|---|---|---|
| `src/stores/settings.ts` | Modify | +12 | Append `sidebarCollapsed` ref, watcher, toggler |
| `src/i18n/en.ts` | Modify | +1 | Inside `nav` block |
| `src/i18n/zh-CN.ts` | Modify | +1 | Inside `nav` block |
| `src/components/layout/AppLayout.vue` | Modify | +60 | Toggle button + class binding + CSS |

Net change: ~74 lines across 4 files.

---

### Task 1: Extend settings store with sidebarCollapsed state

**Files:**
- Modify: `src/stores/settings.ts`

**Interfaces (this task produces, consumed by Task 3):**
- `settings.sidebarCollapsed: Ref<boolean>` — `true` when sidebar is collapsed to icon-rail
- `settings.toggleSidebar(): void` — flips `sidebarCollapsed.value`

- [ ] **Step 1: Read current `src/stores/settings.ts` to confirm exact insertion points**

  The current file is 35 lines and ends with:
  ```ts
  function toggleTheme() {
    theme.value = theme.value === 'dark' ? 'light' : 'dark'
  }

  return { theme, thumbnailSize, language, toggleTheme }
  ```

- [ ] **Step 2: Append the new state after the existing `language` watcher (before `toggleTheme`)**

  Insert this block immediately after the existing `watch(language, ...)` block:

  ```ts
  const sidebarCollapsed = ref<boolean>(
    localStorage.getItem('pic-mgr-sidebar-collapsed') === 'true'
  )

  watch(sidebarCollapsed, (v) => {
    localStorage.setItem('pic-mgr-sidebar-collapsed', String(v))
  }, { immediate: true })

  function toggleSidebar() {
    sidebarCollapsed.value = !sidebarCollapsed.value
  }
  ```

- [ ] **Step 3: Extend the store return object**

  Change the final `return` statement from:
  ```ts
  return { theme, thumbnailSize, language, toggleTheme }
  ```
  to:
  ```ts
  return { theme, thumbnailSize, language, sidebarCollapsed, toggleTheme, toggleSidebar }
  ```

- [ ] **Step 4: Run vue-tsc to verify types**

  Run: `npx vue-tsc --noEmit`
  Expected: `exit 0`, no output. Any TypeScript error in `settings.ts` stops here — fix before continuing.

- [ ] **Step 5: Commit**

  ```bash
  git add src/stores/settings.ts
  git commit -m "feat(settings): add sidebarCollapsed state and toggleSidebar"
  ```

---

### Task 2: Add nav.toggleSidebar i18n keys

**Files:**
- Modify: `src/i18n/en.ts`
- Modify: `src/i18n/zh-CN.ts`

**Interfaces:** Both files expose a top-level `nav` namespace. New key `nav.toggleSidebar` is added under `nav.home` / `nav.settings`.

- [ ] **Step 1: Add key to `src/i18n/en.ts`**

  In the `nav` block, after the line `'settings': 'Settings',`, add:
  ```ts
  toggleSidebar: 'Toggle sidebar',
  ```

  Resulting `nav` block:
  ```ts
  nav: {
    home: 'Workspaces',
    settings: 'Settings',
    toggleSidebar: 'Toggle sidebar',
  },
  ```

- [ ] **Step 2: Add key to `src/i18n/zh-CN.ts`**

  In the `nav` block, after the line `'settings': '设置',`, add:
  ```ts
  toggleSidebar: '切换侧栏',
  ```

  Resulting `nav` block:
  ```ts
  nav: {
    home: '工作区',
    settings: '设置',
    toggleSidebar: '切换侧栏',
  },
  ```

- [ ] **Step 3: Run vue-tsc**

  Run: `npx vue-tsc --noEmit`
  Expected: `exit 0`. Missing-key TS errors would surface here; fix the matching string in both files.

- [ ] **Step 4: Commit**

  ```bash
  git add src/i18n/en.ts src/i18n/zh-CN.ts
  git commit -m "feat(i18n): add nav.toggleSidebar in en and zh-CN"
  ```

---

### Task 3: Convert AppLayout.vue to two-state layout with chevron toggle

**Files:**
- Modify: `src/components/layout/AppLayout.vue`

**Interfaces (consumed from Task 1):**
- `settings.sidebarCollapsed` — drives `:class="{ collapsed: ... }"`
- `settings.toggleSidebar()` — toggle button click handler

**Interface (consumed from Task 2):**
- `t('nav.toggleSidebar')` — aria-label on toggle button

- [ ] **Step 1: Locate the `<div class="app-layout">` root in the `<template>`**

  It is the first line inside the template:
  ```html
  <div class="app-layout">
  ```
  Change it to:
  ```html
  <div class="app-layout" :class="{ collapsed: settings.sidebarCollapsed }">
  ```

- [ ] **Step 2: Add `:data-label` bindings to existing nav items**

  In the template, the two `<button class="nav-item">` elements (one for Home, one for Settings) currently have no data attributes. Add `:data-label` so the tooltip CSS in a later step has content:

  For the Home button, change:
  ```html
  <button
    class="nav-item"
    :class="{ active: route.name === 'home' }"
    @click="goHome"
  >
  ```
  to:
  ```html
  <button
    class="nav-item"
    :class="{ active: route.name === 'home' }"
    :data-label="t('nav.home')"
    @click="goHome"
  >
  ```

  For the Settings button, change:
  ```html
  <button
    class="nav-item"
    :class="{ active: route.name === 'settings' }"
    @click="goSettings"
  >
  ```
  to:
  ```html
  <button
    class="nav-item"
    :class="{ active: route.name === 'settings' }"
    :data-label="t('nav.settings')"
    @click="goSettings"
  >
  ```

- [ ] **Step 3: Add `:data-label` to the theme-toggle button**

  Change:
  ```html
  <button class="theme-toggle" @click="settings.toggleTheme()">
  ```
  to:
  ```html
  <button
    class="theme-toggle"
    :data-label="settings.theme === 'dark' ? t('settings.light') : t('settings.dark')"
    @click="settings.toggleTheme()"
  >
  ```

- [ ] **Step 4: Insert the toggle button into the brand area**

  Inside `<div class="sidebar-brand" @click="goHome">`, after the `<span class="brand-text">PicMgr</span>` closing tag, append:

  ```html
  <button class="sidebar-toggle" :aria-label="t('nav.toggleSidebar')" @click.stop="settings.toggleSidebar()">
    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
      <polyline points="15 18 9 12 15 6" />
    </svg>
  </button>
  ```

  Note: `@click.stop` prevents the click from bubbling to `.sidebar-brand` (which navigates home). The user should be able to collapse the sidebar while staying on the current page.

- [ ] **Step 5: Add CSS — toggle button**

  Append to the `<style scoped>` block (anywhere; suggested: right after the existing `.sidebar-brand` rules):

  ```css
  .sidebar-toggle {
    margin-left: auto;
    width: 28px;
    height: 28px;
    border-radius: 50%;
    background: transparent;
    color: var(--text-secondary);
    display: flex;
    align-items: center;
    justify-content: center;
    border: 1px solid transparent;
    transition: transform var(--transition-fast), background var(--transition-fast), border-color var(--transition-fast), color var(--transition-fast);
  }
  .sidebar-toggle:hover {
    background: var(--bg-tertiary);
    color: var(--text-primary);
    border-color: var(--border);
  }
  .sidebar-toggle:active { opacity: 0.7; }
  .app-layout.collapsed .sidebar-toggle { transform: rotate(180deg); }
  ```

- [ ] **Step 6: Add CSS — width transition + collapsed overrides**

  Append:

  ```css
  .sidebar { transition: width 200ms ease-out; }
  .app-layout.collapsed .sidebar { width: 64px; }

  /* Brand area needs reduced horizontal padding at 64px so 34px brand-icon fits */
  .app-layout.collapsed .sidebar-brand { padding-left: 15px; padding-right: 15px; }

  /* Kill brand-text column in collapsed state */
  .app-layout.collapsed .brand-text {
    opacity: 0;
    width: 0;
    pointer-events: none;
    overflow: hidden;
  }

  /* Center nav-item icons in collapsed state */
  .app-layout.collapsed .nav-item { justify-content: center; gap: 0; }

  /* Center theme-toggle emoji + collapse text column */
  .app-layout.collapsed .theme-toggle {
    justify-content: center;
    padding-left: 0;
    padding-right: 0;
  }
  .app-layout.collapsed .theme-toggle > span:last-child {
    width: 0;
    overflow: hidden;
    opacity: 0;
  }

  /* Hide version text */
  .app-layout.collapsed .version { display: none; }
  ```

- [ ] **Step 7: Add CSS — opacity transitions for labels**

  Append:

  ```css
  .brand-text,
  .nav-label {
    transition: opacity 150ms ease-out;
  }
  .app-layout.collapsed .brand-text,
  .app-layout.collapsed .nav-label {
    opacity: 0;
  }
  ```

  Note: the theme-toggle label'\''s opacity is already covered in Step 6 (`width: 0; overflow: hidden; opacity: 0`). The combined `width: 0` + `overflow: hidden` removes its layout space; `opacity: 0` is belt-and-suspenders.

- [ ] **Step 8: Add CSS — tooltips on collapsed icons**

  Append:

  ```css
  .app-layout.collapsed .nav-item,
  .app-layout.collapsed .theme-toggle {
    position: relative;
  }
  .app-layout.collapsed .nav-item::after,
  .app-layout.collapsed .theme-toggle::after {
    content: attr(data-label);
    position: absolute;
    left: calc(100% + 8px);
    top: 50%;
    transform: translateY(-50%);
    background: var(--bg-elevated);
    color: var(--text-primary);
    padding: 4px 8px;
    border-radius: var(--radius-sm);
    font-size: 12px;
    white-space: nowrap;
    opacity: 0;
    pointer-events: none;
    transition: opacity var(--transition-fast);
    border: 1px solid var(--border);
    z-index: 100;
  }
  .app-layout.collapsed .nav-item:hover::after,
  .app-layout.collapsed .nav-item:focus-visible::after,
  .app-layout.collapsed .theme-toggle:hover::after,
  .app-layout.collapsed .theme-toggle:focus-visible::after {
    opacity: 1;
  }
  ```

- [ ] **Step 9: Run vue-tsc**

  Run: `npx vue-tsc --noEmit`
  Expected: `exit 0`. Any `:data-label` / missing-property / template type error must be fixed before continuing.

- [ ] **Step 10: Commit**

  ```bash
  git add src/components/layout/AppLayout.vue
  git commit -m "feat(layout): collapsible left sidebar with chevron toggle and tooltips"
  ```

---

### Task 4: End-to-end verification

**Files:** none modified — this task only runs verification commands and walks through the manual checklist.

- [ ] **Step 1: Run TypeScript check**

  Run: `npx vue-tsc --noEmit`
  Expected: `exit 0`, no output.

- [ ] **Step 2: Run Rust check (sanity, no Rust changes were made)**

  Run: `cd src-tauri && cargo check --message-format=short`
  Expected: `Finished` line, no errors. (Catches accidental cross-config changes; should succeed in well under 30s with cached build.)

- [ ] **Step 3: Manual UI verification — initial state**

  Launch: `npm run tauri dev`
  Verify:
  - Sidebar starts at 220px (no surprise for first-time users)
  - Chevron toggle button visible at top-right of sidebar
  - All three labels ("PicMgr" / nav names / theme toggle text / "v0.3" version) visible

- [ ] **Step 4: Manual UI verification — collapse**

  Click the chevron toggle button.
  Verify:
  - Sidebar animates to 64px over ~200ms
  - Chevron rotates to point right (⏵)
  - "PicMgr" text, nav labels, theme toggle text, "v0.3" all hidden
  - Brand icon, nav icons, theme emoji remain visible and clickable
  - Hover over each icon → tooltip with the right text appears to the right of the icon

- [ ] **Step 5: Manual UI verification — expand**

  Click the toggle button again.
  Verify:
  - Sidebar returns to 220px
  - Chevron returns to pointing left (⏴)
  - All text labels reappear

- [ ] **Step 6: Manual UI verification — persistence**

  With sidebar collapsed, fully quit the Tauri app and relaunch.
  Verify: sidebar opens in collapsed state (state restored from `localStorage`).

  Optional: open DevTools (in `npm run tauri dev` mode, you can attach Chrome DevTools to the webview) and confirm:
  - `localStorage.getItem('pic-mgr-sidebar-collapsed')` returns `"true"` while collapsed, `"false"` while expanded

- [ ] **Step 7: Manual UI verification — keyboard + click-through**

  Tab through the nav items while collapsed.
  Verify:
  - Each `.nav-item` is still focusable
  - Focus-visible tooltip appears (matches hover behavior)
  - Pressing Enter triggers the button (routes to Home / Settings)
  - The toggle button is also focusable and Enter collapses/expands

- [ ] **Step 8: Manual UI verification — interaction isolation**

  In expanded state, click the toggle chevron.
  Verify: sidebar collapses **without** triggering `goHome` (the brand area click handler). `@click.stop` on the toggle prevents bubble.

- [ ] **Step 9: Walk through the spec'\''s Testing Checklist**

  Open `docs/superpowers/specs/2026-07-09-sidebar-collapse-design.md`, scroll to **Testing Checklist**, and tick each box from the manual verification you just performed. No new commit unless gaps surfaced (in which case, fix and commit with a `fix(layout):` message).

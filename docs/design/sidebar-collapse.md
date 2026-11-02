# Design: Collapsible Left Navigation Sidebar

**Date:** 2026-07-09
**Status:** Draft
**Approach:** Extend `AppLayout.vue` with a chevron toggle, persist state via `settings` store (Icon-rail mode, Approach A)

## Goal

Allow the user to collapse the left navigation sidebar to a 64px icon-rail, freeing ~156px of horizontal space for the thumbnail grid, while keeping the toggle affordance in a fixed, discoverable location. State persists across app restarts.

## Current Behavior

- `AppLayout.vue` renders an `<aside class="sidebar">` hard-coded to `width: 220px`.
- Sidebar contains: brand area (icon + "PicMgr" text), two `.nav-item` buttons (Home, Settings), and footer (theme toggle + version text).
- No mechanism to collapse or resize.

`SidebarPanel.vue` (`src/components/browser/SidebarPanel.vue`) is a *different* sidebar — the in-BrowserView filter panel with SearchBar/TagFilter. **Out of scope** for this change.

## Proposed Behavior

- Two states driven by `settings.sidebarCollapsed` (boolean):
  - `expanded` (default) → 220px, full labels and text
  - `collapsed` → 64px, icons only, labels via tooltip on hover (64px chosen so existing `.nav-item` padding fits the 18px icon without overflow; see layout notes below)
- A single toggle button anchored to the top-right corner of the sidebar, **in both states**. Reachable without hunting.
- 200ms `width` transition (ease-out), with text label fade via `opacity` (no layout reflow on the labels themselves).
- State persists via `localStorage` key `pic-mgr-sidebar-collapsed`.
- Initial app load reads `false` by default — does not surprise existing users with a sudden collapsed sidebar on next launch.

---

## Store: Extend `useSettingsStore`

**File:** `src/stores/settings.ts`

Add:

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

Return `sidebarCollapsed` and `toggleSidebar` from the store.

**Why not a new `stores/ui.ts`?** `settings.ts` already holds all user preferences (`theme`, `thumbnailSize`, `language`) with the same `localStorage + watch()` pattern. Splitting UI prefs into a separate store would be a YAGNI fork that costs consistency without buying anything.

---

## Frontend: Two-State Layout

**File:** `src/components/layout/AppLayout.vue`

### Top-level wrapper

Add `:class="{ collapsed: settings.sidebarCollapsed }"` to `.app-layout` (the existing root flex container). This drives all conditional styling.

```html
<div class="app-layout" :class="{ collapsed: settings.sidebarCollapsed }">
```

### Toggle button

Inserted inside the `.sidebar-brand` row, after `.brand-text`:

```html
<button class="sidebar-toggle" :aria-label="t('nav.toggleSidebar')"
        @click="settings.toggleSidebar()">
  <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor"
       stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
    <polyline points="15 18 9 12 15 6" />
  </svg>
</button>
```

- Chevron points **left** when expanded (⏴ — "I can fold this away"), **right** when collapsed (⏵ — "I can bring this back").
- Cheaper than swapping two SVGs: rotate the same chevron via CSS:

```css
.sidebar-toggle { transition: transform var(--transition-fast); }
.app-layout.collapsed .sidebar-toggle { transform: rotate(180deg); }
```

### Collapsed-state UI adaptations

| Element | expanded | collapsed |
|---|---|---|
| `.sidebar` width | 220px | 64px |
| `.brand-text` ("PicMgr") | visible (`opacity: 1`, normal width) | `opacity: 0; width: 0; pointer-events: none` (no reflow flicker) |
| `.nav-item` | left-aligned icon + label | icon centered (`justify-content: center`); label hidden |
| `.nav-label` | visible | `opacity: 0` + tooltip via `::after` on `.nav-item` |
| `.theme-toggle` | left-aligned emoji + text | centered emoji only; text + tooltip |
| `.version` ("v0.3") | visible | `display: none` |

### Tooltips (CSS-only, no third-party library)

Inline `::after` pseudo-elements on `.nav-item` and `.theme-toggle`:

```css
.app-layout.collapsed .nav-item { position: relative; }
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

Add `:data-label="..."` (Vue prop) to each `.nav-item` (using `nav.home` / `nav.settings` translations) and to `.theme-toggle` (using `settings.theme === 'dark' ? settings.light : settings.dark` translation). Reuses existing translation strings — no new i18n entries needed for tooltips. Reuse the existing `--bg-elevated`, `--border`, `--radius-sm`, `--transition-fast` CSS variables.

### Animation

```css
.sidebar {
  width: 220px;
  transition: width 200ms ease-out;
}
.app-layout.collapsed .sidebar { width: 64px; }

/* Brand area needs reduced horizontal padding at 64px so the 34px brand-icon fits */
.app-layout.collapsed .sidebar-brand { padding-left: 15px; padding-right: 15px; }

/* Center the theme-toggle emoji in collapsed state, kill the text column entirely */
.app-layout.collapsed .theme-toggle { justify-content: center; padding-left: 0; padding-right: 0; }
.app-layout.collapsed .theme-toggle > span:last-child { width: 0; overflow: hidden; opacity: 0; }

.brand-text,
.nav-label,
.theme-toggle > span:last-child {
  transition: opacity 150ms ease-out;
}
.app-layout.collapsed .brand-text,
.app-layout.collapsed .nav-label,
.app-layout.collapsed .theme-toggle > span:last-child {
  opacity: 0;
}
```

---

## i18n

**Files:** `src/i18n/en.ts`, `src/i18n/zh-CN.ts`

Add under the existing `nav` namespace:

```ts
// en
'nav.toggleSidebar': 'Toggle sidebar',

// zh-CN
'nav.toggleSidebar': '切换侧栏',
```

### Existing dead strings

`browser.collapseSidebar` / `browser.expandSidebar` already exist in both language files but are unused. **Leave them alone** — clearing dead keys risks breaking whatever WIP uses them, and they don'\''t conflict with `nav.toggleSidebar`.

---

## Edge Cases

1. **First launch** → `sidebarCollapsed` defaults to `false` (expanded) so existing layouts aren'\''t disrupted on upgrade.
2. **Window resize while collapsed** → 64px is independent of window width; main content gains the freed space because `.main-content { flex: 1 }`.
3. **Focused element hidden by collapse** → if a `.nav-item` had keyboard focus when the user collapses, the tooltip `::after` only appears on `:focus-visible`, so focus stays inside the (still-tabbable) button; tab order unchanged.
4. **Brand area click while collapsed** → `goHome` still fires; the brand icon remains a 34×34 clickable target.
5. **Theme toggle while collapsed** → tooltip shows current state label ("暗色" / "亮色" — same string that appears in expanded mode).
6. **Reduced motion** → Respect `prefers-reduced-motion: reduce` by collapsing the `transition` durations to 0ms. Out of scope for v1 unless trivial — keep `200ms` for now and document as future polish.

---

## Out of Scope (YAGNI)

- Keyboard shortcut (`Ctrl+B`) — user didn'\''t request it.
- Hover-to-expand auto behavior — explicitly excluded by Approach A.
- Separating UI prefs into a new `stores/ui.ts`.
- Animating individual elements (chevron rotate aside from width/opacity).

---

## Testing Checklist

- [ ] `npm run tauri dev` — sidebar starts expanded (220px) on a clean install (no localStorage entry)
- [ ] Click toggle button → sidebar animates to 64px; chevron rotates to ⏵
- [ ] All labels hidden in collapsed state (brand text, nav labels, theme toggle label, version)
- [ ] Tooltips appear on hover over each icon in collapsed state
- [ ] Brand icon, nav icons, theme emoji remain clickable and functional in collapsed state
- [ ] Click toggle again → sidebar returns to 220px; chevron rotates back to ⏴
- [ ] Close and reopen app → state persists (last value reloaded from localStorage)
- [ ] Manual localStorage manipulation (`pic-mgr-sidebar-collapsed`) takes effect on reload
- [ ] Thumbnail grid gains horizontal space (column count increases naturally)
- [ ] `npx vue-tsc --noEmit` passes
- [ ] `cd src-tauri && cargo check` passes

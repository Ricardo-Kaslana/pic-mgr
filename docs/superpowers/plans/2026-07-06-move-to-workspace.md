# Move to Workspace Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Allow users to move images between workspaces via the context menu with proper metadata reassignment.

**Architecture:** Extend existing `move_images` backend command with optional `dest_workspace_id` parameter. Add workspace picker dialog component. Integrate into ThumbnailGrid context menu.

**Tech Stack:** Rust (rusqlite), Vue 3 (Composition API), Tauri v2, vue-i18n

## Global Constraints

- Never modify original image files — metadata only in SQLite
- FTS5 triggers use rowid-only delete: `INSERT INTO images_fts(images_fts, rowid) VALUES('delete', old.id)`
- DB migrations via `ALTER TABLE ADD COLUMN` with `.ok()` — no migration framework
- Workspace deletion: manual cascade in transaction (image_tags → images → workspace)
- Thumbnail generation wraps `image::open()` in `catch_unwind`
- Append-mode pagination (`append=true`): new pages added to existing data
- File dates: `created_at` = mtime, `file_created_at` = birth time, `indexed_at` = DB add time
- Delete uses `trash` crate (system recycle bin), not `fs::remove_file`
- All user-facing strings through vue-i18n (both zh-CN and en)

---

## File Structure

| File | Action | Responsibility |
|------|--------|---------------|
| `src-tauri/src/commands/metadata.rs` | Modify | Extend `move_images` to accept `dest_workspace_id` |
| `src/components/workspace/MoveToWorkspaceDialog.vue` | Create | Workspace picker modal dialog |
| `src/components/browser/ThumbnailGrid.vue` | Modify | Add context menu item + handler |
| `src/i18n/zh-CN.ts` | Modify | Add Chinese translations |
| `src/i18n/en.ts` | Modify | Add English translations |

---

### Task 1: Extend `move_images` backend command

**Files:**
- Modify: `src-tauri/src/commands/metadata.rs:132-265`

**Interfaces:**
- Consumes: existing `move_images` function, `get_workspace` query
- Produces: updated `move_images` command with `dest_workspace_id` parameter

- [ ] **Step 1: Add `dest_workspace_id` parameter to command signature**

```rust
#[tauri::command]
pub fn move_images(
    state: State<AppState>,
    thumb_state: State<super::thumbnail::ThumbnailState>,
    ids: Vec<i64>,
    dest_dir: Option<String>,
    dest_workspace_id: Option<i64>,
) -> Result<MoveResult, String> {
```

- [ ] **Step 2: Add workspace resolution logic at start of function**

After the empty check and before the `let dest = PathBuf::from(...)` line, add:

```rust
    // Resolve destination from workspace ID if provided
    let dest_dir_resolved = if let Some(ws_id) = dest_workspace_id {
        let conn = state.db.conn.lock().map_err(|e| e.to_string())?;
        let workspace = crate::db::queries::get_workspace(&conn, ws_id)
            .map_err(|e| e.to_string())?
            .ok_or_else(|| format!("Workspace {} not found", ws_id))?;
        drop(conn);
        workspace.path
    } else {
        dest_dir.ok_or_else(|| "Either dest_dir or dest_workspace_id must be provided".to_string())?
    };
```

- [ ] **Step 3: Replace `dest_dir` usage with `dest_dir_resolved`**

Change:
```rust
    let dest = PathBuf::from(&dest_dir);
```
To:
```rust
    let dest = PathBuf::from(&dest_dir_resolved);
```

- [ ] **Step 4: Add `target_workspace_id` tracking variable**

After the `let mut moved: Vec<MovedInfo> = Vec::new();` line, add:

```rust
    let target_workspace_id = dest_workspace_id;
```

- [ ] **Step 5: Update DB logic to handle cross-workspace moves**

Replace the DB update block (after `if !moved.is_empty()`) with:

```rust
    if !moved.is_empty() {
        // Clear stale thumbnails
        let cache = thumb_state.cache.lock().map_err(|e| e.to_string())?;
        let moved_ids: Vec<i64> = moved.iter().map(|m| m.id).collect();
        delete_thumbnail_files(&cache, &moved_ids);
        drop(cache);

        let conn = state.db.conn.lock().map_err(|e| e.to_string())?;
        conn.execute_batch("BEGIN").map_err(|e| e.to_string())?;

        for m in &moved {
            if let Some(target_ws_id) = target_workspace_id {
                // Cross-workspace move: update workspace_id + relative_path + file_name
                let new_relative = m.dest_file
                    .strip_prefix(&PathBuf::from(&dest_dir_resolved))
                    .map(|r| r.to_string_lossy().replace('\\', "/"))
                    .unwrap_or_else(|_| m.new_file_name.clone());

                conn.execute(
                    "UPDATE images SET workspace_id = ?1, relative_path = ?2, file_name = ?3, updated_at = datetime('now') WHERE id = ?4",
                    rusqlite::params![target_ws_id, new_relative, m.new_file_name, m.id],
                ).map_err(|e| e.to_string())?;
            } else {
                // Same-workspace or arbitrary directory move
                let ws_path = PathBuf::from(&m.workspace_path);
                let new_relative = if let Ok(rel) = m.dest_file.strip_prefix(&ws_path) {
                    Some(rel.to_string_lossy().replace('\\', "/"))
                } else {
                    None
                };

                if let Some(rel_path) = new_relative {
                    conn.execute(
                        "UPDATE images SET relative_path = ?1, file_name = ?2, updated_at = datetime('now') WHERE id = ?3",
                        rusqlite::params![rel_path, m.new_file_name, m.id],
                    ).map_err(|e| e.to_string())?;
                } else {
                    delete_db_records(&conn, &[m.id])?;
                }
            }
        }

        conn.execute_batch("COMMIT").map_err(|e| e.to_string())?;
    }
```

- [ ] **Step 6: Verify Rust compiles**

Run: `cd src-tauri && cargo check`
Expected: Compiles with no errors

- [ ] **Step 7: Commit**

```bash
git add src-tauri/src/commands/metadata.rs
git commit -m "feat: extend move_images to support cross-workspace moves"
```

---

### Task 2: Add i18n keys

**Files:**
- Modify: `src/i18n/zh-CN.ts`
- Modify: `src/i18n/en.ts`

**Interfaces:**
- Consumes: none
- Produces: translation keys for workspace picker UI

- [ ] **Step 1: Add Chinese translations**

In `src/i18n/zh-CN.ts`, add to the `context` section:

```typescript
'context.moveToWorkspace': '移动到工作区...',
```

Add new section:

```typescript
'moveToWorkspace': {
  title: '选择目标工作区',
  confirm: '移动',
  currentWorkspace: '当前工作区',
},
```

- [ ] **Step 2: Add English translations**

In `src/i18n/en.ts`, add to the `context` section:

```typescript
'context.moveToWorkspace': 'Move to workspace...',
```

Add new section:

```typescript
'moveToWorkspace': {
  title: 'Select target workspace',
  confirm: 'Move',
  currentWorkspace: 'Current workspace',
},
```

- [ ] **Step 3: Commit**

```bash
git add src/i18n/zh-CN.ts src/i18n/en.ts
git commit -m "feat: add i18n keys for move-to-workspace dialog"
```

---

### Task 3: Create workspace picker dialog

**Files:**
- Create: `src/components/workspace/MoveToWorkspaceDialog.vue`

**Interfaces:**
- Consumes: `useWorkspaceStore()`, i18n keys from Task 2
- Produces: `MoveToWorkspaceDialog` component with `select` and `close` events

- [ ] **Step 1: Create component file**

```vue
<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { useWorkspaceStore } from '../../stores/workspace'

const props = defineProps<{ currentWorkspaceId: number }>()
const emit = defineEmits<{
  select: [workspaceId: number]
  close: []
}>()

const { t } = useI18n()
const workspaceStore = useWorkspaceStore()

const workspaces = computed(() =>
  workspaceStore.workspaces.filter(ws => ws.id !== props.currentWorkspaceId)
)

function handleSelect(wsId: number) {
  emit('select', wsId)
}
</script>

<template>
  <div class="dialog-overlay" @click.self="emit('close')">
    <div class="dialog">
      <div class="dialog-header">
        <h3>{{ t('moveToWorkspace.title') }}</h3>
        <button class="close-btn" @click="emit('close')">×</button>
      </div>
      <div class="dialog-body">
        <div
          v-for="ws in workspaces"
          :key="ws.id"
          class="workspace-item"
          @click="handleSelect(ws.id)"
        >
          <div class="ws-name">{{ ws.name }}</div>
          <div class="ws-path">{{ ws.path }}</div>
        </div>
        <div v-if="workspaces.length === 0" class="empty">
          {{ t('moveToWorkspace.noWorkspaces') }}
        </div>
      </div>
      <div class="dialog-footer">
        <button class="secondary" @click="emit('close')">{{ t('common.cancel') }}</button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.dialog-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
  backdrop-filter: blur(4px);
}
.dialog {
  background: var(--bg-secondary);
  border-radius: var(--radius-lg);
  border: 1px solid var(--border);
  width: 400px;
  max-height: 500px;
  display: flex;
  flex-direction: column;
  box-shadow: 0 20px 60px rgba(0, 0, 0, 0.3);
}
.dialog-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px 20px;
  border-bottom: 1px solid var(--border-light);
}
.dialog-header h3 {
  font-size: 15px;
  font-weight: 650;
  margin: 0;
}
.close-btn {
  background: none;
  border: none;
  font-size: 18px;
  color: var(--text-muted);
  cursor: pointer;
  padding: 4px 8px;
  border-radius: var(--radius-xs);
  transition: all var(--transition-fast);
}
.close-btn:hover {
  background: var(--bg-tertiary);
  color: var(--text-primary);
}
.dialog-body {
  flex: 1;
  overflow-y: auto;
  padding: 12px 20px;
}
.workspace-item {
  padding: 12px 16px;
  border-radius: var(--radius-md);
  cursor: pointer;
  transition: all var(--transition-fast);
  margin-bottom: 8px;
  border: 1px solid transparent;
}
.workspace-item:hover {
  background: var(--bg-tertiary);
  border-color: var(--border);
}
.ws-name {
  font-size: 14px;
  font-weight: 600;
  color: var(--text-primary);
  margin-bottom: 4px;
}
.ws-path {
  font-size: 12px;
  color: var(--text-muted);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.empty {
  text-align: center;
  padding: 24px;
  color: var(--text-muted);
  font-size: 13px;
}
.dialog-footer {
  display: flex;
  justify-content: flex-end;
  padding: 12px 20px;
  border-top: 1px solid var(--border-light);
}
.secondary {
  background: var(--bg-tertiary);
  color: var(--text-secondary);
  border: 1px solid var(--border);
  padding: 8px 16px;
  border-radius: var(--radius-md);
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
  transition: all var(--transition-fast);
}
.secondary:hover {
  background: var(--bg-elevated);
  border-color: var(--border-hover);
}
</style>
```

- [ ] **Step 2: Verify component renders (manual check)**

Run: `npm run tauri dev`
Expected: No compile errors, component can be imported

- [ ] **Step 3: Commit**

```bash
git add src/components/workspace/MoveToWorkspaceDialog.vue
git commit -m "feat: add MoveToWorkspaceDialog component"
```

---

### Task 4: Integrate into ThumbnailGrid context menu

**Files:**
- Modify: `src/components/browser/ThumbnailGrid.vue`

**Interfaces:**
- Consumes: `MoveToWorkspaceDialog` from Task 3, `move_images` command from Task 1
- Produces: context menu integration with workspace picker

- [ ] **Step 1: Add import for MoveToWorkspaceDialog**

At the top of `<script setup>`, add:

```typescript
import MoveToWorkspaceDialog from '../workspace/MoveToWorkspaceDialog.vue'
```

- [ ] **Step 2: Add reactive state for workspace picker**

After the `contextMenu` reactive, add:

```typescript
const showWorkspacePicker = ref(false)
```

- [ ] **Step 3: Add moveToWorkspace function**

After the `trashFile` function, add:

```typescript
async function moveToWorkspace() {
  if (!contextMenu.image) return
  showWorkspacePicker.value = true
  closeContextMenu()
}

async function handleWorkspaceSelected(targetWsId: number) {
  showWorkspacePicker.value = false
  if (!contextMenu.image) return
  const { invoke } = await import('@tauri-apps/api/core')
  try {
    await invoke('move_images', {
      ids: [contextMenu.image.id],
      destWorkspaceId: targetWsId,
    })
    emit('refresh')
  } catch (e) {
    console.error('Move to workspace failed:', e)
  }
}
```

- [ ] **Step 4: Add context menu item in template**

In the context menu `<template>`, add after the "Move to folder" item:

```html
<div class="menu-item" @click="moveToWorkspace">
  {{ t('context.moveToWorkspace') }}
</div>
```

- [ ] **Step 5: Add MoveToWorkspaceDialog in template**

After the context menu `</div>`, add:

```html
<MoveToWorkspaceDialog
  v-if="showWorkspacePicker"
  :current-workspace-id="props.workspaceId"
  @select="handleWorkspaceSelected"
  @close="showWorkspacePicker = false"
/>
```

- [ ] **Step 6: Verify context menu appears**

Run: `npm run tauri dev`
Expected: Right-click on image shows "Move to workspace..." item

- [ ] **Step 7: Commit**

```bash
git add src/components/browser/ThumbnailGrid.vue
git commit -m "feat: add move-to-workspace to context menu"
```

---

### Task 5: End-to-end verification

**Files:**
- None (testing only)

**Interfaces:**
- Consumes: all previous tasks
- Produces: verified working feature

- [ ] **Step 1: Run full dev server**

Run: `npm run tauri dev`
Expected: App starts without errors

- [ ] **Step 2: Test move to workspace flow**

1. Create two workspaces (if not already exists)
2. Add images to workspace A
3. Right-click image → "Move to workspace..."
4. Dialog opens showing workspace B
5. Click workspace B
6. Image moves to workspace B's root
7. Verify image appears in workspace B gallery
8. Verify image removed from workspace A gallery
9. Check DB: `workspace_id` updated, `relative_path` correct

- [ ] **Step 3: Test name conflict handling**

1. Move image with name `photo.jpg` to workspace that already has `photo.jpg`
2. Verify moved file renamed to `photo_1.jpg`
3. Verify DB `file_name` updated to `photo_1.jpg`

- [ ] **Step 4: Test existing "Move to folder" still works**

1. Right-click image → "Move to folder..."
2. Select arbitrary directory
3. Verify file moves correctly
4. Verify image removed from DB (current behavior preserved)

- [ ] **Step 5: Final commit**

```bash
git add -A
git commit -m "feat: complete move-to-workspace feature"
```

# Design: Move Images to Existing Workspace

**Date:** 2026-07-06
**Status:** Draft
**Approach:** Extend existing `move_images` command (Approach A)

## Goal

Allow users to move images between workspaces via the context menu, with proper metadata reassignment (workspace_id, relative_path, file_name).

## Current Behavior

- Context menu → "Move to folder..." → OS directory picker → moves file to arbitrary directory
- If dest is inside same workspace → updates `relative_path`/`file_name`
- If dest is outside workspace → **deletes** the image record from DB

## Proposed Behavior

- Context menu → "Move to workspace..." → workspace picker dialog → moves file to target workspace root
- Updates `workspace_id`, `relative_path`, `file_name` in DB
- Clears stale thumbnail cache
- Existing "Move to folder..." remains unchanged

---

## Backend: Extend `move_images`

**File:** `src-tauri/src/commands/metadata.rs`

### Command Signature Change

```rust
#[tauri::command]
pub fn move_images(
    state: State<AppState>,
    thumb_state: State<super::thumbnail::ThumbnailState>,
    ids: Vec<i64>,
    dest_dir: Option<String>,        // existing — arbitrary directory
    dest_workspace_id: Option<i64>,   // new — target workspace ID
) -> Result<MoveResult, String>
```

### Resolution Logic

1. If `dest_workspace_id` is `Some(ws_id)`:
   - Look up workspace path via `get_workspace(conn, ws_id)`
   - Set `dest_dir` = workspace root path
   - Track `target_workspace_id = ws_id`
2. If `dest_dir` is provided but `dest_workspace_id` is `None`:
   - Use current behavior (arbitrary directory)
   - `target_workspace_id = None`
3. If neither is provided → return error

### File Move

Same as current: move file to dest, handle name conflicts with `_1`, `_2` suffixes.

### DB Update (inside transaction)

After all files moved successfully:

```rust
conn.execute_batch("BEGIN")?;

for m in &moved {
    if let Some(target_ws_id) = target_workspace_id {
        // Cross-workspace move: update workspace_id + relative_path + file_name
        conn.execute(
            "UPDATE images SET workspace_id = ?1, relative_path = ?2, file_name = ?3, updated_at = datetime('now') WHERE id = ?4",
            params![target_ws_id, new_file_name, new_file_name, m.id],
        )?;
    } else if let Some(rel_path) = new_relative {
        // Same-workspace move: update relative_path + file_name
        conn.execute(
            "UPDATE images SET relative_path = ?1, file_name = ?2, updated_at = datetime('now') WHERE id = ?3",
            params![rel_path, m.new_file_name, m.id],
        )?;
    } else {
        // Moved outside all workspaces: remove from DB
        delete_db_records(&conn, &[m.id])?;
    }
}

conn.execute_batch("COMMIT")?;
```

### Thumbnail Cache

Clear stale thumbnails for moved image IDs (same as current).

### FTS Index

FTS triggers handle updates automatically via `AFTER UPDATE` trigger on `images`.

---

## Frontend: Workspace Picker Dialog

**New file:** `src/components/workspace/MoveToWorkspaceDialog.vue`

### Props

```typescript
interface Props {
  currentWorkspaceId: number
  excludeWorkspaceId?: number  // optional: workspace to hide (e.g., source)
}
```

### Behavior

- Fetches workspace list from `useWorkspaceStore()`
- Displays workspace cards (name, image count, path)
- Current workspace is highlighted and disabled
- Click a workspace → select it
- "Move" button → emits `select` event with workspace ID
- "Cancel" button or backdrop click → emits `close`

### Styling

Follows existing dialog patterns (`CreateWorkspaceDialog.vue`):
- Modal overlay with backdrop blur
- Centered card with workspace list
- Primary/secondary button styling

---

## Frontend: Context Menu Integration

**File:** `src/components/browser/ThumbnailGrid.vue`

### New Context Menu Item

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

### Template Addition

```html
<!-- In context menu -->
<div class="menu-item" @click="moveToWorkspace">
  {{ t('context.moveToWorkspace') }}
</div>

<!-- Workspace picker dialog -->
<MoveToWorkspaceDialog
  v-if="showWorkspacePicker"
  :current-workspace-id="props.workspaceId"
  @select="handleWorkspaceSelected"
  @close="showWorkspacePicker = false"
/>
```

---

## i18n

**Files:** `src/i18n/zh-CN.ts`, `src/i18n/en.ts`

### New Keys

```typescript
// zh-CN
'context.moveToWorkspace': '移动到工作区...',
'moveToWorkspace.title': '选择目标工作区',
'moveToWorkspace.confirm': '移动',
'moveToWorkspace.currentWorkspace': '当前工作区',

// en
'context.moveToWorkspace': 'Move to workspace...',
'moveToWorkspace.title': 'Select target workspace',
'moveToWorkspace.confirm': 'Move',
'moveToWorkspace.currentWorkspace': 'Current workspace',
```

---

## Edge Cases

1. **Image already in target workspace** → Disable that workspace in picker, or show "Already in this workspace" message
2. **Name conflicts** → Same as current: `_1`, `_2` suffix (handled by existing logic)
3. **Batch moves** → Support multiple images; each gets conflict resolution independently
4. **Cross-drive moves** → Existing fallback: copy + delete (handled by existing logic)

---

## Testing Checklist

- [ ] Right-click image → "Move to workspace..." appears in context menu
- [ ] Click → workspace picker dialog opens
- [ ] Current workspace is disabled/highlighted
- [ ] Select target workspace → "Move" button enabled
- [ ] Click "Move" → file moves to target workspace root
- [ ] DB record updated: workspace_id, relative_path, file_name
- [ ] Stale thumbnail cleared
- [ ] Image appears in target workspace's gallery
- [ ] Source workspace gallery refreshes (image removed)
- [ ] Name conflict → file renamed with `_1` suffix
- [ ] Batch move works (multiple images selected)
- [ ] Existing "Move to folder..." still works unchanged

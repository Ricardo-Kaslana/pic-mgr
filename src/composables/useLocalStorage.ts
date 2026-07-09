import { ref, watch, type Ref } from 'vue'

export interface LocalStorageOptions<T> {
  /**
   * Convert the raw localStorage string into the typed value.
   * If this throws, the default is used and the bad value is
   * overwritten on the next write.
   */
  read?: (raw: string) => T
  /** Convert the typed value into a localStorage string. Defaults to String(). */
  write?: (value: T) => string
  /**
   * Side-effect to run once on init and on every change.
   * Useful for pushing values into the DOM (data-theme attribute)
   * or into other reactive systems (i18n locale).
   */
  apply?: (value: T) => void
}

/**
 * A ref that auto-loads from localStorage on creation and auto-writes
 * back on every change. Replaces the repetitive pattern of
 *   const x = ref(localStorage.getItem(...) ?? default)
 *   watch(x, v => localStorage.setItem(..., v))
 * that was duplicated across the settings store.
 *
 * The watcher runs `{ immediate: true }` so `apply` is also called
 * once during setup — that lets callers wire DOM-side-effects
 * (like the theme attribute) without a second watcher.
 */
export function useLocalStorage<T>(
  key: string,
  defaultValue: T,
  options: LocalStorageOptions<T> = {}
): Ref<T> {
  const read = options.read ?? ((raw: string) => raw as unknown as T)
  const write = options.write ?? ((value: T) => String(value))

  let initial: T = defaultValue
  try {
    const raw = localStorage.getItem(key)
    if (raw !== null) initial = read(raw)
  } catch {
    initial = defaultValue
  }

  const state = ref(initial) as Ref<T>

  watch(
    state,
    (v) => {
      localStorage.setItem(key, write(v))
      options.apply?.(v)
    },
    { immediate: true }
  )

  return state
}
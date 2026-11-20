import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import type { UnlistenFn } from '@tauri-apps/api/event'

export const isNative =
  typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window

export const isMacOS =
  typeof navigator !== 'undefined' && /Mac/i.test(navigator.userAgent)

const STORAGE_KEYS = {
  workspaces: 'flashbee:workspaces:v1',
  settings: 'flashbee:settings:v1',
  layout: 'flashbee:layout:v1',
} as const

type StoreName = keyof typeof STORAGE_KEYS

export function invokeNative<T>(
  command: string,
  args?: Record<string, unknown>,
): Promise<T> {
  return invoke<T>(command, args)
}

export function markNativeEnvironment() {
  const root = document.documentElement
  root.dataset.native = isNative ? 'true' : 'false'
  root.dataset.os = isMacOS ? 'macos' : 'other'
}

/** Loads the native app-data store into localStorage before React mounts. */
export async function hydrateNativeStore() {
  if (!isNative) return

  await Promise.all(
    (Object.entries(STORAGE_KEYS) as Array<[StoreName, string]>).map(
      async ([name, key]) => {
        try {
          const contents = await invoke<string | null>('load_store', { name })
          if (contents) window.localStorage.setItem(key, contents)
        } catch {
          // Fall back to localStorage when the native store is unavailable.
        }
      },
    ),
  )
}

/** Mirrors localStorage into the app-data directory so state survives webview resets. */
export function persistNativeStore(name: StoreName, contents: string) {
  if (!isNative) return
  void invoke('save_store', { name, contents }).catch(() => undefined)
}

export function onMenuCommand(handler: (command: string) => void): () => void {
  if (!isNative) return () => undefined

  let disposed = false
  let unlisten: UnlistenFn | undefined
  void listen<string>('menu-command', (event) => handler(event.payload)).then(
    (stop) => {
      if (disposed) stop()
      else unlisten = stop
    },
  )

  return () => {
    disposed = true
    unlisten?.()
  }
}

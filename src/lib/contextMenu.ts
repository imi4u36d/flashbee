import { Menu } from '@tauri-apps/api/menu'
import { writeText } from '@tauri-apps/plugin-clipboard-manager'
import { isNative } from './native'
import type { MouseEvent as ReactMouseEvent } from 'react'

export type ContextMenuEntry =
  | 'separator'
  | {
      id?: string
      label: string
      enabled?: boolean
      danger?: boolean
      action: () => void
    }

function menuItems(entries: ContextMenuEntry[]) {
  return entries.map((entry) => {
    if (entry === 'separator') {
      return { item: 'Separator' as const }
    }
    return {
      id: entry.id,
      text: entry.label,
      enabled: entry.enabled ?? true,
      action: entry.action,
    }
  })
}

export function showContextMenu(
  event: ReactMouseEvent,
  entries: ContextMenuEntry[],
) {
  event.preventDefault()
  event.stopPropagation()
  if (!isNative) return

  void (async () => {
    try {
      const menu = await Menu.new({ items: menuItems(entries) })
      await menu.popup()
    } catch (error) {
      console.warn('Could not open the context menu.', error)
    }
  })()
}

export async function copyText(text: string) {
  if (isNative) {
    await writeText(text)
    return
  }
  await navigator.clipboard.writeText(text)
}

/** Native edit menu for text fields, so typing controls feel like a Mac app. */
export async function showTextFieldContextMenu() {
  if (!isNative) return
  try {
    const menu = await Menu.new({
      items: [
        { item: 'Undo' },
        { item: 'Redo' },
        { item: 'Separator' },
        { item: 'Cut' },
        { item: 'Copy' },
        { item: 'Paste' },
        { item: 'SelectAll' },
      ],
    })
    await menu.popup()
  } catch (error) {
    console.warn('Could not open the edit menu.', error)
  }
}

/** Copy/select menu for diff output, where text selection is intentional. */
export async function showSelectionContextMenu(surface: HTMLElement) {
  if (!isNative) return
  const selection = window.getSelection()?.toString() ?? ''
  const isChinese = document.documentElement.lang.toLowerCase().startsWith('zh')

  try {
    const menu = await Menu.new({
      items: [
        {
          id: 'copy-selection',
          text: isChinese ? '复制' : 'Copy',
          enabled: selection.length > 0,
          action: () => {
            void writeText(selection).catch(() => undefined)
          },
        },
        {
          id: 'select-all-diff',
          text: isChinese ? '全选' : 'Select All',
          action: () => {
            const range = document.createRange()
            range.selectNodeContents(surface)
            const next = window.getSelection()
            next?.removeAllRanges()
            next?.addRange(range)
          },
        },
      ],
    })
    await menu.popup()
  } catch (error) {
    console.warn('Could not open the text menu.', error)
  }
}

/**
 * Installs app-wide context menu behavior:
 * text fields get the native edit menu, diff code gets copy/select all,
 * and the webview's default page menu never appears.
 */
export function installContextMenuPolicy() {
  if (!isNative) return

  document.addEventListener(
    'contextmenu',
    (event) => {
      if (event.defaultPrevented) return

      const target = event.target as HTMLElement | null
      if (!target) return

      if (
        target instanceof HTMLInputElement ||
        target instanceof HTMLTextAreaElement ||
        target.isContentEditable
      ) {
        event.preventDefault()
        void showTextFieldContextMenu()
        return
      }

      const diffSurface = target.closest<HTMLElement>('.diff-surface')
      if (diffSurface) {
        event.preventDefault()
        void showSelectionContextMenu(diffSurface)
        return
      }

      // Never expose the webview's default menu.
      event.preventDefault()
    },
    { capture: false },
  )
}

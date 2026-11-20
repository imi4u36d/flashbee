import { persistNativeStore } from './native'

export const SIDEBAR_WIDTH_MIN = 240
export const SIDEBAR_WIDTH_MAX = 480

const STORAGE_KEY = 'flashbee:layout:v1'
const LAYOUT_VERSION = 1

export function clampSidebarWidth(width: number) {
  return Math.round(
    Math.min(SIDEBAR_WIDTH_MAX, Math.max(SIDEBAR_WIDTH_MIN, width)),
  )
}

export function loadSidebarWidth(): number | null {
  try {
    const raw = window.localStorage.getItem(STORAGE_KEY)
    if (!raw) return null

    const parsed = JSON.parse(raw) as { sidebarWidth?: unknown }
    if (
      typeof parsed.sidebarWidth !== 'number' ||
      !Number.isFinite(parsed.sidebarWidth)
    ) {
      return null
    }
    return clampSidebarWidth(parsed.sidebarWidth)
  } catch {
    return null
  }
}

export function saveSidebarWidth(width: number | null) {
  const contents = JSON.stringify({
    version: LAYOUT_VERSION,
    sidebarWidth: width,
  })
  try {
    window.localStorage.setItem(STORAGE_KEY, contents)
    persistNativeStore('layout', contents)
  } catch {
    // The width still applies for the current session.
  }
}

import { persistNativeStore } from './native'
import type { LocalePreference } from './i18n'

export type ThemeMode = 'system' | 'light' | 'dark'
export type ResolvedTheme = 'light' | 'dark'
export type UiFont = 'inter' | 'system'
export type CodeFont = 'jetbrains' | 'system'

export interface AppSettings {
  theme: ThemeMode
  locale: LocalePreference
  uiFont: UiFont
  codeFont: CodeFont
  uiSize: number
  codeSize: number
}

export const UI_SIZE_MIN = 14
export const UI_SIZE_MAX = 18
export const CODE_SIZE_MIN = 12
export const CODE_SIZE_MAX = 20

export const DEFAULT_SETTINGS: AppSettings = {
  theme: 'system',
  locale: 'system',
  uiFont: 'system',
  codeFont: 'system',
  uiSize: 15,
  codeSize: 14,
}

export const UI_FONT_STACKS: Record<UiFont, string> = {
  inter:
    "'Inter Variable', -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif",
  system:
    "-apple-system, BlinkMacSystemFont, 'Segoe UI', 'Helvetica Neue', sans-serif",
}

export const CODE_FONT_STACKS: Record<CodeFont, string> = {
  jetbrains:
    "'JetBrains Mono Variable', 'SFMono-Regular', Menlo, Consolas, monospace",
  system:
    "ui-monospace, 'SFMono-Regular', Menlo, Monaco, Consolas, monospace",
}

const STORAGE_KEY = 'flashbee:settings:v1'
const SETTINGS_VERSION = 3

const isThemeMode = (value: unknown): value is ThemeMode =>
  value === 'system' || value === 'light' || value === 'dark'

const isUiFont = (value: unknown): value is UiFont =>
  value === 'inter' || value === 'system'

const isCodeFont = (value: unknown): value is CodeFont =>
  value === 'jetbrains' || value === 'system'

const isLocalePreference = (value: unknown): value is LocalePreference =>
  value === 'system' || value === 'en' || value === 'zh'

export function loadSettings(): AppSettings {
  try {
    const raw = window.localStorage.getItem(STORAGE_KEY)
    if (!raw) return DEFAULT_SETTINGS

    const parsed = JSON.parse(raw) as Partial<AppSettings> & { version?: number }
    const hasScaledSizes = (parsed.version ?? 0) >= 2
    const uiSize =
      hasScaledSizes &&
      typeof parsed.uiSize === 'number' &&
      parsed.uiSize >= UI_SIZE_MIN &&
      parsed.uiSize <= UI_SIZE_MAX
        ? parsed.uiSize
        : DEFAULT_SETTINGS.uiSize
    const codeSize =
      hasScaledSizes &&
      typeof parsed.codeSize === 'number' &&
      parsed.codeSize >= CODE_SIZE_MIN &&
      parsed.codeSize <= CODE_SIZE_MAX
        ? parsed.codeSize
        : DEFAULT_SETTINGS.codeSize

    return {
      theme: isThemeMode(parsed.theme) ? parsed.theme : DEFAULT_SETTINGS.theme,
      locale: isLocalePreference(parsed.locale)
        ? parsed.locale
        : DEFAULT_SETTINGS.locale,
      uiFont: isUiFont(parsed.uiFont) ? parsed.uiFont : DEFAULT_SETTINGS.uiFont,
      codeFont: isCodeFont(parsed.codeFont)
        ? parsed.codeFont
        : DEFAULT_SETTINGS.codeFont,
      uiSize,
      codeSize,
    }
  } catch {
    return DEFAULT_SETTINGS
  }
}

export function saveSettings(settings: AppSettings) {
  const contents = JSON.stringify({ version: SETTINGS_VERSION, ...settings })
  try {
    window.localStorage.setItem(STORAGE_KEY, contents)
    persistNativeStore('settings', contents)
  } catch {
    // Preferences still apply for the current session.
  }
}

export function systemPrefersDark() {
  return window.matchMedia('(prefers-color-scheme: dark)').matches
}

export function resolveTheme(
  theme: ThemeMode,
  prefersDark: boolean,
): ResolvedTheme {
  if (theme === 'system') return prefersDark ? 'dark' : 'light'
  return theme
}

export function applyDocumentSettings(
  settings: AppSettings,
  theme: ResolvedTheme,
) {
  const root = document.documentElement
  root.dataset.theme = theme
  root.style.setProperty('--font-ui', UI_FONT_STACKS[settings.uiFont])
  root.style.setProperty('--font-mono', CODE_FONT_STACKS[settings.codeFont])
  root.style.setProperty('--code-size', `${settings.codeSize}px`)
  root.style.fontSize = `${settings.uiSize}px`

  document
    .querySelector('meta[name="theme-color"]')
    ?.setAttribute('content', theme === 'light' ? '#f3f5f8' : '#262628')
}

/** Applies the saved theme before React renders to avoid a flash of the wrong theme. */
export function applyBootSettings() {
  if (typeof window === 'undefined') return
  const settings = loadSettings()
  applyDocumentSettings(settings, resolveTheme(settings.theme, systemPrefersDark()))
}

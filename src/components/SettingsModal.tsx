import { useEffect, useRef } from 'react'
import {
  Check,
  Code2,
  Languages,
  Monitor,
  Moon,
  RotateCcw,
  Settings2,
  Sun,
  Type,
  X,
} from 'lucide-react'
import type { CSSProperties } from 'react'
import { localeName, useI18n, type LocalePreference } from '../lib/i18n'
import {
  CODE_SIZE_MAX,
  CODE_SIZE_MIN,
  DEFAULT_SETTINGS,
  UI_SIZE_MAX,
  UI_SIZE_MIN,
  type AppSettings,
  type ThemeMode,
} from '../lib/settings'

interface SettingsModalProps {
  settings: AppSettings
  systemDark: boolean
  onChange: (patch: Partial<AppSettings>) => void
  onReset: () => void
  onClose: () => void
}

const themeOptions: Array<{
  id: ThemeMode
  icon: typeof Monitor
}> = [
  { id: 'system', icon: Monitor },
  { id: 'light', icon: Sun },
  { id: 'dark', icon: Moon },
]

const languageOptions: Array<{ id: LocalePreference }> = [
  { id: 'system' },
  { id: 'en' },
  { id: 'zh' },
]

function ThumbPane({ variant }: { variant: 'light' | 'dark' }) {
  return (
    <span className={`thumb-pane thumb-${variant}`}>
      <span className="thumb-side">
        <i />
        <i />
        <i />
      </span>
      <span className="thumb-main">
        <span className="thumb-toolbar" />
        <span className="thumb-line thumb-add" />
        <span className="thumb-line" />
        <span className="thumb-line thumb-del" />
        <span className="thumb-line thumb-short" />
      </span>
    </span>
  )
}

function ThemePreview({ mode }: { mode: ThemeMode }) {
  return (
    <span className={`theme-thumb theme-thumb-${mode}`} aria-hidden="true">
      {mode === 'system' ? (
        <span className="thumb-split">
          <ThumbPane variant="light" />
          <ThumbPane variant="dark" />
        </span>
      ) : (
        <ThumbPane variant={mode} />
      )}
    </span>
  )
}

export function SettingsModal({
  settings,
  systemDark,
  onChange,
  onReset,
  onClose,
}: SettingsModalProps) {
  const { locale, t } = useI18n()
  const modalRef = useRef<HTMLDivElement>(null)
  const systemThemeWord = t(
    systemDark ? 'settings.darkWord' : 'settings.lightWord',
  )
  const languageLabels: Record<LocalePreference, string> = {
    system: t('settings.languageAuto'),
    en: t('settings.languageEnglish'),
    zh: t('settings.languageChinese'),
  }
  const uiSliderProgress =
    ((settings.uiSize - UI_SIZE_MIN) / (UI_SIZE_MAX - UI_SIZE_MIN)) * 100
  const codeSliderProgress =
    ((settings.codeSize - CODE_SIZE_MIN) / (CODE_SIZE_MAX - CODE_SIZE_MIN)) * 100
  const isDefault =
    settings.theme === DEFAULT_SETTINGS.theme &&
    settings.locale === DEFAULT_SETTINGS.locale &&
    settings.uiFont === DEFAULT_SETTINGS.uiFont &&
    settings.codeFont === DEFAULT_SETTINGS.codeFont &&
    settings.uiSize === DEFAULT_SETTINGS.uiSize &&
    settings.codeSize === DEFAULT_SETTINGS.codeSize

  useEffect(() => {
    const modal = modalRef.current
    const opener = document.activeElement as HTMLElement | null
    modal?.focus()

    const trapFocus = (event: KeyboardEvent) => {
      if (event.key !== 'Tab' || !modal) return
      const focusable = Array.from(
        modal.querySelectorAll<HTMLElement>(
          'button:not([disabled]), input:not([disabled]), [href], [tabindex]:not([tabindex="-1"])',
        ),
      ).filter((element) => element.getClientRects().length > 0)

      if (focusable.length === 0) return
      const first = focusable[0]
      const last = focusable[focusable.length - 1]

      if (event.shiftKey && document.activeElement === first) {
        event.preventDefault()
        last.focus()
      } else if (!event.shiftKey && document.activeElement === last) {
        event.preventDefault()
        first.focus()
      }
    }

    modal?.addEventListener('keydown', trapFocus)
    return () => {
      modal?.removeEventListener('keydown', trapFocus)
      opener?.focus?.()
    }
  }, [])

  return (
    <div className="settings-layer">
      <button
        className="settings-backdrop"
        onClick={onClose}
        aria-label={t('settings.close')}
        tabIndex={-1}
      />
      <div
        ref={modalRef}
        className="settings-modal"
        role="dialog"
        aria-modal="true"
        aria-labelledby="settings-title"
        tabIndex={-1}
      >
        <header className="settings-header">
          <div className="settings-heading">
            <span className="settings-mark">
              <Settings2 size={16} strokeWidth={1.9} />
            </span>
            <div>
              <span className="eyebrow">{t('settings.preferences')}</span>
              <h2 id="settings-title">{t('settings.title')}</h2>
            </div>
          </div>
          <div className="settings-header-actions">
            <kbd>esc</kbd>
            <button
              className="icon-button subtle"
              onClick={onClose}
              aria-label={t('settings.close')}
              title={t('common.close')}
            >
              <X size={16} strokeWidth={1.9} />
            </button>
          </div>
        </header>

        <div className="settings-scroll">
          <section className="settings-section">
            <div className="settings-section-head">
              <div>
                <h3>{t('settings.appearance')}</h3>
                <p>{t('settings.appearanceDescription')}</p>
              </div>
              <span className="system-note">
                <Monitor size={11} strokeWidth={1.9} />
                {t('settings.systemIs', { theme: systemThemeWord })}
              </span>
            </div>

            <div className="theme-grid" role="radiogroup" aria-label={t('settings.themeAria')}>
              {themeOptions.map((option) => {
                const Icon = option.icon
                const selected = settings.theme === option.id
                return (
                  <button
                    key={option.id}
                    type="button"
                    role="radio"
                    aria-checked={selected}
                    data-testid={`theme-${option.id}`}
                    className={`theme-option ${selected ? 'is-selected' : ''}`}
                    onClick={() => onChange({ theme: option.id })}
                  >
                    <ThemePreview mode={option.id} />
                    <span className="theme-option-copy">
                      <span className="theme-option-title">
                        <Icon size={12} strokeWidth={1.9} />
                        {option.id === 'system'
                          ? t('settings.system')
                          : option.id === 'light'
                            ? t('settings.light')
                            : t('settings.dark')}
                      </span>
                      <span className="theme-option-description">
                        {option.id === 'system'
                          ? t('settings.systemDescription', {
                              theme: systemThemeWord,
                            })
                          : option.id === 'light'
                            ? t('settings.lightDescription')
                            : t('settings.darkDescription')}
                      </span>
                    </span>
                    <span className="theme-check" aria-hidden="true">
                      <Check size={10} strokeWidth={2.6} />
                    </span>
                  </button>
                )
              })}
            </div>
            <div className="settings-row settings-row-language">
              <div className="settings-row-copy">
                <span className="settings-row-title">
                  <Languages size={13} strokeWidth={1.8} />
                  {t('settings.interfaceLanguage')}
                </span>
                <span className="settings-row-description">
                  {settings.locale === 'system'
                    ? t('settings.languageAutoDescription', {
                        language: localeName(locale),
                      })
                    : t('settings.interfaceLanguageDescription')}
                </span>
              </div>
              <div
                className="segmented-control"
                role="group"
                aria-label={t('settings.languageAria')}
              >
                {languageOptions.map((option) => (
                  <button
                    key={option.id}
                    type="button"
                    className={settings.locale === option.id ? 'is-active' : ''}
                    data-testid={`locale-${option.id}`}
                    aria-pressed={settings.locale === option.id}
                    title={
                      option.id === 'system'
                        ? t('settings.languageAutoDescription', {
                            language: localeName(locale),
                          })
                        : undefined
                    }
                    onClick={() => onChange({ locale: option.id })}
                  >
                    {languageLabels[option.id]}
                  </button>
                ))}
              </div>
            </div>
          </section>

          <section className="settings-section">
            <div className="settings-section-head">
              <div>
                <h3>{t('settings.typography')}</h3>
                <p>{t('settings.typographyDescription')}</p>
              </div>
            </div>

            <div className="settings-row">
              <div className="settings-row-copy">
                <span className="settings-row-title">
                  <Type size={13} strokeWidth={1.8} />
                  {t('settings.interfaceFont')}
                </span>
                <span className="settings-row-description">
                  {t('settings.interfaceFontDescription')}
                </span>
              </div>
              <div className="segmented-control" role="group" aria-label={t('settings.uiFontAria')}>
                <button
                  type="button"
                  className={settings.uiFont === 'inter' ? 'is-active' : ''}
                  data-testid="ui-font-inter"
                  aria-pressed={settings.uiFont === 'inter'}
                  onClick={() => onChange({ uiFont: 'inter' })}
                >
                  <span className="segment-sample sample-inter">Aa</span>
                  Inter
                </button>
                <button
                  type="button"
                  className={settings.uiFont === 'system' ? 'is-active' : ''}
                  data-testid="ui-font-system"
                  aria-pressed={settings.uiFont === 'system'}
                  onClick={() => onChange({ uiFont: 'system' })}
                >
                  <span className="segment-sample sample-system">Aa</span>
                  {t('common.system')}
                </button>
              </div>
            </div>

            <div className="settings-row">
              <div className="settings-row-copy">
                <span className="settings-row-title">{t('settings.interfaceSize')}</span>
                <span className="settings-row-description">
                  {t('settings.interfaceSizeDescription')}
                </span>
              </div>
              <div className="size-control">
                <span className="size-bound">{UI_SIZE_MIN}</span>
                <input
                  type="range"
                  min={UI_SIZE_MIN}
                  max={UI_SIZE_MAX}
                  step={1}
                  value={settings.uiSize}
                  aria-label={t('settings.uiSizeAria')}
                  data-testid="ui-size"
                  style={
                    {
                      '--range-progress': `${uiSliderProgress}%`,
                    } as CSSProperties
                  }
                  onChange={(event) =>
                    onChange({ uiSize: Number(event.target.value) })
                  }
                />
                <span className="size-bound">{UI_SIZE_MAX}</span>
                <span className="size-value">{settings.uiSize}px</span>
              </div>
            </div>

            <div className="settings-row">
              <div className="settings-row-copy">
                <span className="settings-row-title">
                  <Code2 size={13} strokeWidth={1.8} />
                  {t('settings.codeFont')}
                </span>
                <span className="settings-row-description">
                  {t('settings.codeFontDescription')}
                </span>
              </div>
              <div className="segmented-control" role="group" aria-label={t('settings.codeFontAria')}>
                <button
                  type="button"
                  className={settings.codeFont === 'jetbrains' ? 'is-active' : ''}
                  data-testid="code-font-jetbrains"
                  aria-pressed={settings.codeFont === 'jetbrains'}
                  onClick={() => onChange({ codeFont: 'jetbrains' })}
                >
                  <span className="segment-mono">01</span>
                  JetBrains
                </button>
                <button
                  type="button"
                  className={settings.codeFont === 'system' ? 'is-active' : ''}
                  data-testid="code-font-system"
                  aria-pressed={settings.codeFont === 'system'}
                  onClick={() => onChange({ codeFont: 'system' })}
                >
                  <span className="segment-mono">01</span>
                  {t('common.system')}
                </button>
              </div>
            </div>

            <div className="settings-row settings-row-size">
              <div className="settings-row-copy">
                <span className="settings-row-title">{t('settings.codeSize')}</span>
                <span className="settings-row-description">
                  {t('settings.codeSizeDescription')}
                </span>
              </div>
              <div className="size-control">
                <span className="size-bound">{CODE_SIZE_MIN}</span>
                <input
                  type="range"
                  min={CODE_SIZE_MIN}
                  max={CODE_SIZE_MAX}
                  step={1}
                  value={settings.codeSize}
                  aria-label={t('settings.codeSizeAria')}
                  data-testid="code-size"
                  style={
                    {
                      '--range-progress': `${codeSliderProgress}%`,
                    } as CSSProperties
                  }
                  onChange={(event) =>
                    onChange({ codeSize: Number(event.target.value) })
                  }
                />
                <span className="size-bound">{CODE_SIZE_MAX}</span>
                <span className="size-value">{settings.codeSize}px</span>
              </div>
            </div>

            <div className="type-preview">
              <div className="type-preview-head">
                <span>
                  <span className="preview-live-dot" />
                  {t('settings.livePreview')}
                </span>
                <span className="preview-token">
                  {settings.uiFont === 'inter' ? 'Inter' : 'System'} ·{' '}
                  {settings.codeFont === 'jetbrains' ? 'JetBrains Mono' : 'System Mono'}
                </span>
              </div>
              <pre className="type-preview-code">
                <span className="tok-comment">{t('settings.previewComment')}</span>
                {'\n'}
                <span className="tok-keyword">const</span>{' '}
                <span className="tok-variable">review</span>{' '}
                <span className="tok-punctuation">=</span>{' '}
                <span className="tok-keyword">await</span>{' '}
                <span className="tok-function">flashbee</span>
                <span className="tok-punctuation">.</span>
                <span className="tok-function">diff</span>
                <span className="tok-punctuation">(</span>
                <span className="tok-string">'main'</span>
                <span className="tok-punctuation">)</span>
              </pre>
              <p className="type-preview-ui">
                {t('settings.previewSentence')}
              </p>
            </div>
          </section>
        </div>

        <footer className="settings-footer">
          <span className="settings-saved">
            <span className="saved-dot" />
            {t('settings.saved')}
          </span>
          <div className="settings-footer-actions">
            <button
              data-testid="settings-reset"
              className={`settings-reset ${isDefault ? 'is-disabled' : ''}`}
              disabled={isDefault}
              onClick={onReset}
            >
              <RotateCcw size={12} strokeWidth={1.9} />
              {t('settings.restore')}
            </button>
            <button className="settings-done" data-testid="settings-done" onClick={onClose}>
              {t('settings.done')}
            </button>
          </div>
        </footer>
      </div>
    </div>
  )
}

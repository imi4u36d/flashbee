import { useEffect, useMemo, useState } from 'react'
import {
  ArrowDown,
  ArrowUp,
  Check,
  Columns2,
  Copy,
  Rows3,
} from 'lucide-react'
import {
  fileStats,
  languageLabels,
  pathParts,
  statusLabels,
} from '../data/projects'
import { useI18n, type TranslationKey } from '../lib/i18n'
import { buildSplitRows } from '../lib/diff'
import { highlightLine } from '../lib/highlight'
import type { ResolvedTheme } from '../lib/settings'
import type { DiffFile, DiffLine, DiffMode, Hunk } from '../types'

interface DiffViewerProps {
  file: DiffFile
  mode: DiffMode
  baseBranch: string
  appTheme: ResolvedTheme
  fileIndex: number
  fileCount: number
  hasPrev: boolean
  hasNext: boolean
  onModeChange: (mode: DiffMode) => void
  onPrev: () => void
  onNext: () => void
  onToast: (key: TranslationKey, vars?: Record<string, string | number>) => void
}

function LineCode({
  line,
  language,
  appTheme,
}: {
  line?: DiffLine
  language: string
  appTheme: ResolvedTheme
}) {
  if (!line) return <span className="line-code" />

  return (
    <span
      className="line-code"
      dangerouslySetInnerHTML={{
        __html: highlightLine(line.content, language, appTheme),
      }}
    />
  )
}

function HunkHeader({ hunk }: { hunk: Hunk }) {
  return (
    <div className="hunk-header">
      <span className="hunk-gutter" />
      <span className="hunk-text">{hunk.header}</span>
    </div>
  )
}

function SplitDiff({
  file,
  baseBranch,
  appTheme,
}: {
  file: DiffFile
  baseBranch: string
  appTheme: ResolvedTheme
}) {
  const { t } = useI18n()
  const rows = useMemo(() => buildSplitRows(file.hunks), [file])

  return (
    <div className="diff-split">
      <div className="split-column-header">
        <div className="split-column-title">
          <span className="column-label">{t('diff.original')}</span>
          <span className="column-ref">{baseBranch}</span>
        </div>
        <div className="split-column-title">
          <span className="column-label">{t('diff.modified')}</span>
          <span className="column-ref">{t('common.workingTree')}</span>
        </div>
      </div>

      <div className="split-grid">
        {rows.map((row, index) => {
          if (row.kind === 'hunk') {
            return <HunkHeader key={`hunk-${index}`} hunk={row.hunk} />
          }

          const { left, right } = row.row
          return (
            <div className="split-row" key={`row-${index}`}>
              <div
                className={`split-side side-left ${
                  left ? `is-${left.type}` : 'is-empty'
                }`}
              >
                <span className="line-number">{left?.oldNo ?? ''}</span>
                <span className="line-marker">
                  {left?.type === 'del' ? '−' : ''}
                </span>
                <LineCode line={left} language={file.language} appTheme={appTheme} />
              </div>
              <div
                className={`split-side side-right ${
                  right ? `is-${right.type}` : 'is-empty'
                }`}
              >
                <span className="line-number">{right?.newNo ?? ''}</span>
                <span className="line-marker">
                  {right?.type === 'add' ? '+' : ''}
                </span>
                <LineCode line={right} language={file.language} appTheme={appTheme} />
              </div>
            </div>
          )
        })}
      </div>
    </div>
  )
}

function UnifiedDiff({
  file,
  appTheme,
}: {
  file: DiffFile
  appTheme: ResolvedTheme
}) {
  const { t } = useI18n()

  return (
    <div className="diff-unified">
      <div className="unified-column-header">
        <span className="unified-gutter-label">{t('diff.oldGutter')}</span>
        <span className="unified-gutter-label">{t('diff.newGutter')}</span>
        <span />
        <span className="column-label">{t('diff.unifiedHeader')}</span>
      </div>

      {file.hunks.map((hunk, hunkIndex) => (
        <div className="unified-hunk" key={`unified-${hunkIndex}`}>
          <HunkHeader hunk={hunk} />
          {hunk.lines.map((line, lineIndex) => (
            <div
              className={`unified-row is-${line.type}`}
              key={`unified-line-${hunkIndex}-${lineIndex}`}
            >
              <span className="line-number">{line.oldNo ?? ''}</span>
              <span className="line-number">{line.newNo ?? ''}</span>
              <span className="line-marker">
                {line.type === 'add' ? '+' : line.type === 'del' ? '−' : ''}
              </span>
              <LineCode line={line} language={file.language} appTheme={appTheme} />
            </div>
          ))}
        </div>
      ))}
    </div>
  )
}

export function DiffViewer({
  file,
  mode,
  baseBranch,
  appTheme,
  fileIndex,
  fileCount,
  hasPrev,
  hasNext,
  onModeChange,
  onPrev,
  onNext,
  onToast,
}: DiffViewerProps) {
  const { t } = useI18n()
  const [copied, setCopied] = useState(false)
  const stats = useMemo(() => fileStats(file), [file])
  const { name, dir } = pathParts(file.path)
  const language = languageLabels[file.language] ?? file.language.toUpperCase()

  useEffect(() => {
    if (!copied) return
    const timer = window.setTimeout(() => setCopied(false), 1800)
    return () => window.clearTimeout(timer)
  }, [copied])

  const copyPath = async () => {
    try {
      await navigator.clipboard.writeText(file.path)
      setCopied(true)
      onToast('diff.pathCopied')
    } catch {
      onToast('diff.copyFailed')
    }
  }

  return (
    <section className="diff-viewer">
      <header className="file-toolbar">
        <div className="file-heading">
          <span className={`file-status status-${file.status}`}>
            {statusLabels[file.status]}
          </span>
          <div className="file-heading-copy">
            <div className="file-title-row">
              <h1>{name}</h1>
              <span className="language-chip">{language}</span>
            </div>
            <div className="file-path-line">
              <span>{dir || '.'}</span>
              <span className="path-dot">·</span>
              <span className="staged-label">
                {file.staged ? t('common.staged') : t('common.unstaged')}
              </span>
            </div>
          </div>
        </div>

        <div className="file-actions">
          <div className="file-stats" aria-label={t('diff.lineChanges')}>
            <span className="file-add">+{stats.additions}</span>
            <span className="file-del">−{stats.deletions}</span>
          </div>

          <span className="action-divider" />

          <div className="view-switch" role="group" aria-label={t('diff.layout')}>
            <button
              data-testid="diff-mode-split"
              className={mode === 'split' ? 'is-active' : ''}
              onClick={() => onModeChange('split')}
              title={t('diff.splitTitle')}
            >
              <Columns2 size={13} strokeWidth={1.9} />
              <span>{t('diff.split')}</span>
            </button>
            <button
              data-testid="diff-mode-unified"
              className={mode === 'unified' ? 'is-active' : ''}
              onClick={() => onModeChange('unified')}
              title={t('diff.inlineTitle')}
            >
              <Rows3 size={13} strokeWidth={1.9} />
              <span>{t('diff.unified')}</span>
            </button>
          </div>

          <div className="file-nav">
            <button
              className="icon-button nav-button"
              onClick={onPrev}
              disabled={!hasPrev}
              title={t('diff.previousTitle')}
              aria-label={t('diff.previous')}
            >
              <ArrowUp size={14} strokeWidth={1.9} />
            </button>
            <button
              className="icon-button nav-button"
              onClick={onNext}
              disabled={!hasNext}
              title={t('diff.nextTitle')}
              aria-label={t('diff.next')}
            >
              <ArrowDown size={14} strokeWidth={1.9} />
            </button>
          </div>

          <button
            className="icon-button toolbar-button"
            onClick={copyPath}
            title={t('diff.copyPath')}
            aria-label={t('diff.copyPath')}
          >
            {copied ? (
              <Check size={14} strokeWidth={2.2} className="copied-icon" />
            ) : (
              <Copy size={14} strokeWidth={1.8} />
            )}
          </button>
        </div>
      </header>

      <div className="diff-scroll">
        <div className="diff-surface">
          {mode === 'split' ? (
            <SplitDiff file={file} baseBranch={baseBranch} appTheme={appTheme} />
          ) : (
            <UnifiedDiff file={file} appTheme={appTheme} />
          )}
        </div>
      </div>

      <footer className="diff-footer">
        <div className="footer-left">
          <span className="footer-worktree">
            <span className="worktree-dot" />
            {t('common.workingTree')}
          </span>
          <span className="footer-separator" />
          <span>{language}</span>
          <span>UTF-8</span>
          <span>LF</span>
        </div>

        <div className="footer-right">
          <span className="ref-compare">
            {baseBranch} <span className="ref-arrow">→</span>{' '}
            {t('common.workingTree')}
          </span>
          <span className="footer-separator" />
          <span className="file-position">
            {fileIndex + 1} / {fileCount}
          </span>
          <span className="key-hint">
            <kbd>J</kbd>
            <kbd>K</kbd>
          </span>
        </div>
      </footer>
    </section>
  )
}

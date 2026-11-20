import { useEffect, useMemo, useRef, useState } from 'react'
import type { MouseEvent as ReactMouseEvent } from 'react'
import {
  ArrowDown,
  ArrowUp,
  Check,
  Columns2,
  Copy,
  LoaderCircle,
  Rows3,
} from 'lucide-react'
import {
  fileStats,
  languageLabels,
  pathParts,
  statusLabels,
} from '../data/projects'
import { useI18n, type Translate, type TranslationKey } from '../lib/i18n'
import { copyText } from '../lib/contextMenu'
import { buildSplitRows } from '../lib/diff'
import { highlightLine } from '../lib/highlight'
import { fetchBlame } from '../lib/repositoryApi'
import type { ResolvedTheme } from '../lib/settings'
import type { BlameLine, DiffFile, DiffLine, DiffMode, Hunk } from '../types'

type BlameSide = 'old' | 'new'

interface BlameHover {
  side: BlameSide
  lineNo: number
  x: number
  y: number
}

interface DiffViewerProps {
  file: DiffFile
  mode: DiffMode
  baseBranch: string
  repositoryPath: string | null
  appTheme: ResolvedTheme
  fileIndex: number
  fileCount: number
  onModeChange: (mode: DiffMode) => void
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

function HunkHeader({
  hunk,
  index,
  flashing,
}: {
  hunk: Hunk
  index: number
  flashing: boolean
}) {
  return (
    <div
      className={`hunk-header ${flashing ? 'is-flashing' : ''}`}
      data-hunk-index={index}
    >
      <span className="hunk-gutter" />
      <span className="hunk-text">{hunk.header}</span>
    </div>
  )
}

function formatBlameTime(time: number, t: Translate) {
  if (!time) return ''
  const elapsed = Math.max(0, Date.now() - time * 1000)
  const minutes = Math.floor(elapsed / 60_000)
  if (minutes < 1) return t('blame.justNow')
  if (minutes < 60) return t('blame.minutesAgo', { count: minutes })
  const hours = Math.floor(minutes / 60)
  if (hours < 24) return t('blame.hoursAgo', { count: hours })
  const days = Math.floor(hours / 24)
  if (days < 30) return t('blame.daysAgo', { count: days })
  const months = Math.floor(days / 30)
  if (months < 12) return t('blame.monthsAgo', { count: months })
  return t('blame.yearsAgo', { count: Math.floor(months / 12) })
}

function BlamePopover({
  hover,
  entry,
  failed,
}: {
  hover: BlameHover
  entry?: BlameLine
  failed: boolean
}) {
  const { t } = useI18n()
  const width = 300
  const margin = 12
  const estimatedHeight = 76
  const left = Math.max(
    margin,
    Math.min(hover.x + 16, window.innerWidth - width - margin),
  )
  const above = hover.y + estimatedHeight + 18 > window.innerHeight
  const top = above ? hover.y - 14 : hover.y + 18
  const uncommitted = entry?.uncommitted === true || (failed && hover.side === 'new')

  return (
    <div
      className={`blame-popover ${above ? 'is-above' : 'is-below'}`}
      style={{ left, top, width }}
      role="tooltip"
    >
      {entry && !entry.uncommitted ? (
        <>
          <span className="blame-avatar" aria-hidden="true">
            {(entry.author.trim().charAt(0) || '?').toUpperCase()}
          </span>
          <div className="blame-copy">
            <div className="blame-headline">
              <strong>{entry.author || t('blame.unknownAuthor')}</strong>
              <span className="blame-time">{formatBlameTime(entry.time, t)}</span>
            </div>
            <div className="blame-meta">
              <span className="blame-hash">{entry.commit.slice(0, 7)}</span>
              <span className="blame-summary">
                {entry.summary || t('blame.noSummary')}
              </span>
            </div>
          </div>
        </>
      ) : uncommitted ? (
        <>
          <span className="blame-avatar is-uncommitted" aria-hidden="true">
            •
          </span>
          <div className="blame-copy">
            <strong>{t('blame.uncommitted')}</strong>
            <span className="blame-hint">{t('blame.uncommittedHint')}</span>
          </div>
        </>
      ) : failed ? (
        <>
          <span className="blame-avatar is-unavailable" aria-hidden="true">
            ?
          </span>
          <div className="blame-copy">
            <strong>{t('blame.unavailable')}</strong>
            <span className="blame-hint">{t('blame.unavailableHint')}</span>
          </div>
        </>
      ) : (
        <>
          <span className="blame-avatar is-loading" aria-hidden="true">
            <LoaderCircle size={13} strokeWidth={2} className="is-spinning" />
          </span>
          <div className="blame-copy">
            <strong>{t('blame.loading')}</strong>
          </div>
        </>
      )}
    </div>
  )
}

function SplitDiff({
  file,
  baseBranch,
  appTheme,
  flashIndex,
  onLineEnter,
  onLineLeave,
}: {
  file: DiffFile
  baseBranch: string
  appTheme: ResolvedTheme
  flashIndex: number | null
  onLineEnter: (side: BlameSide, lineNo: number, event: ReactMouseEvent) => void
  onLineLeave: () => void
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
            return (
              <HunkHeader
                key={`hunk-${index}`}
                hunk={row.hunk}
                index={row.hunkIndex}
                flashing={flashIndex === row.hunkIndex}
              />
            )
          }

          const { left, right } = row.row
          const leftLineNo = left?.oldNo
          const rightLineNo = right?.newNo
          return (
            <div className="split-row" key={`row-${index}`}>
              <div
                className={`split-side side-left ${
                  left ? `is-${left.type}` : 'is-empty'
                }`}
                onMouseEnter={
                  leftLineNo
                    ? (event) => onLineEnter('old', leftLineNo, event)
                    : undefined
                }
                onMouseLeave={leftLineNo ? onLineLeave : undefined}
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
                onMouseEnter={
                  rightLineNo
                    ? (event) => onLineEnter('new', rightLineNo, event)
                    : undefined
                }
                onMouseLeave={rightLineNo ? onLineLeave : undefined}
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
  flashIndex,
  onLineEnter,
  onLineLeave,
}: {
  file: DiffFile
  appTheme: ResolvedTheme
  flashIndex: number | null
  onLineEnter: (side: BlameSide, lineNo: number, event: ReactMouseEvent) => void
  onLineLeave: () => void
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
          <HunkHeader
            hunk={hunk}
            index={hunkIndex}
            flashing={flashIndex === hunkIndex}
          />
          {hunk.lines.map((line, lineIndex) => {
            const side: BlameSide = line.type === 'del' ? 'old' : 'new'
            const lineNo =
              side === 'old' ? line.oldNo : (line.newNo ?? line.oldNo)
            return (
              <div
                className={`unified-row is-${line.type}`}
                key={`unified-line-${hunkIndex}-${lineIndex}`}
                onMouseEnter={
                  lineNo
                    ? (event) => onLineEnter(side, lineNo, event)
                    : undefined
                }
                onMouseLeave={lineNo ? onLineLeave : undefined}
              >
                <span className="line-number">{line.oldNo ?? ''}</span>
                <span className="line-number">{line.newNo ?? ''}</span>
                <span className="line-marker">
                  {line.type === 'add' ? '+' : line.type === 'del' ? '−' : ''}
                </span>
                <LineCode line={line} language={file.language} appTheme={appTheme} />
              </div>
            )
          })}
        </div>
      ))}
    </div>
  )
}

export function DiffViewer({
  file,
  mode,
  baseBranch,
  repositoryPath,
  appTheme,
  fileIndex,
  fileCount,
  onModeChange,
  onToast,
}: DiffViewerProps) {
  const { t } = useI18n()
  const [copied, setCopied] = useState(false)
  const [activeHunk, setActiveHunk] = useState(0)
  const [flashHunk, setFlashHunk] = useState<number | null>(null)
  const [blameHover, setBlameHover] = useState<BlameHover | null>(null)
  const [blameEntries, setBlameEntries] = useState<Record<string, BlameLine>>({})
  const [blameFailures, setBlameFailures] = useState<Record<BlameSide, boolean>>({
    old: false,
    new: false,
  })
  const [blameReady, setBlameReady] = useState<Record<BlameSide, boolean>>({
    old: false,
    new: false,
  })
  const scrollRef = useRef<HTMLDivElement>(null)
  const scrollFrame = useRef<number | null>(null)
  const flashTimer = useRef<number | null>(null)
  const hoverTimer = useRef<number | null>(null)
  const stats = useMemo(() => fileStats(file), [file])
  const { name, dir } = pathParts(file.path)
  const language = languageLabels[file.language] ?? file.language.toUpperCase()
  const hunkCount = file.hunks.length

  const blameLines = useMemo(() => {
    const oldLines = new Set<number>()
    const newLines = new Set<number>()
    for (const hunk of file.hunks) {
      for (const line of hunk.lines) {
        if (line.oldNo) oldLines.add(line.oldNo)
        if (line.newNo) newLines.add(line.newNo)
      }
    }
    return { old: Array.from(oldLines), new: Array.from(newLines) }
  }, [file])

  useEffect(() => {
    setBlameEntries({})
    setBlameFailures({ old: false, new: false })
    setBlameReady({ old: false, new: false })
    setBlameHover(null)
    if (!repositoryPath) return

    let cancelled = false
    const load = async (side: BlameSide, lineNumbers: number[]) => {
      if (lineNumbers.length === 0) {
        setBlameReady((current) => ({ ...current, [side]: true }))
        return
      }
      try {
        const entries = await fetchBlame(
          repositoryPath,
          file.path,
          lineNumbers,
          side === 'old' ? baseBranch : undefined,
        )
        if (cancelled) return
        setBlameEntries((current) => {
          const next = { ...current }
          for (const entry of entries) {
            next[`${side}:${entry.line}`] = entry
          }
          return next
        })
      } catch {
        if (cancelled) return
        setBlameFailures((current) => ({ ...current, [side]: true }))
      } finally {
        if (!cancelled) {
          setBlameReady((current) => ({ ...current, [side]: true }))
        }
      }
    }

    void load('new', blameLines.new)
    void load('old', blameLines.old)

    return () => {
      cancelled = true
    }
  }, [repositoryPath, file.path, baseBranch, blameLines])

  useEffect(() => {
    if (!copied) return
    const timer = window.setTimeout(() => setCopied(false), 1800)
    return () => window.clearTimeout(timer)
  }, [copied])

  const copyPath = async () => {
    try {
      await copyText(file.path)
      setCopied(true)
      onToast('diff.pathCopied')
    } catch {
      onToast('diff.copyFailed')
    }
  }

  const readActiveHunk = () => {
    const scroller = scrollRef.current
    if (!scroller) return 0
    const anchors = Array.from(
      scroller.querySelectorAll<HTMLElement>('[data-hunk-index]'),
    )
    if (anchors.length === 0) return 0

    const scrollerTop = scroller.getBoundingClientRect().top
    let current = 0
    anchors.forEach((anchor, index) => {
      // The column header is sticky, so it covers the top of the scrollport.
      if (anchor.getBoundingClientRect().top - scrollerTop <= 38) {
        current = index
      }
    })
    if (scroller.scrollTop + scroller.clientHeight >= scroller.scrollHeight - 2) {
      current = anchors.length - 1
    }
    return current
  }

  const handleDiffScroll = () => {
    if (scrollFrame.current !== null) return
    scrollFrame.current = window.requestAnimationFrame(() => {
      scrollFrame.current = null
      setActiveHunk(readActiveHunk())
      if (hoverTimer.current !== null) {
        window.clearTimeout(hoverTimer.current)
        hoverTimer.current = null
      }
      setBlameHover(null)
    })
  }

  const handleLineEnter = (
    side: BlameSide,
    lineNo: number,
    event: ReactMouseEvent,
  ) => {
    if (!repositoryPath) return
    const { clientX, clientY } = event
    if (hoverTimer.current !== null) {
      window.clearTimeout(hoverTimer.current)
    }
    // A short dwell keeps the card from flashing while the pointer crosses lines.
    hoverTimer.current = window.setTimeout(() => {
      hoverTimer.current = null
      setBlameHover({ side, lineNo, x: clientX, y: clientY })
    }, 110)
  }

  const handleLineLeave = () => {
    if (hoverTimer.current !== null) {
      window.clearTimeout(hoverTimer.current)
      hoverTimer.current = null
    }
    setBlameHover(null)
  }

  const jumpToHunk = (index: number) => {
    const scroller = scrollRef.current
    if (!scroller) return
    const anchor = scroller.querySelector<HTMLElement>(
      `[data-hunk-index="${index}"]`,
    )
    if (!anchor) return

    const top =
      anchor.getBoundingClientRect().top -
      scroller.getBoundingClientRect().top +
      scroller.scrollTop
    const reduceMotion = window.matchMedia('(prefers-reduced-motion: reduce)')
      .matches
    scroller.scrollTo({
      top: Math.max(0, top - 32),
      behavior: reduceMotion ? 'auto' : 'smooth',
    })
    setActiveHunk(index)
    setFlashHunk(index)

    if (flashTimer.current !== null) {
      window.clearTimeout(flashTimer.current)
    }
    flashTimer.current = window.setTimeout(() => {
      flashTimer.current = null
      setFlashHunk(null)
    }, 900)
  }

  const stepHunk = (offset: -1 | 1) => {
    if (hunkCount === 0) return
    const current = readActiveHunk()
    jumpToHunk(Math.min(hunkCount - 1, Math.max(0, current + offset)))
  }

  useEffect(() => {
    // Anchors are rebuilt when the layout switches between split and unified.
    setActiveHunk(readActiveHunk())
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [mode])

  useEffect(
    () => () => {
      if (scrollFrame.current !== null) {
        window.cancelAnimationFrame(scrollFrame.current)
      }
      if (flashTimer.current !== null) {
        window.clearTimeout(flashTimer.current)
      }
      if (hoverTimer.current !== null) {
        window.clearTimeout(hoverTimer.current)
      }
    },
    [],
  )

  const atFirstHunk = hunkCount === 0 || activeHunk === 0
  const atLastHunk = hunkCount === 0 || activeHunk >= hunkCount - 1
  const hoveredBlame = blameHover
    ? blameEntries[`${blameHover.side}:${blameHover.lineNo}`]
    : undefined
  const hoveredBlameFailed = blameHover
    ? blameFailures[blameHover.side] ||
      (blameReady[blameHover.side] && !hoveredBlame)
    : false

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

          <div className="file-nav" role="group" aria-label={t('diff.changeNav')}>
            <button
              data-testid="diff-prev-change"
              className="icon-button nav-button"
              onClick={() => stepHunk(-1)}
              disabled={atFirstHunk}
              title={t('diff.previousTitle')}
              aria-label={t('diff.previous')}
            >
              <ArrowUp size={14} strokeWidth={1.9} />
            </button>
            <button
              data-testid="diff-next-change"
              className="icon-button nav-button"
              onClick={() => stepHunk(1)}
              disabled={atLastHunk}
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

      <div
        className="diff-scroll"
        ref={scrollRef}
        onScroll={handleDiffScroll}
      >
        <div className="diff-surface">
          {mode === 'split' ? (
            <SplitDiff
              file={file}
              baseBranch={baseBranch}
              appTheme={appTheme}
              flashIndex={flashHunk}
              onLineEnter={handleLineEnter}
              onLineLeave={handleLineLeave}
            />
          ) : (
            <UnifiedDiff
              file={file}
              appTheme={appTheme}
              flashIndex={flashHunk}
              onLineEnter={handleLineEnter}
              onLineLeave={handleLineLeave}
            />
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

      {blameHover && (
        <BlamePopover
          hover={blameHover}
          entry={hoveredBlame}
          failed={hoveredBlameFailed}
        />
      )}
    </section>
  )
}

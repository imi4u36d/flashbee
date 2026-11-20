import { useEffect, useMemo, useRef, useState } from 'react'
import {
  ArrowLeftToLine,
  ArrowRightToLine,
  Check,
  CheckCircle2,
  ChevronDown,
  CircleAlert,
  Cloud,
  Columns2,
  FileText,
  FileWarning,
  GitMerge,
  LoaderCircle,
  Rows3,
  Sparkles,
  X,
} from 'lucide-react'
import { languageLabels, pathParts } from '../data/projects'
import { useI18n, type TranslationKey } from '../lib/i18n'
import { highlightLine } from '../lib/highlight'
import {
  fetchConflict,
  resolveConflictSide,
  saveConflictResolution,
  type ConflictDetail,
} from '../lib/repositoryApi'
import type { ResolvedTheme } from '../lib/settings'
import type {
  ConflictEntry,
  ConflictSegment,
  ConflictSide,
  DiffMode,
  Project,
} from '../types'

interface ConflictResolverProps {
  project: Project
  conflicts: ConflictEntry[]
  mergeInProgress: boolean
  mode: DiffMode
  appTheme: ResolvedTheme
  onModeChange: (mode: DiffMode) => void
  onToast: (key: TranslationKey, vars?: Record<string, string | number>) => void
  onResolved: () => Promise<void>
  onCompleteMerge: () => void
  onAbortMerge: () => void
  onClose: () => void
}

interface NumberedSegment {
  segment: ConflictSegment
  oursStart: number
  theirsStart: number
}

function conflictKindLabel(kind: string) {
  switch (kind) {
    case 'bothAdded':
      return 'conflict.kindBothAdded' as const
    case 'deletedByUs':
      return 'conflict.kindDeletedByUs' as const
    case 'deletedByThem':
      return 'conflict.kindDeletedByThem' as const
    case 'addedByUs':
      return 'conflict.kindAddedByUs' as const
    case 'addedByThem':
      return 'conflict.kindAddedByThem' as const
    default:
      return 'conflict.kindBothModified' as const
  }
}

export function ConflictResolver({
  project,
  conflicts,
  mergeInProgress,
  mode,
  appTheme,
  onModeChange,
  onToast,
  onResolved,
  onCompleteMerge,
  onAbortMerge,
  onClose,
}: ConflictResolverProps) {
  const { t } = useI18n()
  const [selectedPath, setSelectedPath] = useState(conflicts[0]?.path ?? '')
  const [detail, setDetail] = useState<ConflictDetail | null>(null)
  const [resolutions, setResolutions] = useState<Record<number, ConflictSide>>({})
  const [loading, setLoading] = useState(false)
  const [saving, setSaving] = useState(false)
  const [loadError, setLoadError] = useState(false)
  const [actionError, setActionError] = useState<string | null>(null)
  const [reloadToken, setReloadToken] = useState(0)
  const [fileMenuOpen, setFileMenuOpen] = useState(false)
  const fileMenuRef = useRef<HTMLDivElement>(null)

  useEffect(() => {
    if (!fileMenuOpen) return

    const closeOnOutsideClick = (event: PointerEvent) => {
      if (!fileMenuRef.current?.contains(event.target as Node)) {
        setFileMenuOpen(false)
      }
    }
    const closeOnEscape = (event: KeyboardEvent) => {
      if (event.key === 'Escape') {
        event.stopPropagation()
        setFileMenuOpen(false)
      }
    }

    document.addEventListener('pointerdown', closeOnOutsideClick)
    window.addEventListener('keydown', closeOnEscape, true)
    return () => {
      document.removeEventListener('pointerdown', closeOnOutsideClick)
      window.removeEventListener('keydown', closeOnEscape, true)
    }
  }, [fileMenuOpen])

  const selectedEntry =
    conflicts.find((conflict) => conflict.path === selectedPath) ?? conflicts[0]

  const selectedPathValue = selectedEntry?.path ?? ''

  useEffect(() => {
    if (conflicts.length === 0) return
    if (!conflicts.some((conflict) => conflict.path === selectedPath)) {
      // oxlint-disable-next-line react/set-state-in-effect
      setSelectedPath(conflicts[0].path)
    }
  }, [conflicts, selectedPath])

  useEffect(() => {
    if (!selectedPathValue) {
      // oxlint-disable-next-line react/set-state-in-effect
      setDetail(null)
      return
    }

    let cancelled = false
    setLoading(true)
    setLoadError(false)
    setActionError(null)
    setDetail(null)
    setResolutions({})

    fetchConflict(project.path, selectedPathValue)
      .then((next) => {
        if (!cancelled) setDetail(next)
      })
      .catch(() => {
        if (!cancelled) setLoadError(true)
      })
      .finally(() => {
        if (!cancelled) setLoading(false)
      })

    return () => {
      cancelled = true
    }
  }, [project.path, selectedPathValue, reloadToken])

  const numberedSegments = useMemo<NumberedSegment[]>(() => {
    if (!detail) return []
    const numbered = detail.segments.reduce<{
      entries: NumberedSegment[]
      ours: number
      theirs: number
    }>(
      (state, segment) => {
        const common = segment.kind === 'common'
        return {
          entries: [
            ...state.entries,
            {
              segment,
              oursStart: state.ours,
              theirsStart: state.theirs,
            },
          ],
          ours: state.ours + (common ? segment.lines.length : segment.ours.length),
          theirs:
            state.theirs +
            (common ? segment.lines.length : segment.theirs.length),
        }
      },
      { entries: [], ours: 1, theirs: 1 },
    )
    return numbered.entries
  }, [detail])

  const conflictNumbers = useMemo(() => {
    const numbered = numberedSegments.reduce<{ numbers: number[]; count: number }>(
      (state, entry) => {
        if (entry.segment.kind !== 'conflict') {
          return { numbers: [...state.numbers, 0], count: state.count }
        }
        return {
          numbers: [...state.numbers, state.count + 1],
          count: state.count + 1,
        }
      },
      { numbers: [], count: 0 },
    )
    return numbered.numbers
  }, [numberedSegments])

  const conflictIndexes = useMemo(
    () =>
      numberedSegments
        .map((entry, index) => (entry.segment.kind === 'conflict' ? index : -1))
        .filter((index) => index >= 0),
    [numberedSegments],
  )

  const unresolved = conflictIndexes.filter((index) => !resolutions[index])
  const resolvedCount = conflictIndexes.length - unresolved.length
  const autoCount = conflictIndexes.filter((index) => {
    const segment = numberedSegments[index].segment
    return segment.kind === 'conflict' && segment.suggestion !== null
  }).length

  const { name, dir } = detail ? pathParts(detail.path) : pathParts('')
  const languageLabel = detail
    ? (languageLabels[detail.language] ?? detail.language.toUpperCase())
    : ''
  const oursLabel = project.currentBranch ?? project.branch

  const choose = (index: number, side: ConflictSide) => {
    setResolutions((current) => ({ ...current, [index]: side }))
    setActionError(null)
  }

  const clearChoice = (index: number) => {
    setResolutions((current) => {
      const next = { ...current }
      delete next[index]
      return next
    })
  }

  const chooseAll = (side: ConflictSide) => {
    const next: Record<number, ConflictSide> = {}
    for (const index of conflictIndexes) next[index] = side
    setResolutions(next)
    setActionError(null)
  }

  const quickMerge = () => {
    if (!detail) return
    const next = { ...resolutions }
    let applied = 0
    detail.segments.forEach((segment, index) => {
      if (
        segment.kind === 'conflict' &&
        segment.suggestion &&
        !next[index]
      ) {
        next[index] = segment.suggestion
        applied += 1
      }
    })
    setResolutions(next)
    setActionError(null)
    if (applied === 0) {
      onToast('conflict.nothingToMerge')
    } else {
      onToast('conflict.quickMerged', { count: applied })
    }
  }

  const save = async () => {
    if (!detail || saving || detail.binary || unresolved.length > 0) return

    const lines: string[] = []
    const complete = detail.segments.every((segment, index) => {
      if (segment.kind === 'common') {
        lines.push(...segment.lines)
        return true
      }
      const side = resolutions[index]
      if (!side) return false
      lines.push(...(side === 'ours' ? segment.ours : segment.theirs))
      return true
    })
    if (!complete) return

    const content =
      lines.length === 0
        ? ''
        : `${lines.join('\n')}${detail.trailingNewline ? '\n' : ''}`

    setSaving(true)
    setActionError(null)
    try {
      await saveConflictResolution(project.path, detail.path, content)
      onToast('conflict.saved', { name })
      await onResolved()
    } catch {
      setActionError(t('conflict.saveFailed'))
    } finally {
      setSaving(false)
    }
  }

  const applyWholeSide = async (side: ConflictSide) => {
    if (!detail || saving) return
    setSaving(true)
    setActionError(null)
    try {
      await resolveConflictSide(project.path, detail.path, side)
      onToast(
        side === 'ours' ? 'conflict.oursFileUsed' : 'conflict.theirsFileUsed',
        { name },
      )
      await onResolved()
    } catch {
      setActionError(t('conflict.actionFailed'))
    } finally {
      setSaving(false)
    }
  }

  const renderCodeLine = (
    lineNo: number,
    marker: string,
    content: string,
    className: string,
    key: string,
  ) => (
    <div className={`conflict-code-row ${className}`} key={key}>
      <span className="line-number">{lineNo}</span>
      <span className="line-marker">{marker}</span>
      <span
        className="line-code"
        dangerouslySetInnerHTML={{
          __html: highlightLine(content, detail?.language ?? 'text', appTheme),
        }}
      />
    </div>
  )

  const renderSplitCommon = (entry: NumberedSegment, index: number) => {
    if (entry.segment.kind !== 'common') return null
    return (
      <div className="conflict-common" key={`common-${index}`}>
        {entry.segment.lines.map((line, lineIndex) => (
          <div className="split-row conflict-common-row" key={`common-${index}-${lineIndex}`}>
            <div className="split-side side-left is-context">
              <span className="line-number">{entry.oursStart + lineIndex}</span>
              <span className="line-marker" />
              <span
                className="line-code"
                dangerouslySetInnerHTML={{
                  __html: highlightLine(line, detail?.language ?? 'text', appTheme),
                }}
              />
            </div>
            <div className="split-side side-right is-context">
              <span className="line-number">{entry.theirsStart + lineIndex}</span>
              <span className="line-marker" />
              <span
                className="line-code"
                dangerouslySetInnerHTML={{
                  __html: highlightLine(line, detail?.language ?? 'text', appTheme),
                }}
              />
            </div>
          </div>
        ))}
      </div>
    )
  }

  const renderSplitConflict = (entry: NumberedSegment, index: number) => {
    if (entry.segment.kind !== 'conflict') return null
    const segment = entry.segment
    const choice = resolutions[index]
    const blockNumber = conflictNumbers[index]

    return (
      <div
        className={`conflict-block ${choice ? `is-chosen-${choice}` : ''}`}
        key={`conflict-${index}`}
      >
        <div className="conflict-block-bar">
          <span className="conflict-block-title">
            <span className="conflict-index">
              <CircleAlert size={11} strokeWidth={2.2} />
              {t('conflict.block', { index: blockNumber })}
            </span>
            {segment.suggestion && (
              <span className="conflict-auto-hint">
                <Sparkles size={10} strokeWidth={2} />
                {t('conflict.autoHint')}
              </span>
            )}
            {choice && (
              <span className="conflict-choice-hint">
                <Check size={10} strokeWidth={2.6} />
                {t(choice === 'ours' ? 'conflict.usingOurs' : 'conflict.usingTheirs')}
              </span>
            )}
          </span>
          <span className="conflict-block-actions">
            <button
              type="button"
              className={`conflict-side-button ${
                choice === 'ours' ? 'is-active' : ''
              }`}
              onClick={() => choose(index, 'ours')}
            >
              <ArrowLeftToLine size={11} strokeWidth={2} />
              {t('conflict.left')}
            </button>
            <button
              type="button"
              className={`conflict-side-button ${
                choice === 'theirs' ? 'is-active' : ''
              }`}
              onClick={() => choose(index, 'theirs')}
            >
              <ArrowRightToLine size={11} strokeWidth={2} />
              {t('conflict.right')}
            </button>
            {choice && (
              <button
                type="button"
                className="conflict-side-button is-clear"
                onClick={() => clearChoice(index)}
                title={t('conflict.clearChoice')}
                aria-label={t('conflict.clearChoice')}
              >
                <X size={11} strokeWidth={2} />
              </button>
            )}
          </span>
        </div>

        <div className="conflict-split">
          <div
            className={`conflict-side is-ours ${
              choice === 'ours' ? 'is-chosen' : ''
            } ${choice === 'theirs' ? 'is-dimmed' : ''}`}
          >
            <div className="conflict-side-label">
              <span>{t('conflict.ours')}</span>
              <span>{oursLabel}</span>
            </div>
            {segment.ours.length === 0 ? (
              <div className="conflict-empty-side">
                {t('conflict.emptySide')}
              </div>
            ) : (
              segment.ours.map((line, lineIndex) =>
                renderCodeLine(
                  entry.oursStart + lineIndex,
                  '−',
                  line,
                  'is-ours',
                  `ours-${index}-${lineIndex}`,
                ),
              )
            )}
          </div>

          <div
            className={`conflict-side is-theirs ${
              choice === 'theirs' ? 'is-chosen' : ''
            } ${choice === 'ours' ? 'is-dimmed' : ''}`}
          >
            <div className="conflict-side-label">
              <span>{t('conflict.theirs')}</span>
              <span>{project.upstream ?? t('conflict.remote')}</span>
            </div>
            {segment.theirs.length === 0 ? (
              <div className="conflict-empty-side">
                {t('conflict.emptySide')}
              </div>
            ) : (
              segment.theirs.map((line, lineIndex) =>
                renderCodeLine(
                  entry.theirsStart + lineIndex,
                  '+',
                  line,
                  'is-theirs',
                  `theirs-${index}-${lineIndex}`,
                ),
              )
            )}
          </div>
        </div>
      </div>
    )
  }

  const renderUnified = () => (
    <div className="conflict-unified">
      <div className="unified-column-header conflict-unified-header">
        <span className="unified-gutter-label">{t('conflict.left')}</span>
        <span className="unified-gutter-label">{t('conflict.right')}</span>
        <span />
        <span className="column-label">{t('conflict.unifiedHeader')}</span>
      </div>
      {numberedSegments.map((entry, index) => {
        if (entry.segment.kind === 'common') {
          return (
            <div className="conflict-common" key={`unified-common-${index}`}>
              {entry.segment.lines.map((line, lineIndex) => (
                <div
                  className="unified-row is-context"
                  key={`unified-common-${index}-${lineIndex}`}
                >
                  <span className="line-number">{entry.oursStart + lineIndex}</span>
                  <span className="line-number">{entry.theirsStart + lineIndex}</span>
                  <span className="line-marker" />
                  <span
                    className="line-code"
                    dangerouslySetInnerHTML={{
                      __html: highlightLine(
                        line,
                        detail?.language ?? 'text',
                        appTheme,
                      ),
                    }}
                  />
                </div>
              ))}
            </div>
          )
        }

        const segment = entry.segment
        const choice = resolutions[index]
        return (
          <div
            className={`conflict-block is-unified ${
              choice ? `is-chosen-${choice}` : ''
            }`}
            key={`unified-conflict-${index}`}
          >
            <div className="conflict-block-bar">
              <span className="conflict-block-title">
                <span className="conflict-index">
                  <CircleAlert size={11} strokeWidth={2.2} />
                  {t('conflict.block', { index: conflictNumbers[index] })}
                </span>
                {segment.suggestion && (
                  <span className="conflict-auto-hint">
                    <Sparkles size={10} strokeWidth={2} />
                    {t('conflict.autoHint')}
                  </span>
                )}
                {choice && (
                  <span className="conflict-choice-hint">
                    <Check size={10} strokeWidth={2.6} />
                    {t(
                      choice === 'ours'
                        ? 'conflict.usingOurs'
                        : 'conflict.usingTheirs',
                    )}
                  </span>
                )}
              </span>
              <span className="conflict-block-actions">
                <button
                  type="button"
                  className={`conflict-side-button ${
                    choice === 'ours' ? 'is-active' : ''
                  }`}
                  onClick={() => choose(index, 'ours')}
                >
                  <ArrowLeftToLine size={11} strokeWidth={2} />
                  {t('conflict.left')}
                </button>
                <button
                  type="button"
                  className={`conflict-side-button ${
                    choice === 'theirs' ? 'is-active' : ''
                  }`}
                  onClick={() => choose(index, 'theirs')}
                >
                  <ArrowRightToLine size={11} strokeWidth={2} />
                  {t('conflict.right')}
                </button>
                {choice && (
                  <button
                    type="button"
                    className="conflict-side-button is-clear"
                    onClick={() => clearChoice(index)}
                    title={t('conflict.clearChoice')}
                    aria-label={t('conflict.clearChoice')}
                  >
                    <X size={11} strokeWidth={2} />
                  </button>
                )}
              </span>
            </div>

            <div
              className={`conflict-unified-side is-ours ${
                choice === 'ours' ? 'is-chosen' : ''
              } ${choice === 'theirs' ? 'is-dimmed' : ''}`}
            >
              <div className="conflict-side-label">
                <span>{t('conflict.ours')}</span>
                <span>{oursLabel}</span>
              </div>
              {segment.ours.length === 0 ? (
                <div className="conflict-empty-side">{t('conflict.emptySide')}</div>
              ) : (
                segment.ours.map((line, lineIndex) => (
                  <div
                    className="conflict-code-row is-ours"
                    key={`unified-ours-${index}-${lineIndex}`}
                  >
                    <span className="line-number">
                      {entry.oursStart + lineIndex}
                    </span>
                    <span className="line-number" />
                    <span className="line-marker">−</span>
                    <span
                      className="line-code"
                      dangerouslySetInnerHTML={{
                        __html: highlightLine(
                          line,
                          detail?.language ?? 'text',
                          appTheme,
                        ),
                      }}
                    />
                  </div>
                ))
              )}
            </div>

            <div
              className={`conflict-unified-side is-theirs ${
                choice === 'theirs' ? 'is-chosen' : ''
              } ${choice === 'ours' ? 'is-dimmed' : ''}`}
            >
              <div className="conflict-side-label">
                <span>{t('conflict.theirs')}</span>
                <span>{project.upstream ?? t('conflict.remote')}</span>
              </div>
              {segment.theirs.length === 0 ? (
                <div className="conflict-empty-side">{t('conflict.emptySide')}</div>
              ) : (
                segment.theirs.map((line, lineIndex) => (
                  <div
                    className="conflict-code-row is-theirs"
                    key={`unified-theirs-${index}-${lineIndex}`}
                  >
                    <span className="line-number" />
                    <span className="line-number">
                      {entry.theirsStart + lineIndex}
                    </span>
                    <span className="line-marker">+</span>
                    <span
                      className="line-code"
                      dangerouslySetInnerHTML={{
                        __html: highlightLine(
                          line,
                          detail?.language ?? 'text',
                          appTheme,
                        ),
                      }}
                    />
                  </div>
                ))
              )}
            </div>
          </div>
        )
      })}
    </div>
  )

  const mergeReady = conflicts.length === 0 && mergeInProgress
  const fileLevelDecision =
    detail !== null &&
    (detail.binary || !detail.oursExists || !detail.theirsExists)
  const bothDeleted =
    detail !== null && !detail.oursExists && !detail.theirsExists
  const fileLevelTitle = detail?.binary
    ? t('conflict.binaryTitle')
    : bothDeleted
      ? t('conflict.bothDeletedTitle')
      : t('conflict.fileLevelTitle')
  const fileLevelDescription = detail?.binary
    ? t('conflict.binaryDescription')
    : bothDeleted
      ? t('conflict.bothDeletedDescription')
      : t('conflict.fileLevelDescription')

  return (
    <section className="conflict-viewer">
      <header className="file-toolbar conflict-toolbar">
        <div className="file-heading">
          <span className="file-status conflict-status">
            {mergeInProgress ? t('conflict.mergeBadge') : t('conflict.badge')}
          </span>
          <div className="file-heading-copy">
            <div className="file-title-row">
              <h1>{mergeReady ? t('conflict.mergeTitle') : name || t('conflict.title')}</h1>
              {detail && !mergeReady && <span className="language-chip">{languageLabel}</span>}
              {conflicts.length > 0 && (
                <span className="conflict-count-chip">
                  {t('conflict.countChip', { count: conflicts.length })}
                </span>
              )}
            </div>
            <div className="file-path-line">
              {mergeReady ? (
                <span>{t('conflict.mergeReadyPath')}</span>
              ) : (
                <>
                  <span>{dir || '.'}</span>
                  <span className="path-dot">·</span>
                  <span>
                    {selectedEntry
                      ? t(conflictKindLabel(selectedEntry.kind))
                      : t('conflict.title')}
                  </span>
                  {conflictIndexes.length > 0 && (
                    <>
                      <span className="path-dot">·</span>
                      <span>
                        {t('conflict.progress', {
                          resolved: resolvedCount,
                          total: conflictIndexes.length,
                        })}
                      </span>
                    </>
                  )}
                </>
              )}
            </div>
          </div>
        </div>

        <div className="file-actions conflict-actions">
          {!fileLevelDecision && (
            <div className="view-switch" role="group" aria-label={t('diff.layout')}>
              <button
                className={mode === 'split' ? 'is-active' : ''}
                onClick={() => onModeChange('split')}
                title={t('diff.splitTitle')}
              >
                <Columns2 size={13} strokeWidth={1.9} />
                <span>{t('diff.split')}</span>
              </button>
              <button
                className={mode === 'unified' ? 'is-active' : ''}
                onClick={() => onModeChange('unified')}
                title={t('diff.inlineTitle')}
              >
                <Rows3 size={13} strokeWidth={1.9} />
                <span>{t('diff.unified')}</span>
              </button>
            </div>
          )}

          {!mergeReady && !fileLevelDecision && (
            <>
              <span className="action-divider" />
              <button
                type="button"
                className="conflict-tool-button"
                onClick={quickMerge}
                disabled={!detail || detail.binary || autoCount === 0}
                title={t('conflict.quickMergeHint')}
              >
                <Sparkles size={12} strokeWidth={1.9} />
                {t('conflict.quickMerge')}
              </button>
              <button
                type="button"
                className="conflict-tool-button"
                onClick={() => chooseAll('ours')}
                disabled={!detail || detail.binary || conflictIndexes.length === 0}
                title={t('conflict.selectAllOursHint')}
              >
                <ArrowLeftToLine size={12} strokeWidth={1.9} />
                {t('conflict.selectAllOurs')}
              </button>
              <button
                type="button"
                className="conflict-tool-button"
                onClick={() => chooseAll('theirs')}
                disabled={!detail || detail.binary || conflictIndexes.length === 0}
                title={t('conflict.selectAllTheirsHint')}
              >
                <ArrowRightToLine size={12} strokeWidth={1.9} />
                {t('conflict.selectAllTheirs')}
              </button>
              <div className="conflict-file-menu-wrap" ref={fileMenuRef}>
                <button
                  type="button"
                  className={`conflict-tool-button ${
                    fileMenuOpen ? 'is-active' : ''
                  }`}
                  onClick={() => setFileMenuOpen((open) => !open)}
                  disabled={!detail}
                  aria-haspopup="menu"
                  aria-expanded={fileMenuOpen}
                  title={t('conflict.wholeFileHint')}
                >
                  <FileText size={12} strokeWidth={1.9} />
                  {t('conflict.wholeFile')}
                  <ChevronDown size={11} strokeWidth={2} />
                </button>
                {fileMenuOpen && detail && (
                  <div className="conflict-file-menu" role="menu">
                    <button
                      type="button"
                      role="menuitem"
                      onClick={() => {
                        setFileMenuOpen(false)
                        void applyWholeSide('ours')
                      }}
                    >
                      <ArrowLeftToLine size={13} strokeWidth={1.9} />
                      <span>{t('conflict.wholeOurs')}</span>
                      <small>{oursLabel}</small>
                    </button>
                    <button
                      type="button"
                      role="menuitem"
                      onClick={() => {
                        setFileMenuOpen(false)
                        void applyWholeSide('theirs')
                      }}
                    >
                      <ArrowRightToLine size={13} strokeWidth={1.9} />
                      <span>{t('conflict.wholeTheirs')}</span>
                      <small>{project.upstream ?? t('conflict.remote')}</small>
                    </button>
                    <p className="conflict-file-menu-hint">
                      {t('conflict.wholeFileWarning')}
                    </p>
                  </div>
                )}
              </div>
              <button
                type="button"
                className="conflict-save-button"
                onClick={() => void save()}
                disabled={!detail || detail.binary || saving || unresolved.length > 0}
                title={
                  unresolved.length > 0
                    ? t('conflict.unresolvedHint', { count: unresolved.length })
                    : t('conflict.save')
                }
              >
                {saving ? (
                  <LoaderCircle size={12} strokeWidth={2} className="is-spinning" />
                ) : (
                  <CheckCircle2 size={12} strokeWidth={2} />
                )}
                {t('conflict.save')}
              </button>
            </>
          )}

          <span className="action-divider" />
          <button
            className="icon-button toolbar-button"
            onClick={onClose}
            title={t('conflict.later')}
            aria-label={t('conflict.later')}
          >
            <X size={14} strokeWidth={1.9} />
          </button>
        </div>
      </header>

      {mergeReady ? (
        <div className="conflict-complete">
          <span className="conflict-complete-icon">
            <GitMerge size={24} strokeWidth={1.6} />
          </span>
          <h2>{t('conflict.allResolved')}</h2>
          <p>{t('conflict.allResolvedDescription')}</p>
          <div className="conflict-complete-actions">
            <button
              type="button"
              className="conflict-save-button is-large"
              onClick={onCompleteMerge}
            >
              <GitMerge size={14} strokeWidth={2} />
              {t('conflict.completeMerge')}
            </button>
            <button
              type="button"
              className="conflict-tool-button is-large"
              onClick={onAbortMerge}
            >
              {t('conflict.abortMerge')}
            </button>
          </div>
        </div>
      ) : (
        <div className="conflict-body">
          <aside className="conflict-file-list">
            <div className="conflict-file-heading">
              <span>{t('conflict.files')}</span>
              <span className="conflict-file-count">{conflicts.length}</span>
            </div>
            <div className="conflict-file-scroll">
              {conflicts.map((conflict) => {
                const parts = pathParts(conflict.path)
                const active = selectedEntry?.path === conflict.path
                return (
                  <button
                    key={conflict.path}
                    type="button"
                    className={`conflict-file-row ${active ? 'is-active' : ''}`}
                    onClick={() => setSelectedPath(conflict.path)}
                    title={conflict.path}
                  >
                    <span className={`conflict-file-dot is-${conflict.kind}`} />
                    <span className="conflict-file-copy">
                      <strong>{parts.name}</strong>
                      <span>{parts.dir || '.'}</span>
                    </span>
                  </button>
                )
              })}
            </div>
            {mergeInProgress && (
              <div className="conflict-file-footer">
                <span className="conflict-merge-state">
                  <GitMerge size={11} strokeWidth={2} />
                  {t('conflict.mergeInProgress')}
                </span>
                <button
                  type="button"
                  className="conflict-abort-link"
                  onClick={onAbortMerge}
                >
                  {t('conflict.abortMerge')}
                </button>
              </div>
            )}
          </aside>

          <div className="conflict-main">
            {loading ? (
              <div className="conflict-loading" role="status">
                <LoaderCircle size={19} strokeWidth={2} className="is-spinning" />
                <span>{t('conflict.loading')}</span>
              </div>
            ) : loadError ? (
              <div className="conflict-error-panel" role="alert">
                <FileWarning size={22} strokeWidth={1.7} />
                <strong>{t('conflict.loadFailed')}</strong>
                <button
                  type="button"
                  className="conflict-tool-button"
                  onClick={() => setReloadToken((current) => current + 1)}
                >
                  {t('conflict.retry')}
                </button>
              </div>
            ) : fileLevelDecision && detail ? (
              <div className="conflict-binary">
                <FileWarning size={26} strokeWidth={1.6} />
                <h2>{fileLevelTitle}</h2>
                <p>{fileLevelDescription}</p>
                <div className="conflict-binary-actions">
                  {bothDeleted ? (
                    <button
                      type="button"
                      className="conflict-side-card is-ours"
                      onClick={() => void applyWholeSide('ours')}
                      disabled={saving}
                    >
                      <ArrowLeftToLine size={16} strokeWidth={1.9} />
                      <strong>{t('conflict.keepDeleted')}</strong>
                      <span>{oursLabel}</span>
                    </button>
                  ) : (
                    <>
                      <button
                        type="button"
                        className="conflict-side-card is-ours"
                        onClick={() => void applyWholeSide('ours')}
                        disabled={saving}
                      >
                        <ArrowLeftToLine size={16} strokeWidth={1.9} />
                        <strong>
                          {detail.oursExists
                            ? t('conflict.useOursFile')
                            : t('conflict.useOursDelete')}
                        </strong>
                        <span>{oursLabel}</span>
                      </button>
                      <button
                        type="button"
                        className="conflict-side-card is-theirs"
                        onClick={() => void applyWholeSide('theirs')}
                        disabled={saving}
                      >
                        <ArrowRightToLine size={16} strokeWidth={1.9} />
                        <strong>
                          {detail.theirsExists
                            ? t('conflict.useTheirsFile')
                            : t('conflict.useTheirsDelete')}
                        </strong>
                        <span>{project.upstream ?? t('conflict.remote')}</span>
                      </button>
                    </>
                  )}
                </div>
                {actionError && (
                  <div className="conflict-action-error" role="alert">
                    <CircleAlert size={12} strokeWidth={2} />
                    {actionError}
                  </div>
                )}
              </div>
            ) : detail ? (
              <>
                <div className="conflict-scroll">
                  <div className={`conflict-surface is-${mode}`}>
                    {mode === 'split' ? (
                      <>
                        <div className="conflict-column-header">
                          <div className="conflict-column-title is-ours">
                            <ArrowLeftToLine size={11} strokeWidth={2} />
                            <span className="column-label">{t('conflict.ours')}</span>
                            <span className="column-ref">{oursLabel}</span>
                          </div>
                          <div className="conflict-column-title is-theirs">
                            <span className="column-label">{t('conflict.theirs')}</span>
                            <span className="column-ref">
                              {project.upstream ?? t('conflict.remote')}
                            </span>
                            <Cloud size={11} strokeWidth={2} />
                          </div>
                        </div>
                        {numberedSegments.map((entry, index) =>
                          entry.segment.kind === 'common'
                            ? renderSplitCommon(entry, index)
                            : renderSplitConflict(entry, index),
                        )}
                      </>
                    ) : (
                      renderUnified()
                    )}
                  </div>
                </div>

                {actionError && (
                  <div className="conflict-action-error" role="alert">
                    <CircleAlert size={12} strokeWidth={2} />
                    {actionError}
                  </div>
                )}

                <footer className="conflict-footer">
                  <div className="footer-left">
                    <span className={`conflict-progress-pill ${
                      unresolved.length === 0 ? 'is-ready' : ''
                    }`}>
                      {unresolved.length === 0 ? (
                        <>
                          <CheckCircle2 size={12} strokeWidth={2.2} />
                          {t('conflict.allBlocksResolved')}
                        </>
                      ) : (
                        <>
                          <CircleAlert size={12} strokeWidth={2.2} />
                          {t('conflict.unresolved', { count: unresolved.length })}
                        </>
                      )}
                    </span>
                    {autoCount > 0 && (
                      <span className="conflict-auto-count">
                        {t('conflict.autoCount', { count: autoCount })}
                      </span>
                    )}
                  </div>
                  <div className="footer-right">
                    <span className="ref-compare">
                      {oursLabel} <span className="ref-arrow">↔</span>{' '}
                      {project.upstream ?? t('conflict.remote')}
                    </span>
                  </div>
                </footer>
              </>
            ) : null}
          </div>
        </div>
      )}
    </section>
  )
}

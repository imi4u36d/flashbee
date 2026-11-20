import { useEffect, useRef, useState, type FormEvent } from 'react'
import {
  ArrowUpFromLine,
  Check,
  ChevronDown,
  CircleAlert,
  GitCommitHorizontal,
  LoaderCircle,
  Minus,
  RefreshCw,
  Tag,
  TriangleAlert,
} from 'lucide-react'
import { DialogFrame } from './ManageDialogs'
import { fileStats, statusLabels } from '../data/projects'
import { useI18n, type Translate } from '../lib/i18n'
import { highlightLine } from '../lib/highlight'
import {
  RepositoryApiError,
  fetchCommitDetail,
  fetchPushInfo,
  pushRepository,
  type CommitDetail,
  type PushInfo,
} from '../lib/repositoryApi'
import type { ResolvedTheme } from '../lib/settings'
import type { DiffFile } from '../types'

const dateFormatters = new Map<string, Intl.DateTimeFormat>()

function formatTimestamp(time: number, locale: string) {
  if (!time) return ''
  let formatter = dateFormatters.get(locale)
  if (!formatter) {
    formatter = new Intl.DateTimeFormat(locale === 'zh' ? 'zh-CN' : 'en-US', {
      year: 'numeric',
      month: 'short',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit',
    })
    dateFormatters.set(locale, formatter)
  }
  return formatter.format(new Date(time * 1000))
}

function describeError(reason: unknown, t: Translate) {
  if (reason instanceof RepositoryApiError) {
    switch (reason.code) {
      case 'no_remote':
        return t('repo.noRemote')
      case 'detached_head':
        return t('repo.detachedHead')
      case 'bad_revision':
        return t('push.staleSelection')
      case 'bad_tag':
        return t('push.badTag')
      default:
        return reason.message || t('repo.actionFailed')
    }
  }
  if (reason instanceof Error && reason.message) return reason.message
  return t('repo.actionFailed')
}

function FileDiff({
  file,
  appTheme,
}: {
  file: DiffFile
  appTheme: ResolvedTheme
}) {
  const stats = fileStats(file)

  return (
    <section className="push-file">
      <header className="push-file-head">
        <span className={`file-status status-${file.status}`}>
          {statusLabels[file.status]}
        </span>
        <span className="push-file-path">{file.path}</span>
        <span className="push-file-stats">
          <span className="file-add">+{stats.additions}</span>
          <span className="file-del">−{stats.deletions}</span>
        </span>
      </header>
      <div className="diff-unified push-file-diff">
        {file.hunks.map((hunk, hunkIndex) => (
          <div className="unified-hunk" key={`${file.path}-${hunkIndex}`}>
            <div className="hunk-header">
              <span className="hunk-gutter" />
              <span className="hunk-text">{hunk.header}</span>
            </div>
            {hunk.lines.map((line, lineIndex) => (
              <div
                className={`unified-row is-${line.type}`}
                key={`${file.path}-${hunkIndex}-${lineIndex}`}
              >
                <span className="line-number">{line.oldNo ?? ''}</span>
                <span className="line-number">{line.newNo ?? ''}</span>
                <span className="line-marker">
                  {line.type === 'add' ? '+' : line.type === 'del' ? '−' : ''}
                </span>
                <span
                  className="line-code"
                  dangerouslySetInnerHTML={{
                    __html: highlightLine(line.content, file.language, appTheme),
                  }}
                />
              </div>
            ))}
          </div>
        ))}
      </div>
    </section>
  )
}

function CommitChanges({
  detail,
  appTheme,
  locale,
  excluded,
}: {
  detail: CommitDetail
  appTheme: ResolvedTheme
  locale: string
  excluded: boolean
}) {
  const { t } = useI18n()
  const totals = detail.files.reduce(
    (sum, file) => {
      const stats = fileStats(file)
      sum.additions += stats.additions
      sum.deletions += stats.deletions
      return sum
    },
    { additions: 0, deletions: 0 },
  )

  return (
    <div className="push-detail-scroll">
      <header className="push-commit-head">
        <h3>{detail.subject || t('push.noSubject')}</h3>
        {detail.body && <p className="push-commit-message">{detail.body}</p>}
        <div className="push-commit-meta">
          <span className="push-commit-author">{detail.author}</span>
          <span>{formatTimestamp(detail.time, locale)}</span>
          <span className="push-commit-hash">{detail.shortHash}</span>
          {excluded && (
            <span className="push-excluded-chip">{t('push.excluded')}</span>
          )}
        </div>
        <div className="push-commit-summary">
          <span>
            {t('push.filesChanged', {
              count: detail.fileCount,
            })}
          </span>
          <span className="file-add">+{totals.additions}</span>
          <span className="file-del">−{totals.deletions}</span>
        </div>
      </header>

      {detail.files.length === 0 ? (
        <div className="push-detail-note">{t('push.noFiles')}</div>
      ) : (
        detail.files.map((file) => (
          <FileDiff key={file.path} file={file} appTheme={appTheme} />
        ))
      )}

      {detail.truncated && (
        <p className="push-detail-note">
          {t('push.filesTruncated', { count: detail.files.length })}
        </p>
      )}
    </div>
  )
}

interface PushDialogProps {
  projectName: string
  repositoryPath: string
  appTheme: ResolvedTheme
  onPushed: () => void
  onClose: () => void
}

export function PushDialog({
  projectName,
  repositoryPath,
  appTheme,
  onPushed,
  onClose,
}: PushDialogProps) {
  const { t, locale } = useI18n()
  const [info, setInfo] = useState<PushInfo | null>(null)
  const [loading, setLoading] = useState(true)
  const [loadError, setLoadError] = useState<string | null>(null)
  const [reloadToken, setReloadToken] = useState(0)
  const [tipIndex, setTipIndex] = useState(0)
  const [selectedHash, setSelectedHash] = useState<string | null>(null)
  const [detail, setDetail] = useState<CommitDetail | null>(null)
  const [detailLoading, setDetailLoading] = useState(false)
  const [detailError, setDetailError] = useState<string | null>(null)
  const [detailToken, setDetailToken] = useState(0)
  const [pushTags, setPushTags] = useState(false)
  const [tagChoice, setTagChoice] = useState('all')
  const [tagMenuOpen, setTagMenuOpen] = useState(false)
  const [force, setForce] = useState(false)
  const [forceMenuOpen, setForceMenuOpen] = useState(false)
  const [pushing, setPushing] = useState(false)
  const [pushError, setPushError] = useState<string | null>(null)
  const detailCache = useRef(new Map<string, CommitDetail>())
  const tagMenuRef = useRef<HTMLDivElement>(null)
  const submitMenuRef = useRef<HTMLDivElement>(null)

  useEffect(() => {
    let cancelled = false
    fetchPushInfo(repositoryPath)
      .then((next) => {
        if (cancelled) return
        setInfo(next)
        setTipIndex(0)
        setSelectedHash(null)
        setDetail(null)
        setDetailLoading(false)
        setTagChoice('all')
      })
      .catch((reason) => {
        if (cancelled) return
        setInfo(null)
        setLoadError(describeError(reason, t))
      })
      .finally(() => {
        if (!cancelled) setLoading(false)
      })
    return () => {
      cancelled = true
    }
  }, [repositoryPath, reloadToken, t])

  useEffect(() => {
    if (!selectedHash || detailCache.current.has(selectedHash)) return

    let cancelled = false
    fetchCommitDetail(repositoryPath, selectedHash)
      .then((next) => {
        if (cancelled) return
        detailCache.current.set(selectedHash, next)
        setDetail(next)
        setDetailError(null)
      })
      .catch((reason) => {
        if (!cancelled) setDetailError(describeError(reason, t))
      })
      .finally(() => {
        if (!cancelled) setDetailLoading(false)
      })
    return () => {
      cancelled = true
    }
  }, [repositoryPath, selectedHash, detailToken, t])

  useEffect(() => {
    if (!tagMenuOpen && !forceMenuOpen) return

    const closeOnOutsideClick = (event: PointerEvent) => {
      if (tagMenuOpen && !tagMenuRef.current?.contains(event.target as Node)) {
        setTagMenuOpen(false)
      }
      if (
        forceMenuOpen &&
        !submitMenuRef.current?.contains(event.target as Node)
      ) {
        setForceMenuOpen(false)
      }
    }
    const closeOnEscape = (event: KeyboardEvent) => {
      if (event.key !== 'Escape') return
      event.preventDefault()
      event.stopPropagation()
      setTagMenuOpen(false)
      setForceMenuOpen(false)
    }

    document.addEventListener('pointerdown', closeOnOutsideClick)
    window.addEventListener('keydown', closeOnEscape, true)
    return () => {
      document.removeEventListener('pointerdown', closeOnOutsideClick)
      window.removeEventListener('keydown', closeOnEscape, true)
    }
  }, [tagMenuOpen, forceMenuOpen])

  const commits = info?.commits ?? []
  const selectedCount = Math.max(0, commits.length - tipIndex)
  const allSelected = commits.length > 0 && tipIndex === 0
  const someSelected = selectedCount > 0 && !allSelected
  const tagsAvailable = (info?.tags.length ?? 0) > 0
  const targetRef = info ? `${info.remote}/${info.remoteBranch}` : ''
  const canPush =
    Boolean(info) &&
    !loading &&
    !pushing &&
    (selectedCount > 0 || (pushTags && tagsAvailable))

  const toggleCommit = (index: number) => {
    const checked = index >= tipIndex
    setTipIndex(checked ? index + 1 : index)
    setPushError(null)
  }

  const toggleAll = () => {
    setTipIndex(allSelected ? commits.length : 0)
    setPushError(null)
  }

  const refreshInfo = () => {
    setLoading(true)
    setInfo(null)
    setLoadError(null)
    setPushError(null)
    setTipIndex(0)
    setSelectedHash(null)
    setDetail(null)
    setDetailLoading(false)
    setReloadToken((current) => current + 1)
  }

  const openCommit = (hash: string) => {
    setSelectedHash(hash)
    setDetailError(null)
    const cached = detailCache.current.get(hash)
    if (cached) {
      setDetail(cached)
      setDetailLoading(false)
      return
    }
    setDetail(null)
    setDetailLoading(true)
  }

  const retryDetail = () => {
    setDetail(null)
    setDetailError(null)
    setDetailLoading(true)
    setDetailToken((current) => current + 1)
  }

  const submit = async (event: FormEvent) => {
    event.preventDefault()
    if (!info || !canPush) return

    setPushing(true)
    setPushError(null)
    const skipCommits = selectedCount === 0
    try {
      await pushRepository(repositoryPath, {
        commit:
          skipCommits || selectedCount === commits.length
            ? null
            : commits[tipIndex]?.hash ?? null,
        force,
        pushTags: pushTags && tagsAvailable,
        skipCommits,
        tag: pushTags && tagChoice !== 'all' ? tagChoice : null,
      })
      onPushed()
    } catch (reason) {
      setPushError(describeError(reason, t))
    } finally {
      setPushing(false)
    }
  }

  const detailIndex = selectedHash
    ? commits.findIndex((commit) => commit.hash === selectedHash)
    : -1
  const detailExcluded = detailIndex >= 0 && detailIndex < tipIndex

  const detailBody = (() => {
    if (!selectedHash) {
      return (
        <div className="push-detail-empty">
          <span className="push-detail-empty-icon">
            <GitCommitHorizontal size={20} strokeWidth={1.7} />
          </span>
          <strong>{t('push.noSelection')}</strong>
          <span>{t('push.noSelectionHint')}</span>
        </div>
      )
    }
    if (detailLoading) {
      return (
        <div className="push-detail-empty">
          <LoaderCircle size={17} strokeWidth={2} className="is-spinning" />
          <span>{t('push.detailLoading')}</span>
        </div>
      )
    }
    if (detailError) {
      return (
        <div className="push-detail-empty is-error" role="alert">
          <CircleAlert size={17} strokeWidth={2} />
          <strong>{detailError}</strong>
          <button type="button" className="push-retry" onClick={retryDetail}>
            {t('push.retry')}
          </button>
        </div>
      )
    }
    if (detail) {
      return (
        <CommitChanges
          detail={detail}
          appTheme={appTheme}
          locale={locale}
          excluded={detailExcluded}
        />
      )
    }
    return null
  })()

  return (
    <DialogFrame
      title={t('push.title', { name: projectName })}
      description={
        info ? t('push.description', { branch: info.branch, target: targetRef }) : undefined
      }
      icon={<ArrowUpFromLine size={16} strokeWidth={1.9} />}
      onClose={onClose}
      large
    >
      <form className="manage-form push-form" onSubmit={submit}>
        <div className="push-body">
          <section className="push-list-pane">
            <header className="push-list-head">
              <button
                type="button"
                className="push-select-all"
                onClick={toggleAll}
                disabled={commits.length === 0 || pushing}
                title={t('push.selectAll')}
              >
                <span
                  className={`push-checkbox ${
                    allSelected ? 'is-checked' : someSelected ? 'is-mixed' : ''
                  }`}
                  aria-hidden="true"
                >
                  {allSelected && <Check size={10} strokeWidth={3} />}
                  {someSelected && <Minus size={10} strokeWidth={3} />}
                </span>
                <span>{t('push.selectAll')}</span>
              </button>
              <span className="push-list-count">
                {t('push.commitsCount', { count: commits.length })}
              </span>
              <button
                type="button"
                className="icon-button subtle push-refresh"
                onClick={refreshInfo}
                disabled={loading || pushing}
                title={t('push.refresh')}
                aria-label={t('push.refresh')}
              >
                <RefreshCw
                  size={13}
                  strokeWidth={1.9}
                  className={loading ? 'is-spinning' : ''}
                />
              </button>
            </header>

            {loading ? (
              <div className="push-list-state">
                <LoaderCircle size={15} strokeWidth={2} className="is-spinning" />
                <span>{t('push.loading')}</span>
              </div>
            ) : loadError ? (
              <div className="push-list-state is-error" role="alert">
                <CircleAlert size={16} strokeWidth={2} />
                <strong>{loadError}</strong>
                <button
                  type="button"
                  className="push-retry"
                  onClick={refreshInfo}
                >
                  {t('push.retry')}
                </button>
              </div>
            ) : commits.length === 0 ? (
              <div className="push-list-state">
                <GitCommitHorizontal size={18} strokeWidth={1.7} />
                <strong>{t('push.noCommits')}</strong>
                <span>{t('push.noCommitsHint')}</span>
              </div>
            ) : (
              <div
                className="push-commit-list"
                role="listbox"
                aria-label={t('push.commitList')}
                aria-multiselectable="true"
              >
                {commits.map((commit, index) => {
                  const checked = index >= tipIndex
                  const active = commit.hash === selectedHash
                  return (
                    <div
                      key={commit.hash}
                      role="option"
                      aria-selected={active}
                      tabIndex={0}
                      className={`push-commit ${
                        active ? 'is-active' : ''
                      } ${checked ? '' : 'is-excluded'}`}
                      onClick={() => openCommit(commit.hash)}
                      onKeyDown={(event) => {
                        if (event.key === 'Enter' || event.key === ' ') {
                          event.preventDefault()
                          openCommit(commit.hash)
                        }
                      }}
                    >
                      <button
                        type="button"
                        role="checkbox"
                        aria-checked={checked}
                        className={`push-checkbox ${checked ? 'is-checked' : ''}`}
                        onClick={(event) => {
                          event.stopPropagation()
                          toggleCommit(index)
                        }}
                        tabIndex={-1}
                        title={t('push.includeCommit', {
                          hash: commit.shortHash,
                        })}
                      >
                        {checked && <Check size={10} strokeWidth={3} />}
                      </button>
                      <span className="push-commit-copy">
                        <span className="push-commit-subject">
                          {commit.subject || t('push.noSubject')}
                        </span>
                        <span className="push-commit-meta">
                          <span className="push-commit-hash">
                            {commit.shortHash}
                          </span>
                          <span className="push-commit-author">
                            {commit.author}
                          </span>
                          <span>{formatTimestamp(commit.time, locale)}</span>
                        </span>
                      </span>
                    </div>
                  )
                })}
              </div>
            )}

            {info && info.commitsTotal > commits.length && (
              <p className="push-list-note">
                {t('push.commitsTruncated', {
                  count: commits.length,
                  total: info.commitsTotal,
                })}
              </p>
            )}
          </section>

          <section className="push-detail-pane" aria-label={t('push.detail')}>
            {detailBody}
          </section>
        </div>

        {pushError && (
          <div className="push-banner is-error" role="alert">
            <CircleAlert size={14} strokeWidth={2} />
            <span>{pushError}</span>
          </div>
        )}
        {!pushError && info && info.behind > 0 && (
          <div className="push-banner is-warning">
            <TriangleAlert size={14} strokeWidth={2} />
            <span>{t('push.behind', { count: info.behind })}</span>
          </div>
        )}
        {!pushError && info && !info.upstream && commits.length > 0 && (
          <div className="push-banner is-info">
            <Tag size={14} strokeWidth={2} />
            <span>{t('push.newBranch', { target: targetRef })}</span>
          </div>
        )}
        {!pushError && force && (
          <div className="push-banner is-danger">
            <TriangleAlert size={14} strokeWidth={2} />
            <span>{t('push.forceWarning')}</span>
          </div>
        )}
        {!pushError &&
          info &&
          tipIndex > 0 &&
          selectedCount > 0 &&
          commits[tipIndex] && (
            <div className="push-banner is-info">
              <GitCommitHorizontal size={14} strokeWidth={2} />
              <span>
                {t('push.partialHint', {
                  count: selectedCount,
                  hash: commits[tipIndex].shortHash,
                  newer: tipIndex,
                })}
              </span>
            </div>
          )}

        <footer className="manage-footer push-footer">
          <div className="push-footer-left">
            <button
              type="button"
              role="checkbox"
              aria-checked={pushTags}
              className={`push-tags-toggle ${pushTags ? 'is-checked' : ''}`}
              onClick={() => {
                setPushTags((current) => !current)
                setTagMenuOpen(false)
                setPushError(null)
              }}
              disabled={!tagsAvailable || pushing}
            >
              <span
                className={`push-checkbox ${pushTags ? 'is-checked' : ''}`}
                aria-hidden="true"
              >
                {pushTags && <Check size={10} strokeWidth={3} />}
              </span>
              <Tag size={13} strokeWidth={1.9} />
              {t('push.tags')}
            </button>

            <div className="push-tag-select" ref={tagMenuRef}>
              <button
                type="button"
                className="push-tag-button"
                onClick={() => setTagMenuOpen((current) => !current)}
                disabled={!pushTags || !tagsAvailable || pushing}
                aria-haspopup="listbox"
                aria-expanded={tagMenuOpen}
              >
                <span>
                  {!tagsAvailable
                    ? t('push.noTags')
                    : tagChoice === 'all'
                      ? t('push.allTags')
                      : tagChoice}
                </span>
                <ChevronDown size={12} strokeWidth={2} />
              </button>
              {tagMenuOpen && tagsAvailable && (
                <div className="push-menu push-tag-menu" role="listbox">
                  <button
                    type="button"
                    role="option"
                    aria-selected={tagChoice === 'all'}
                    className={`push-menu-item ${
                      tagChoice === 'all' ? 'is-active' : ''
                    }`}
                    onClick={() => {
                      setTagChoice('all')
                      setTagMenuOpen(false)
                    }}
                  >
                    {t('push.allTags')}
                    {tagChoice === 'all' && <Check size={11} strokeWidth={2.6} />}
                  </button>
                  {info?.tags.map((tag) => (
                    <button
                      key={tag}
                      type="button"
                      role="option"
                      aria-selected={tagChoice === tag}
                      className={`push-menu-item ${
                        tagChoice === tag ? 'is-active' : ''
                      }`}
                      onClick={() => {
                        setTagChoice(tag)
                        setTagMenuOpen(false)
                      }}
                    >
                      <span className="push-menu-tag">{tag}</span>
                      {tagChoice === tag && <Check size={11} strokeWidth={2.6} />}
                    </button>
                  ))}
                </div>
              )}
            </div>
          </div>

          <div className="push-footer-right">
            {!canPush && !pushing && !loading && !loadError && (
              <span className="push-footer-hint">{t('push.selectHint')}</span>
            )}
            <button
              type="button"
              className="manage-cancel"
              onClick={onClose}
              disabled={pushing}
            >
              {t('common.cancel')}
            </button>
            <div className="push-submit-group" ref={submitMenuRef}>
              <button
                type="submit"
                className={`push-submit ${force ? 'is-force' : ''}`}
                disabled={!canPush}
                title={force ? t('push.forcePushTitle') : undefined}
              >
                {pushing ? (
                  <LoaderCircle size={13} strokeWidth={2} className="is-spinning" />
                ) : (
                  <ArrowUpFromLine size={13} strokeWidth={2} />
                )}
                {pushing
                  ? t('push.pushing')
                  : force
                    ? t('push.forcePush')
                    : t('push.push')}
              </button>
              <button
                type="button"
                className={`push-submit-caret ${force ? 'is-force' : ''}`}
                onClick={() => setForceMenuOpen((current) => !current)}
                disabled={pushing}
                aria-haspopup="menu"
                aria-expanded={forceMenuOpen}
                aria-label={t('push.options')}
                title={t('push.options')}
              >
                <ChevronDown size={13} strokeWidth={2.2} />
              </button>
              {forceMenuOpen && (
                <div className="push-menu push-submit-menu" role="menu">
                  <button
                    type="button"
                    role="menuitemradio"
                    aria-checked={!force}
                    className={`push-menu-item ${force ? '' : 'is-active'}`}
                    onClick={() => {
                      setForce(false)
                      setForceMenuOpen(false)
                    }}
                  >
                    <span className="push-menu-label">
                      <ArrowUpFromLine size={13} strokeWidth={1.9} />
                      {t('push.push')}
                    </span>
                    {!force && <Check size={11} strokeWidth={2.6} />}
                  </button>
                  <button
                    type="button"
                    role="menuitemradio"
                    aria-checked={force}
                    className={`push-menu-item is-force ${force ? 'is-active' : ''}`}
                    onClick={() => {
                      setForce(true)
                      setForceMenuOpen(false)
                    }}
                  >
                    <span className="push-menu-label">
                      <TriangleAlert size={13} strokeWidth={1.9} />
                      {t('push.forcePushTitle')}
                    </span>
                    {force && <Check size={11} strokeWidth={2.6} />}
                  </button>
                </div>
              )}
            </div>
          </div>
        </footer>
      </form>
    </DialogFrame>
  )
}

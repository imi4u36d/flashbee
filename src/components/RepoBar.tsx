import { useEffect, useRef, useState } from 'react'
import type { MouseEvent as ReactMouseEvent } from 'react'
import {
  ArrowDownToLine,
  ArrowUpFromLine,
  CloudCog,
  Code2,
  Copy,
  FolderOpen,
  FolderPlus,
  GitFork,
  Layers3,
  LoaderCircle,
  MoreHorizontal,
  PanelLeft,
  RefreshCw,
  Search,
  Settings2,
  Trash2,
  TriangleAlert,
} from 'lucide-react'
import { BranchSelect } from './BranchSelect'
import { useI18n } from '../lib/i18n'
import type { Project } from '../types'

interface RepoBarProps {
  project: Project | null
  workspaceName: string
  sidebarCollapsed: boolean
  refreshing: boolean
  syncing?: 'pull' | null
  conflictCount?: number
  mergeInProgress?: boolean
  onOpenSidebar: () => void
  onOpenSettings: () => void
  onQuickOpen: () => void
  onRefresh: () => void
  onPull?: () => void
  onPush?: () => void
  onSelectBranch: (branch: string) => void
  onOpenRemoteSettings?: () => void
  onResolveConflicts?: () => void
  onAddRepository: () => void
  onRequestRemoveRepository: () => void
  onRevealInFinder: () => void
  onOpenInEditor: () => void
  onCopyPath: () => void
  onContextMenu: (event: ReactMouseEvent) => void
}

export function RepoBar({
  project,
  workspaceName,
  sidebarCollapsed,
  refreshing,
  syncing = null,
  conflictCount = 0,
  mergeInProgress = false,
  onOpenSidebar,
  onOpenSettings,
  onQuickOpen,
  onRefresh,
  onPull,
  onPush,
  onSelectBranch,
  onOpenRemoteSettings,
  onResolveConflicts,
  onAddRepository,
  onRequestRemoveRepository,
  onRevealInFinder,
  onOpenInEditor,
  onCopyPath,
  onContextMenu,
}: RepoBarProps) {
  const { t } = useI18n()
  const [actionsOpen, setActionsOpen] = useState(false)
  const actionsRef = useRef<HTMLDivElement>(null)
  const local = project?.source === 'local'
  const canSync = local && Boolean(onPull && onPush)
  const syncingBusy = syncing !== null
  const mergeBlocked = conflictCount > 0 || mergeInProgress
  const syncDisabled = !canSync || syncingBusy || mergeBlocked
  const pullTitle = mergeBlocked ? t('repo.syncBlocked') : t('repo.pullTitle')
  const pushTitle = mergeBlocked ? t('repo.syncBlocked') : t('repo.pushTitle')

  useEffect(() => {
    if (!actionsOpen) return

    const closeOnOutsideClick = (event: PointerEvent) => {
      if (!actionsRef.current?.contains(event.target as Node)) {
        setActionsOpen(false)
      }
    }
    const closeOnEscape = (event: KeyboardEvent) => {
      if (event.key === 'Escape') {
        event.stopPropagation()
        setActionsOpen(false)
      }
    }

    document.addEventListener('pointerdown', closeOnOutsideClick)
    window.addEventListener('keydown', closeOnEscape, true)
    return () => {
      document.removeEventListener('pointerdown', closeOnOutsideClick)
      window.removeEventListener('keydown', closeOnEscape, true)
    }
  }, [actionsOpen])

  return (
    <header className="repo-bar" data-tauri-drag-region="deep">
      <div
        className="repo-identity"
        onContextMenu={project ? onContextMenu : undefined}
      >
        {sidebarCollapsed && (
          <button
            className="icon-button subtle open-sidebar-button"
            onClick={onOpenSidebar}
            title={`${t('repo.openSidebar')}  ⌘\\`}
            aria-label={t('repo.openSidebar')}
          >
            <PanelLeft size={16} strokeWidth={1.8} />
          </button>
        )}
        <span
          className="repo-mark"
          style={
            project
              ? {
                  color: project.accent,
                  background: project.accentSoft,
                  borderColor: `${project.accent}26`,
                }
              : {
                  color: 'var(--amber)',
                  background: 'var(--amber-soft)',
                  borderColor: 'rgba(246, 189, 96, 0.14)',
                }
          }
        >
          {project ? (
            project.name.charAt(0).toUpperCase()
          ) : (
            <Layers3 size={14} strokeWidth={1.9} />
          )}
        </span>
        <span className="repo-copy">
          <span className="repo-title-row">
            <span className="repo-title">{project?.name ?? workspaceName}</span>
            {project &&
              (project.worktreeOf ? (
                <span className="repo-local-chip" title={project.path}>
                  <GitFork size={10} strokeWidth={2.2} />
                  {t('repo.worktree')}
                </span>
              ) : (
                <span className="repo-local-chip">
                  <i />
                  {t('repo.local')}
                </span>
              ))}
          </span>
          <span className="repo-path">
            {project?.path ?? t('sidebar.noRepositories')}
          </span>
        </span>
      </div>

      {project && (
        <BranchSelect
          branches={project.branches ?? [project.branch]}
          remoteBranches={project.remoteBranches ?? []}
          value={project.branch}
          onChange={onSelectBranch}
        />
      )}

      <div className="repo-actions-wrap" ref={actionsRef}>
        <button
          className={`icon-button toolbar-button repo-actions-button ${
            actionsOpen ? 'is-active' : ''
          }`}
          onClick={() => setActionsOpen((open) => !open)}
          title={t('repo.actions')}
          aria-label={t('repo.actions')}
          aria-haspopup="menu"
          aria-expanded={actionsOpen}
        >
          <MoreHorizontal size={15} strokeWidth={1.9} />
        </button>

        {actionsOpen && (
          <div className="repo-actions-menu" role="menu">
            <div className="repo-actions-label">
              {project?.name ?? workspaceName}
            </div>
            {project && (
              <>
                {conflictCount > 0 && onResolveConflicts && (
                  <button
                    type="button"
                    role="menuitem"
                    className="repo-actions-item is-conflict"
                    onClick={() => {
                      setActionsOpen(false)
                      onResolveConflicts()
                    }}
                  >
                    <TriangleAlert size={14} strokeWidth={1.9} />
                    {t('conflict.resolveCount', { count: conflictCount })}
                  </button>
                )}
                <button
                  type="button"
                  role="menuitem"
                  className="repo-actions-item"
                  onClick={() => {
                    setActionsOpen(false)
                    onRevealInFinder()
                  }}
                >
                  <FolderOpen size={14} strokeWidth={1.9} />
                  {t('repo.revealInFinder')}
                </button>
                <button
                  type="button"
                  role="menuitem"
                  className="repo-actions-item"
                  onClick={() => {
                    setActionsOpen(false)
                    onOpenInEditor()
                  }}
                >
                  <Code2 size={14} strokeWidth={1.9} />
                  {t('repo.openInEditor')}
                </button>
                <button
                  type="button"
                  role="menuitem"
                  className="repo-actions-item"
                  onClick={() => {
                    setActionsOpen(false)
                    onCopyPath()
                  }}
                >
                  <Copy size={13} strokeWidth={1.9} />
                  {t('repo.copyPath')}
                </button>
                <span className="repo-actions-divider" />
                <button
                  type="button"
                  role="menuitem"
                  className="repo-actions-item"
                  onClick={() => {
                    setActionsOpen(false)
                    onRefresh()
                  }}
                >
                  <RefreshCw
                    size={14}
                    strokeWidth={1.9}
                    className={refreshing ? 'is-spinning' : ''}
                  />
                  {t('repo.refresh')}
                </button>
                {local && onOpenRemoteSettings && (
                  <button
                    type="button"
                    role="menuitem"
                    className="repo-actions-item"
                    onClick={() => {
                      setActionsOpen(false)
                      onOpenRemoteSettings()
                    }}
                  >
                    <CloudCog size={14} strokeWidth={1.9} />
                    {t('repo.remoteSettings')}
                  </button>
                )}
                <span className="repo-actions-divider" />
              </>
            )}
            <button
              type="button"
              role="menuitem"
              className="repo-actions-item"
              onClick={() => {
                setActionsOpen(false)
                onAddRepository()
              }}
            >
              <FolderPlus size={14} strokeWidth={1.9} />
              {t('repo.add')}
            </button>
            {project && !project.worktreeOf && (
              <button
                type="button"
                role="menuitem"
                className="repo-actions-item is-danger"
                onClick={() => {
                  setActionsOpen(false)
                  onRequestRemoveRepository()
                }}
              >
                <Trash2 size={13} strokeWidth={1.9} />
                {t('repo.remove')}
              </button>
            )}
          </div>
        )}
      </div>

      <div className="repo-bar-spacer" />

      <button className="quick-jump" data-testid="quick-open" onClick={onQuickOpen}>
        <Search size={14} strokeWidth={1.9} />
        <span>{t('repo.jumpToFile')}</span>
        <kbd>⌘K</kbd>
      </button>

      {canSync && (
        <div className="repo-sync-group" role="group" aria-label={t('repo.sync')}>
          <button
            className="icon-button toolbar-button sync-button"
            onClick={() => onPull?.()}
            disabled={syncDisabled}
            title={pullTitle}
            aria-label={t('repo.pull')}
            aria-busy={syncing === 'pull'}
          >
            {syncing === 'pull' ? (
              <LoaderCircle size={15} strokeWidth={2} className="is-spinning" />
            ) : (
              <ArrowDownToLine size={15} strokeWidth={1.8} />
            )}
          </button>
          <button
            className="icon-button toolbar-button sync-button"
            onClick={() => onPush?.()}
            disabled={syncDisabled}
            title={pushTitle}
            aria-label={t('repo.push')}
          >
            <ArrowUpFromLine size={15} strokeWidth={1.8} />
          </button>
        </div>
      )}

      <button
        className="icon-button toolbar-button settings-button"
        data-testid="settings-button"
        onClick={onOpenSettings}
        title={`${t('repo.settings')}  ⌘,`}
        aria-label={t('repo.openSettings')}
      >
        <Settings2 size={15} strokeWidth={1.8} />
      </button>
    </header>
  )
}

import { useEffect, useRef, useState } from 'react'
import {
  FolderPlus,
  Layers3,
  MoreHorizontal,
  PanelLeft,
  RefreshCw,
  Search,
  Settings2,
  Trash2,
} from 'lucide-react'
import { BranchSelect } from './BranchSelect'
import { useI18n } from '../lib/i18n'
import type { Project } from '../types'

interface RepoBarProps {
  project: Project | null
  workspaceName: string
  sidebarCollapsed: boolean
  refreshing: boolean
  onOpenSidebar: () => void
  onOpenSettings: () => void
  onQuickOpen: () => void
  onRefresh: () => void
  onSelectBranch: (branch: string) => void
  onAddRepository: () => void
  onRequestRemoveRepository: () => void
}

export function RepoBar({
  project,
  workspaceName,
  sidebarCollapsed,
  refreshing,
  onOpenSidebar,
  onOpenSettings,
  onQuickOpen,
  onRefresh,
  onSelectBranch,
  onAddRepository,
  onRequestRemoveRepository,
}: RepoBarProps) {
  const { t } = useI18n()
  const [actionsOpen, setActionsOpen] = useState(false)
  const actionsRef = useRef<HTMLDivElement>(null)

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
    <header className="repo-bar">
      <div className="repo-identity">
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
            {project && <span className="repo-visibility">{t('repo.private')}</span>}
          </span>
          <span className="repo-path">
            {project?.path ?? t('sidebar.noRepositories')}
          </span>
        </span>
      </div>

      {project && (
        <BranchSelect
          branches={project.branches ?? [project.branch]}
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
            {project && (
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

      <button
        className="icon-button toolbar-button"
        onClick={onRefresh}
        disabled={!project}
        title={t('repo.refresh')}
        aria-label={t('repo.refresh')}
      >
        <RefreshCw
          size={15}
          strokeWidth={1.8}
          className={refreshing ? 'is-spinning' : ''}
        />
      </button>

      <button
        className="icon-button toolbar-button settings-button"
        data-testid="settings-button"
        onClick={onOpenSettings}
        title={`${t('repo.settings')}  ⌘,`}
        aria-label={t('repo.openSettings')}
      >
        <Settings2 size={15} strokeWidth={1.8} />
      </button>

      <button className="user-avatar" title="wangzhuo" aria-label={t('repo.openProfile')}>
        W
        <span className="presence-dot" />
      </button>
    </header>
  )
}

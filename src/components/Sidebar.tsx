import { useEffect, useRef, useState } from 'react'
import {
  ArrowUp,
  Check,
  ChevronDown,
  Command,
  FolderPlus,
  GitBranch,
  Layers3,
  PanelLeftClose,
  Plus,
  Trash2,
} from 'lucide-react'
import {
  fileStats,
  pathParts,
  statusLabels,
} from '../data/projects'
import { useI18n } from '../lib/i18n'
import {
  workspaceInitial,
  workspaceLabel,
} from '../lib/workspaces'
import type { DiffFile, Project, Workspace } from '../types'

interface SidebarProps {
  workspace: Workspace
  workspaces: Workspace[]
  projects: Project[]
  activeProjectId: string
  expandedProjectId: string | null
  selectedFile: string | null
  onProjectClick: (project: Project) => void
  onSelectFile: (project: Project, file: DiffFile) => void
  onSwitchWorkspace: (workspaceId: string) => void
  onCreateWorkspace: () => void
  onRequestDeleteWorkspace: () => void
  onAddRepository: () => void
  onQuickOpen: () => void
  onCollapse: () => void
}

function ProjectMark({ project }: { project: Project }) {
  return (
    <span
      className="project-mark"
      style={{
        color: project.accent,
        background: project.accentSoft,
        borderColor: `${project.accent}26`,
      }}
    >
      <span>{project.name.charAt(0).toUpperCase()}</span>
      <i style={{ background: project.accent }} />
    </span>
  )
}

function FileGroup({
  label,
  files,
  project,
  selectedFile,
  onSelectFile,
}: {
  label: string
  files: DiffFile[]
  project: Project
  selectedFile: string | null
  onSelectFile: (project: Project, file: DiffFile) => void
}) {
  if (files.length === 0) return null

  return (
    <div className="file-group">
      <div className="file-group-title">
        <span>{label}</span>
        <span>{files.length}</span>
      </div>
      {files.map((file) => {
        const { name, dir } = pathParts(file.path)
        const stats = fileStats(file)
        return (
          <button
            key={file.path}
            data-testid={`file-${project.id}-${file.path.replaceAll('/', '-')}`}
            className={`file-row ${selectedFile === file.path ? 'is-selected' : ''}`}
            onClick={() => onSelectFile(project, file)}
            title={file.path}
          >
            <span className={`file-status status-${file.status}`}>
              {statusLabels[file.status]}
            </span>
            <span className="file-copy">
              <span className="file-name">{name}</span>
              <span className="file-dir">{dir || project.name}</span>
            </span>
            <span className="file-delta">
              <span className="delta-add">+{stats.additions}</span>
              <span className="delta-del">−{stats.deletions}</span>
            </span>
          </button>
        )
      })}
    </div>
  )
}

export function Sidebar({
  workspace,
  workspaces,
  projects,
  activeProjectId,
  expandedProjectId,
  selectedFile,
  onProjectClick,
  onSelectFile,
  onSwitchWorkspace,
  onCreateWorkspace,
  onRequestDeleteWorkspace,
  onAddRepository,
  onQuickOpen,
  onCollapse,
}: SidebarProps) {
  const { t } = useI18n()
  const [workspaceMenuOpen, setWorkspaceMenuOpen] = useState(false)
  const workspaceSwitchRef = useRef<HTMLDivElement>(null)

  useEffect(() => {
    if (!workspaceMenuOpen) return

    const closeOnOutsideClick = (event: PointerEvent) => {
      if (!workspaceSwitchRef.current?.contains(event.target as Node)) {
        setWorkspaceMenuOpen(false)
      }
    }
    const closeOnEscape = (event: KeyboardEvent) => {
      if (event.key === 'Escape') {
        event.stopPropagation()
        setWorkspaceMenuOpen(false)
      }
    }

    document.addEventListener('pointerdown', closeOnOutsideClick)
    window.addEventListener('keydown', closeOnEscape, true)
    return () => {
      document.removeEventListener('pointerdown', closeOnOutsideClick)
      window.removeEventListener('keydown', closeOnEscape, true)
    }
  }, [workspaceMenuOpen])

  const activeWorkspaceName = workspaceLabel(workspace, t)
  const repoCountLabel = (count: number) =>
    t(count === 1 ? 'sidebar.workspaceRepoCountOne' : 'sidebar.workspaceRepoCountMany', {
      count,
    })

  return (
    <aside className="sidebar">
      <div className="sidebar-aurora" aria-hidden="true" />

      <header className="sidebar-brand">
        <div className="brand-lockup">
          <span className="brand-mark" aria-hidden="true">
            <svg viewBox="0 0 24 24" fill="none">
              <path
                d="M12 2.6 20.1 7v10L12 21.4 3.9 17V7L12 2.6Z"
                fill="currentColor"
                opacity=".16"
              />
              <path
                d="M12 2.6 20.1 7v10L12 21.4 3.9 17V7L12 2.6Z"
                stroke="currentColor"
                strokeWidth="1.35"
              />
              <path
                d="m13.45 6.25-5.2 6.6h3.05l-.78 4.9 5.22-6.9h-3.1l.81-4.6Z"
                fill="currentColor"
              />
            </svg>
          </span>
          <span className="brand-name">FlashBee</span>
          <span className="brand-tag">{t('common.local')}</span>
        </div>
        <button
          className="icon-button subtle"
          onClick={onCollapse}
          title={`${t('sidebar.collapse')}  ⌘\\`}
          aria-label={t('sidebar.collapse')}
        >
          <PanelLeftClose size={16} strokeWidth={1.8} />
        </button>
      </header>

      <div className="workspace-switch-wrap" ref={workspaceSwitchRef}>
        <button
          className={`workspace-switcher ${workspaceMenuOpen ? 'is-open' : ''}`}
          type="button"
          aria-haspopup="menu"
          aria-expanded={workspaceMenuOpen}
          onClick={() => setWorkspaceMenuOpen((open) => !open)}
        >
          <span className="workspace-avatar">{workspaceInitial(workspace, t)}</span>
          <span className="workspace-copy">
            <strong>{activeWorkspaceName}</strong>
            <span>
              {repoCountLabel(projects.length)}
            </span>
          </span>
          <ChevronDown size={14} className="workspace-chevron" />
        </button>

        {workspaceMenuOpen && (
          <div className="workspace-menu" role="menu">
            <div className="workspace-menu-label">{t('sidebar.workspaces')}</div>
            {workspaces.map((entry) => {
              const isActive = entry.id === workspace.id
              return (
                <button
                  key={entry.id}
                  type="button"
                  role="menuitemradio"
                  aria-checked={isActive}
                  className={`workspace-menu-item ${isActive ? 'is-active' : ''}`}
                  onClick={() => {
                    setWorkspaceMenuOpen(false)
                    onSwitchWorkspace(entry.id)
                  }}
                >
                  <span className="workspace-menu-avatar">
                    {workspaceInitial(entry, t)}
                  </span>
                  <span className="workspace-menu-copy">
                    <strong>{workspaceLabel(entry, t)}</strong>
                    <span>
                      {repoCountLabel(entry.projects.length)}
                    </span>
                  </span>
                  {isActive && <Check size={13} strokeWidth={2.2} />}
                </button>
              )
            })}
            <span className="workspace-menu-divider" />
            <button
              type="button"
              className="workspace-menu-action"
              onClick={() => {
                setWorkspaceMenuOpen(false)
                onCreateWorkspace()
              }}
            >
              <Plus size={14} strokeWidth={1.9} />
              {t('sidebar.newWorkspace')}
            </button>
            <button
              type="button"
              className="workspace-menu-action is-danger"
              disabled={workspaces.length <= 1}
              title={
                workspaces.length <= 1
                  ? t('sidebar.keepOneWorkspace')
                  : t('sidebar.deleteWorkspace')
              }
              onClick={() => {
                setWorkspaceMenuOpen(false)
                onRequestDeleteWorkspace()
              }}
            >
              <Trash2 size={13} strokeWidth={1.9} />
              {t('sidebar.deleteWorkspace')}
            </button>
          </div>
        )}
      </div>

      <div className="projects-heading">
        <span>{t('sidebar.repositories')}</span>
        <span className="heading-count">{projects.length}</span>
        <span className="heading-line" />
        <button
          className="icon-button subtle heading-add"
          onClick={onAddRepository}
          title={t('sidebar.addRepository')}
          aria-label={t('sidebar.addRepository')}
        >
          <Plus size={14} strokeWidth={2} />
        </button>
      </div>

      <div className="projects-scroll">
        {projects.length === 0 && (
          <div className="sidebar-empty-repos">
            <span className="sidebar-empty-icon">
              <Layers3 size={17} strokeWidth={1.7} />
            </span>
            <strong>{t('sidebar.noRepositories')}</strong>
            <button onClick={onAddRepository}>
              <FolderPlus size={12} strokeWidth={2} />
              {t('repo.add')}
            </button>
          </div>
        )}
        {projects.map((project) => {
          const expanded = expandedProjectId === project.id
          const active = activeProjectId === project.id
          const staged = project.files.filter((file) => file.staged)
          const unstaged = project.files.filter((file) => !file.staged)

          return (
            <section
              key={project.id}
              className={`project-card ${active ? 'is-active' : ''} ${
                expanded ? 'is-expanded' : ''
              }`}
            >
              <button
                className="project-header"
                data-testid={`project-${project.id}`}
                onClick={() => onProjectClick(project)}
                title={project.path}
                aria-expanded={expanded}
              >
                <ProjectMark project={project} />
                <span className="project-copy">
                  <span className="project-topline">
                    <span className="project-name">{project.name}</span>
                    {project.files.length > 0 ? (
                      <span className="project-count">{project.files.length}</span>
                    ) : (
                      <span className="clean-label">{t('common.clean')}</span>
                    )}
                  </span>
                  <span className="project-branchline">
                    <GitBranch size={11} strokeWidth={1.9} />
                    <span className="branch-name">{project.branch}</span>
                    {project.ahead > 0 && (
                      <span className="ahead-count">
                        <ArrowUp size={9} strokeWidth={2.4} />
                        {project.ahead}
                      </span>
                    )}
                    {project.behind > 0 && (
                      <span className="behind-count">↓{project.behind}</span>
                    )}
                  </span>
                </span>
                <ChevronDown
                  size={14}
                  className={`project-chevron ${expanded ? 'is-open' : ''}`}
                />
              </button>

              {expanded && (
                <div className="project-files">
                  {project.files.length === 0 ? (
                    <div className="clean-state">
                      <span className="clean-state-check">✓</span>
                      <div>
                        <strong>{t('sidebar.workingTreeClean')}</strong>
                        <span>{t('sidebar.upToDate')}</span>
                      </div>
                    </div>
                  ) : (
                    <>
                      <FileGroup
                        label={t('sidebar.staged')}
                        files={staged}
                        project={project}
                        selectedFile={active ? selectedFile : null}
                        onSelectFile={onSelectFile}
                      />
                      <FileGroup
                        label={t('sidebar.changes')}
                        files={unstaged}
                        project={project}
                        selectedFile={active ? selectedFile : null}
                        onSelectFile={onSelectFile}
                      />
                    </>
                  )}
                </div>
              )}
            </section>
          )
        })}
      </div>

      <footer className="sidebar-footer">
        <button className="quick-open-row" onClick={onQuickOpen}>
          <span className="quick-open-copy">
            <Command size={13} strokeWidth={1.9} />
            {t('sidebar.quickOpen')}
          </span>
          <kbd>⌘ K</kbd>
        </button>
        <div className="watch-row">
          <span className="watch-dot" />
          <span>
              {t(
                projects.length === 1
                  ? 'sidebar.watchingOne'
                  : 'sidebar.watchingMany',
                { count: projects.length },
              )}
            </span>
          <span className="watch-latency">12ms</span>
        </div>
      </footer>
    </aside>
  )
}

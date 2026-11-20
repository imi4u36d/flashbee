import { useEffect, useRef, useState } from 'react'
import type {
  MouseEvent as ReactMouseEvent,
  ReactNode,
  Ref,
} from 'react'
import {
  ArrowUp,
  Check,
  ChevronDown,
  Command,
  FolderGit2,
  FolderPlus,
  GitBranch,
  Layers3,
  PanelLeftClose,
  Plus,
  RefreshCw,
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
import type { DiffFile, Project, ProjectGroup, Workspace } from '../types'

interface SidebarProps {
  ref?: Ref<HTMLElement>
  width: number | null
  workspace: Workspace
  workspaces: Workspace[]
  projects: Project[]
  groups: ProjectGroup[]
  activeProjectId: string
  expandedProjectId: string | null
  selectedFile: string | null
  scanningGroupId: string | null
  onProjectClick: (project: Project) => void
  onSelectFile: (project: Project, file: DiffFile) => void
  onSwitchWorkspace: (workspaceId: string) => void
  onCreateWorkspace: () => void
  onRequestDeleteWorkspace: () => void
  onAddRepository: () => void
  onQuickOpen: () => void
  onCollapse: () => void
  onProjectContextMenu: (
    project: Project,
    event: ReactMouseEvent,
  ) => void
  onFileContextMenu: (
    project: Project,
    file: DiffFile,
    event: ReactMouseEvent,
  ) => void
  onGroupContextMenu: (
    group: ProjectGroup,
    event: ReactMouseEvent,
  ) => void
  onToggleGroup: (group: ProjectGroup) => void
  onRescanGroup: (group: ProjectGroup) => void
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

function ProjectFiles({
  open,
  children,
}: {
  open: boolean
  children: ReactNode
}) {
  const [mounted, setMounted] = useState(open)
  const [revealed, setRevealed] = useState(open)

  useEffect(() => {
    if (open) {
      // oxlint-disable-next-line react/set-state-in-effect
      setMounted(true)
      return
    }
    // oxlint-disable-next-line react/set-state-in-effect
    setRevealed(false)
  }, [open])

  useEffect(() => {
    if (!mounted) return

    if (!open) {
      const timer = window.setTimeout(() => setMounted(false), 240)
      return () => window.clearTimeout(timer)
    }

    // Let the collapsed frame paint before revealing so the grid row animates.
    let secondFrame = 0
    const firstFrame = window.requestAnimationFrame(() => {
      secondFrame = window.requestAnimationFrame(() => setRevealed(true))
    })
    // Keep the panel opening even when animation frames are paused.
    const fallback = window.setTimeout(() => setRevealed(true), 40)
    return () => {
      window.cancelAnimationFrame(firstFrame)
      window.cancelAnimationFrame(secondFrame)
      window.clearTimeout(fallback)
    }
  }, [mounted, open])

  return (
    <div
      className={`project-files-reveal ${revealed ? 'is-open' : ''}`}
      inert={!open}
    >
      {mounted ? (
        <div className="project-files-clip">
          <div className="project-files">{children}</div>
        </div>
      ) : null}
    </div>
  )
}

function FileGroup({
  label,
  files,
  project,
  selectedFile,
  nameCounts,
  onSelectFile,
  onContextMenu,
}: {
  label: string
  files: DiffFile[]
  project: Project
  selectedFile: string | null
  nameCounts: Map<string, number>
  onSelectFile: (project: Project, file: DiffFile) => void
  onContextMenu: (
    project: Project,
    file: DiffFile,
    event: ReactMouseEvent,
  ) => void
}) {
  if (files.length === 0) return null

  return (
    <div className="file-group">
      <div className="file-group-title">
        <span>{label}</span>
        <span>{files.length}</span>
      </div>
      {files.map((file) => {
        const { name } = pathParts(file.path)
        const stats = fileStats(file)
        const displayName = (nameCounts.get(name) ?? 0) > 1 ? file.path : name
        return (
          <button
            key={file.path}
            data-testid={`file-${project.id}-${file.path.replaceAll('/', '-')}`}
            className={`file-row ${selectedFile === file.path ? 'is-selected' : ''}`}
            onClick={() => onSelectFile(project, file)}
            onContextMenu={(event) => onContextMenu(project, file, event)}
            title={file.path}
          >
            <span className={`file-status status-${file.status}`}>
              {statusLabels[file.status]}
            </span>
            <span className="file-copy">
              <span className="file-name">{displayName}</span>
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
  ref,
  width,
  workspace,
  workspaces,
  projects,
  groups,
  activeProjectId,
  expandedProjectId,
  selectedFile,
  scanningGroupId,
  onProjectClick,
  onSelectFile,
  onSwitchWorkspace,
  onCreateWorkspace,
  onRequestDeleteWorkspace,
  onAddRepository,
  onQuickOpen,
  onCollapse,
  onProjectContextMenu,
  onFileContextMenu,
  onGroupContextMenu,
  onToggleGroup,
  onRescanGroup,
}: SidebarProps) {
  const { t } = useI18n()
  const [workspaceMenuOpen, setWorkspaceMenuOpen] = useState(false)
  const workspaceSwitchRef = useRef<HTMLDivElement>(null)
  const [closingProjectId, setClosingProjectId] = useState<string | null>(null)
  const previousExpandedId = useRef(expandedProjectId)
  const closingTimer = useRef<number | null>(null)

  useEffect(() => {
    const previous = previousExpandedId.current
    previousExpandedId.current = expandedProjectId
    if (!previous || previous === expandedProjectId) return

    // Keep the card chrome while its files animate closed, then release it.
    setClosingProjectId(previous)
    if (closingTimer.current !== null) {
      window.clearTimeout(closingTimer.current)
    }
    closingTimer.current = window.setTimeout(() => {
      closingTimer.current = null
      setClosingProjectId(null)
    }, 280)

    return () => {
      if (closingTimer.current !== null) {
        window.clearTimeout(closingTimer.current)
        closingTimer.current = null
      }
    }
  }, [expandedProjectId])

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

  const renderProject = (project: Project) => {
    const expanded = expandedProjectId === project.id
    const closing = !expanded && closingProjectId === project.id
    const cardExpanded = expanded || closing
    const active = activeProjectId === project.id
    const staged = project.files.filter((file) => file.staged)
    const unstaged = project.files.filter((file) => !file.staged)
    const nameCounts = new Map<string, number>()
    for (const file of project.files) {
      const { name } = pathParts(file.path)
      nameCounts.set(name, (nameCounts.get(name) ?? 0) + 1)
    }

    return (
      <section
        key={project.id}
        className={`project-card ${active ? 'is-active' : ''} ${
          cardExpanded ? 'is-expanded' : ''
        }`}
      >
        <button
          className="project-header"
          data-testid={`project-${project.id}`}
          onClick={() => onProjectClick(project)}
          onContextMenu={(event) => onProjectContextMenu(project, event)}
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

        <ProjectFiles open={expanded}>
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
                nameCounts={nameCounts}
                onSelectFile={onSelectFile}
                onContextMenu={onFileContextMenu}
              />
              <FileGroup
                label={t('sidebar.changes')}
                files={unstaged}
                project={project}
                selectedFile={active ? selectedFile : null}
                nameCounts={nameCounts}
                onSelectFile={onSelectFile}
                onContextMenu={onFileContextMenu}
              />
            </>
          )}
        </ProjectFiles>
      </section>
    )
  }

  const renderGroup = (group: ProjectGroup, groupProjects: Project[]) => {
    const collapsed = Boolean(group.collapsed)
    const scanning = scanningGroupId === group.id

    return (
      <div
        key={group.id}
        className={`project-group ${collapsed ? 'is-collapsed' : ''}`}
      >
        <div
          className="project-group-header"
          onContextMenu={(event) => onGroupContextMenu(group, event)}
        >
          <button
            type="button"
            className="project-group-toggle"
            onClick={() => onToggleGroup(group)}
            aria-expanded={!collapsed}
            title={group.path}
          >
            <span className="project-group-mark">
              <FolderGit2 size={14} strokeWidth={1.8} />
            </span>
            <span className="project-group-copy">
              <span className="project-group-name">{group.name}</span>
              <span className="project-group-meta">
                {repoCountLabel(groupProjects.length)}
              </span>
            </span>
            <ChevronDown
              size={13}
              className={`project-group-chevron ${collapsed ? '' : 'is-open'}`}
            />
          </button>
          <button
            type="button"
            className="icon-button subtle project-group-rescan"
            onClick={() => onRescanGroup(group)}
            disabled={scanning}
            title={t('repo.groupRescan')}
            aria-label={t('repo.groupRescan')}
          >
            <RefreshCw
              size={12}
              strokeWidth={1.9}
              className={scanning ? 'is-spinning' : ''}
            />
          </button>
        </div>
        {collapsed ? null : (
          <div className="project-group-projects">
            {groupProjects.length === 0 ? (
              <span className="project-group-empty">{t('repo.groupEmpty')}</span>
            ) : (
              groupProjects.map(renderProject)
            )}
          </div>
        )}
      </div>
    )
  }

  // Keep groups where their first repository appears, then show any empty group.
  const groupById = new Map(groups.map((group) => [group.id, group]))
  const renderedGroupIds = new Set<string>()
  const sections: ReactNode[] = []
  for (const project of projects) {
    const group = project.groupId ? groupById.get(project.groupId) : undefined
    if (!group) {
      sections.push(renderProject(project))
      continue
    }
    if (renderedGroupIds.has(group.id)) continue
    renderedGroupIds.add(group.id)
    sections.push(
      renderGroup(
        group,
        projects.filter((entry) => entry.groupId === group.id),
      ),
    )
  }
  for (const group of groups) {
    if (renderedGroupIds.has(group.id)) continue
    renderedGroupIds.add(group.id)
    sections.push(
      renderGroup(
        group,
        projects.filter((entry) => entry.groupId === group.id),
      ),
    )
  }

  return (
    <aside
      ref={ref}
      className="sidebar"
      style={width === null ? undefined : { width }}
    >
      <div className="sidebar-aurora" aria-hidden="true" />

      <header className="sidebar-top" data-tauri-drag-region="deep">
        <div className="workspace-switch-wrap" ref={workspaceSwitchRef}>
          <button
            className={`workspace-switcher ${workspaceMenuOpen ? 'is-open' : ''}`}
            type="button"
            title={activeWorkspaceName}
            aria-haspopup="menu"
            aria-expanded={workspaceMenuOpen}
            onClick={() => setWorkspaceMenuOpen((open) => !open)}
          >
            <span className="workspace-avatar">
              {workspaceInitial(workspace, t)}
            </span>
            <span className="workspace-copy">
              <strong>{activeWorkspaceName}</strong>
              <span>{repoCountLabel(projects.length)}</span>
            </span>
            <ChevronDown size={14} className="workspace-chevron" />
          </button>

          {workspaceMenuOpen && (
            <div className="workspace-menu" role="menu">
              <div className="workspace-menu-label">
                {t('sidebar.workspaces')}
              </div>
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
                      <span>{repoCountLabel(entry.projects.length)}</span>
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

        <button
          className="icon-button subtle sidebar-collapse"
          onClick={onCollapse}
          title={`${t('sidebar.collapse')}  ⌘\\`}
          aria-label={t('sidebar.collapse')}
        >
          <PanelLeftClose size={16} strokeWidth={1.8} />
        </button>
      </header>

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
        {projects.length === 0 && groups.length === 0 && (
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
        {sections}
      </div>

      <footer className="sidebar-footer">
        <button className="quick-open-row" onClick={onQuickOpen}>
          <span className="quick-open-copy">
            <Command size={13} strokeWidth={1.9} />
            {t('sidebar.quickOpen')}
          </span>
          <kbd>⌘ K</kbd>
        </button>
        <div className="watch-row" title={t('sidebar.autoRefresh')}>
          <span className="watch-dot" />
          <span>{t('sidebar.localGit')}</span>
          <span className="watch-latency">⌘R</span>
        </div>
      </footer>
    </aside>
  )
}

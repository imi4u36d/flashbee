import { useEffect, useLayoutEffect, useMemo, useRef, useState } from 'react'
import { CheckCircle2, LoaderCircle } from 'lucide-react'
import { CleanState } from './components/CleanState'
import { CommandPalette } from './components/CommandPalette'
import { DiffViewer } from './components/DiffViewer'
import { I18nProvider } from './components/I18nProvider'
import {
  AddRepositoryDialog,
  ConfirmRemoveDialog,
  CreateWorkspaceDialog,
} from './components/ManageDialogs'
import { RepoBar } from './components/RepoBar'
import { SettingsModal } from './components/SettingsModal'
import { Sidebar } from './components/Sidebar'
import { WorkspaceEmptyState } from './components/WorkspaceEmptyState'
import { fileStats } from './data/projects'
import { createTranslator, resolveLocale, type TranslationKey } from './lib/i18n'
import { warmHighlighter } from './lib/highlight'
import {
  fetchRepositorySnapshot,
  snapshotFiles,
} from './lib/repositoryApi'
import {
  DEFAULT_SETTINGS,
  applyDocumentSettings,
  loadSettings,
  resolveTheme,
  saveSettings,
  systemPrefersDark,
  type AppSettings,
  type ResolvedTheme,
} from './lib/settings'
import {
  createRepository,
  createWorkspace,
  initialSelectionForWorkspaces,
  loadWorkspaceState,
  saveWorkspaceState,
  workspaceLabel,
  type NewRepositoryInput,
  type WorkspaceState,
} from './lib/workspaces'
import type { DiffFile, DiffMode, Project } from './types'

type Toast = {
  id: number
  key: TranslationKey
  vars?: Record<string, string | number>
}

type ManageDialog =
  | { kind: 'create-workspace' }
  | { kind: 'add-repository' }
  | { kind: 'delete-workspace' }
  | { kind: 'delete-repository' }
  | null

export default function App() {
  const [workspaceState, setWorkspaceState] =
    useState<WorkspaceState>(loadWorkspaceState)
  const initialWorkspace =
    workspaceState.workspaces.find(
      (workspace) => workspace.id === workspaceState.activeWorkspaceId,
    ) ?? workspaceState.workspaces[0]
  const [activeProjectId, setActiveProjectId] = useState(
    initialWorkspace.projects[0]?.id ?? '',
  )
  const [expandedProjectId, setExpandedProjectId] = useState<string | null>(
    initialWorkspace.projects[0]?.id ?? null,
  )
  const [selectedByProject, setSelectedByProject] = useState<
    Record<string, string>
  >(() => initialSelectionForWorkspaces(workspaceState.workspaces))
  const [mode, setMode] = useState<DiffMode>('split')
  const [paletteOpen, setPaletteOpen] = useState(false)
  const [settingsOpen, setSettingsOpen] = useState(false)
  const [manageDialog, setManageDialog] = useState<ManageDialog>(null)
  const [settings, setSettings] = useState<AppSettings>(loadSettings)
  const [systemDark, setSystemDark] = useState(systemPrefersDark)
  const [sidebarCollapsed, setSidebarCollapsed] = useState(false)
  const [refreshing, setRefreshing] = useState(false)
  const [loadingRepositoryId, setLoadingRepositoryId] = useState<string | null>(null)
  const [toast, setToast] = useState<Toast | null>(null)
  const [, setHighlightReady] = useState(false)
  const previousTheme = useRef<ResolvedTheme | null>(
    (document.documentElement.dataset.theme as ResolvedTheme | undefined) ?? null,
  )
  const themeTransitionTimer = useRef<number | null>(null)

  const resolvedTheme = resolveTheme(settings.theme, systemDark)
  const locale = resolveLocale(settings.locale)
  const t = useMemo(() => createTranslator(locale), [locale])

  const { workspaces } = workspaceState
  const activeWorkspace =
    workspaces.find(
      (workspace) => workspace.id === workspaceState.activeWorkspaceId,
    ) ?? workspaces[0]
  const workspaceProjects = activeWorkspace.projects
  const activeWorkspaceName = workspaceLabel(activeWorkspace, t)
  const activeProject =
    workspaceProjects.find((project) => project.id === activeProjectId) ??
    workspaceProjects[0] ??
    null

  const activeFile = useMemo(() => {
    if (!activeProject) return null
    const selectedPath = selectedByProject[activeProject.id]
    return (
      activeProject.files.find((file) => file.path === selectedPath) ??
      activeProject.files[0] ??
      null
    )
  }, [activeProject, selectedByProject])

  const fileIndex = activeFile && activeProject
    ? activeProject.files.findIndex((file) => file.path === activeFile.path)
    : -1

  const totalChanges = useMemo(
    () =>
      workspaceProjects.reduce(
        (total, project) =>
          total +
          project.files.reduce((sum, file) => {
            const stats = fileStats(file)
            return sum + stats.additions + stats.deletions
          }, 0),
        0,
      ),
    [workspaceProjects],
  )

  const existingWorkspaceNames = useMemo(
    () =>
      Array.from(
        new Set(
          workspaces.flatMap((workspace) => [
            workspace.name,
            workspaceLabel(workspace, t),
          ]),
        ),
      ),
    [workspaces, t],
  )

  const existingRepositoryPaths = useMemo(
    () => workspaceProjects.map((project) => project.path),
    [workspaceProjects],
  )

  const showToast = (
    key: TranslationKey,
    vars?: Record<string, string | number>,
  ) => {
    setToast({ id: Date.now(), key, vars })
  }

  useEffect(() => {
    warmHighlighter().then(() => setHighlightReady(true))
  }, [])

  useEffect(() => {
    saveWorkspaceState(workspaceState)
  }, [workspaceState])

  useEffect(() => {
    const query = window.matchMedia('(prefers-color-scheme: dark)')
    const sync = (event: MediaQueryListEvent) => setSystemDark(event.matches)

    query.addEventListener('change', sync)
    return () => query.removeEventListener('change', sync)
  }, [])

  useEffect(() => {
    document.documentElement.lang = locale === 'zh' ? 'zh-CN' : 'en'
    document.title = t('app.title')
  }, [locale, t])

  useLayoutEffect(() => {
    const root = document.documentElement
    const themeChanged =
      previousTheme.current !== null && previousTheme.current !== resolvedTheme
    const reduceMotion = window.matchMedia('(prefers-reduced-motion: reduce)').matches
    const transitionDocument = document as Document & {
      startViewTransition?: (callback: () => void) => { finished: Promise<void> }
    }

    if (themeTransitionTimer.current) {
      window.clearTimeout(themeTransitionTimer.current)
      themeTransitionTimer.current = null
    }
    root.classList.remove('theme-transition')

    if (themeChanged && transitionDocument.startViewTransition && !reduceMotion) {
      try {
        const transition = transitionDocument.startViewTransition(() => {
          applyDocumentSettings(settings, resolvedTheme)
        })
        transition.finished.catch(() => undefined)
      } catch {
        applyDocumentSettings(settings, resolvedTheme)
      }
    } else {
      if (themeChanged && !reduceMotion) {
        root.classList.add('theme-transition')
        themeTransitionTimer.current = window.setTimeout(() => {
          root.classList.remove('theme-transition')
          themeTransitionTimer.current = null
        }, 460)
      }
      applyDocumentSettings(settings, resolvedTheme)
    }

    previousTheme.current = resolvedTheme
    saveSettings(settings)
  }, [resolvedTheme, settings])

  useEffect(() => {
    if (!toast) return
    const timer = window.setTimeout(() => setToast(null), 2600)
    return () => window.clearTimeout(timer)
  }, [toast])

  const applySnapshot = (
    projectId: string,
    snapshot: Awaited<ReturnType<typeof fetchRepositorySnapshot>>,
  ) => {
    setWorkspaceState((current) => ({
      ...current,
      workspaces: current.workspaces.map((workspace) => ({
        ...workspace,
        projects: workspace.projects.map((project) => {
          if (project.id !== projectId) return project
          return {
            ...project,
            files: snapshotFiles(snapshot),
            branches: snapshot.branches,
            branch: snapshot.branches.includes(project.branch)
              ? project.branch
              : snapshot.currentBranch,
            updated: 'now',
          }
        }),
      })),
    }))
  }

  useEffect(() => {
    if (!activeProject || activeProject.source !== 'local') return

    let cancelled = false
    setLoadingRepositoryId(activeProject.id)
    const load = async () => {
      try {
        const snapshot = await fetchRepositorySnapshot(activeProject.path)
        if (cancelled) return
        applySnapshot(activeProject.id, snapshot)
      } catch {
        if (!cancelled) {
          showToast('repo.loadFailed', { name: activeProject.name })
        }
      } finally {
        if (!cancelled) setLoadingRepositoryId(null)
      }
    }

    void load()
    return () => {
      cancelled = true
    }
    // Refresh when a different local repository becomes active.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [activeProject?.id, activeProject?.path, activeProject?.source])

  const queryForWorkspace = (workspaceId: string) => {
    const target = workspaces.find((workspace) => workspace.id === workspaceId)
    if (!target || target.id === activeWorkspace.id) return
    setWorkspaceState((current) => ({
      ...current,
      activeWorkspaceId: target.id,
    }))
    const firstProject = target.projects[0]
    setActiveProjectId(firstProject?.id ?? '')
    setExpandedProjectId(firstProject?.id ?? null)
    setSelectedByProject((current) => {
      const next = { ...current }
      for (const project of target.projects) {
        if (!(project.id in next)) {
          next[project.id] = project.files[0]?.path ?? ''
        }
      }
      return next
    })
    setPaletteOpen(false)
    setSettingsOpen(false)
  }

  const activateProject = (project: Project) => {
    setActiveProjectId(project.id)
    setExpandedProjectId(project.id)
    setSelectedByProject((current) => ({
      ...current,
      [project.id]: current[project.id] || project.files[0]?.path || '',
    }))
  }

  const handleProjectClick = (project: Project) => {
    if (project.id === activeProject?.id) {
      setExpandedProjectId((current) =>
        current === project.id ? null : project.id,
      )
      return
    }
    activateProject(project)
  }

  const selectFile = (project: Project, file: DiffFile) => {
    setActiveProjectId(project.id)
    setExpandedProjectId(project.id)
    setSelectedByProject((current) => ({
      ...current,
      [project.id]: file.path,
    }))
  }

  const goToFile = (offset: number) => {
    if (!activeFile || !activeProject || fileIndex < 0) return
    const nextFile = activeProject.files[fileIndex + offset]
    if (!nextFile) return
    setSelectedByProject((current) => ({
      ...current,
      [activeProject.id]: nextFile.path,
    }))
  }

  const handleRefresh = () => {
    if (refreshing) return

    if (activeProject?.source === 'local') {
      setRefreshing(true)
      fetchRepositorySnapshot(activeProject.path)
        .then((snapshot) => {
          applySnapshot(activeProject.id, snapshot)
          showToast('app.repoRefreshed', { count: snapshot.files.length })
        })
        .catch(() => showToast('repo.refreshFailed'))
        .finally(() => setRefreshing(false))
      return
    }

    setRefreshing(true)
    window.setTimeout(() => {
      setRefreshing(false)
      showToast('app.refreshed')
    }, 750)
  }

  const handleSelectBranch = (branch: string) => {
    if (!activeProject) return
    setWorkspaceState((current) => ({
      ...current,
      workspaces: current.workspaces.map((workspace) => ({
        ...workspace,
        projects: workspace.projects.map((project) =>
          project.id === activeProject.id ? { ...project, branch } : project,
        ),
      })),
    }))
  }

  const updateSettings = (patch: Partial<AppSettings>) => {
    setSettings((current) => ({ ...current, ...patch }))
  }

  const resetSettings = () => {
    setSettings(DEFAULT_SETTINGS)
    showToast('settings.resetToast')
  }

  const openSettings = () => {
    setPaletteOpen(false)
    setManageDialog(null)
    setSettingsOpen(true)
  }

  const openAddRepository = () => {
    if (!activeWorkspace) return
    setPaletteOpen(false)
    setSettingsOpen(false)
    setManageDialog({ kind: 'add-repository' })
  }

  const openCreateWorkspace = () => {
    setPaletteOpen(false)
    setSettingsOpen(false)
    setManageDialog({ kind: 'create-workspace' })
  }

  const requestDeleteWorkspace = () => {
    if (workspaces.length <= 1) {
      showToast('sidebar.keepOneWorkspace')
      return
    }
    setManageDialog({ kind: 'delete-workspace' })
  }

  const requestDeleteRepository = () => {
    if (!activeProject) return
    setManageDialog({ kind: 'delete-repository' })
  }

  const handleCreateWorkspace = (name: string) => {
    const workspace = createWorkspace(name)
    setWorkspaceState((current) => ({
      workspaces: [...current.workspaces, workspace],
      activeWorkspaceId: workspace.id,
    }))
    setActiveProjectId('')
    setExpandedProjectId(null)
    setPaletteOpen(false)
    setManageDialog(null)
    showToast('workspace.created', { name })
  }

  const handleAddRepository = (input: NewRepositoryInput) => {
    if (!activeWorkspace) return
    const project = createRepository(input, activeWorkspace.projects)
    setWorkspaceState((current) => ({
      ...current,
      workspaces: current.workspaces.map((workspace) =>
        workspace.id === activeWorkspace.id
          ? { ...workspace, projects: [...workspace.projects, project] }
          : workspace,
      ),
    }))
    setActiveProjectId(project.id)
    setExpandedProjectId(project.id)
    setSelectedByProject((current) => ({
      ...current,
      [project.id]: project.files[0]?.path ?? '',
    }))
    setManageDialog(null)
    setPaletteOpen(false)
    showToast('repo.created', { name: project.name })
  }

  const handleRemoveRepository = () => {
    if (!activeProject) return
    const removed = activeProject
    const remaining = activeWorkspace.projects.filter(
      (project) => project.id !== removed.id,
    )
    setWorkspaceState((current) => ({
      ...current,
      workspaces: current.workspaces.map((workspace) =>
        workspace.id === activeWorkspace.id
          ? { ...workspace, projects: remaining }
          : workspace,
      ),
    }))
    setActiveProjectId(remaining[0]?.id ?? '')
    setExpandedProjectId(remaining[0]?.id ?? null)
    setSelectedByProject((current) => {
      const next = { ...current }
      delete next[removed.id]
      return next
    })
    setManageDialog(null)
    setPaletteOpen(false)
    showToast('repo.removed', { name: removed.name })
  }

  const handleDeleteWorkspace = () => {
    if (workspaces.length <= 1) {
      setManageDialog(null)
      showToast('sidebar.keepOneWorkspace')
      return
    }
    const remaining = workspaces.filter(
      (workspace) => workspace.id !== activeWorkspace.id,
    )
    const nextWorkspace = remaining[0]
    const removedName = activeWorkspaceName
    setWorkspaceState({
      workspaces: remaining,
      activeWorkspaceId: nextWorkspace.id,
    })
    setActiveProjectId(nextWorkspace.projects[0]?.id ?? '')
    setExpandedProjectId(nextWorkspace.projects[0]?.id ?? null)
    setSelectedByProject((current) => {
      const next = { ...current }
      for (const project of nextWorkspace.projects) {
        if (!(project.id in next)) {
          next[project.id] = project.files[0]?.path ?? ''
        }
      }
      return next
    })
    setManageDialog(null)
    setPaletteOpen(false)
    showToast('workspace.deleted', { name: removedName })
  }

  useEffect(() => {
    const onKeyDown = (event: KeyboardEvent) => {
      const target = event.target as HTMLElement | null
      const typing =
        target instanceof HTMLInputElement ||
        target instanceof HTMLTextAreaElement ||
        target?.isContentEditable

      if (manageDialog) {
        if (event.key === 'Escape') {
          setManageDialog(null)
        }
        return
      }

      if ((event.metaKey || event.ctrlKey) && event.key === ',') {
        event.preventDefault()
        setPaletteOpen(false)
        setSettingsOpen((current) => !current)
        return
      }

      if (settingsOpen) {
        if (event.key === 'Escape') {
          setSettingsOpen(false)
        }
        return
      }

      if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === 'k') {
        event.preventDefault()
        setPaletteOpen(true)
        return
      }

      if ((event.metaKey || event.ctrlKey) && event.key === '\\') {
        event.preventDefault()
        setSidebarCollapsed((current) => !current)
        return
      }

      if (event.key === 'Escape') {
        if (paletteOpen) {
          setPaletteOpen(false)
        }
        return
      }

      if (typing || paletteOpen) return

      if (event.key.toLowerCase() === 'j') {
        event.preventDefault()
        goToFile(1)
      }
      if (event.key.toLowerCase() === 'k') {
        event.preventDefault()
        goToFile(-1)
      }
      if (event.key.toLowerCase() === 's') {
        event.preventDefault()
        setMode((current) => (current === 'split' ? 'unified' : 'split'))
      }
    }

    window.addEventListener('keydown', onKeyDown)
    return () => window.removeEventListener('keydown', onKeyDown)
  })

  return (
    <I18nProvider locale={locale}>
      <div className={`app-shell ${sidebarCollapsed ? 'is-sidebar-collapsed' : ''}`}>
        {!sidebarCollapsed && (
          <Sidebar
            workspace={activeWorkspace}
            workspaces={workspaces}
            projects={workspaceProjects}
            activeProjectId={activeProject?.id ?? ''}
            expandedProjectId={expandedProjectId}
            selectedFile={activeFile?.path ?? null}
            onProjectClick={handleProjectClick}
            onSelectFile={selectFile}
            onSwitchWorkspace={queryForWorkspace}
            onCreateWorkspace={openCreateWorkspace}
            onRequestDeleteWorkspace={requestDeleteWorkspace}
            onAddRepository={openAddRepository}
            onQuickOpen={() => setPaletteOpen(true)}
            onCollapse={() => setSidebarCollapsed(true)}
          />
        )}

        <div className="app-main">
          <RepoBar
            project={activeProject}
            workspaceName={activeWorkspaceName}
            sidebarCollapsed={sidebarCollapsed}
            refreshing={refreshing}
            onOpenSidebar={() => setSidebarCollapsed(false)}
            onOpenSettings={openSettings}
            onQuickOpen={() => setPaletteOpen(true)}
            onRefresh={handleRefresh}
            onSelectBranch={handleSelectBranch}
            onAddRepository={openAddRepository}
            onRequestRemoveRepository={requestDeleteRepository}
          />

          <div className="workbench">
            <main className="main-pane">
              {activeFile && activeProject ? (
                <DiffViewer
                  key={`${activeProject.id}-${activeFile.path}`}
                  file={activeFile}
                  mode={mode}
                  baseBranch="main"
                  appTheme={resolvedTheme}
                  fileIndex={fileIndex}
                  fileCount={activeProject.files.length}
                  hasPrev={fileIndex > 0}
                  hasNext={fileIndex < activeProject.files.length - 1}
                  onModeChange={setMode}
                  onPrev={() => goToFile(-1)}
                  onNext={() => goToFile(1)}
                  onToast={showToast}
                />
              ) : activeProject ? (
                activeProject.source === 'local' &&
                loadingRepositoryId === activeProject.id &&
                activeProject.files.length === 0 ? (
                  <div className="clean-pane">
                    <div className="repo-loading-state" role="status">
                      <span className="repo-loading-orb">
                        <LoaderCircle size={19} strokeWidth={2} className="is-spinning" />
                      </span>
                      <span>{t('repo.loadingChanges')}</span>
                    </div>
                  </div>
                ) : (
                  <div className="clean-pane">
                    <CleanState project={activeProject} onRefresh={handleRefresh} />
                  </div>
                )
              ) : (
                <WorkspaceEmptyState
                  workspaceName={activeWorkspaceName}
                  onAddRepository={openAddRepository}
                />
              )}
            </main>
          </div>
        </div>

        {paletteOpen && (
          <CommandPalette
            open={paletteOpen}
            projects={workspaceProjects}
            activeProjectId={activeProject?.id ?? ''}
            onClose={() => setPaletteOpen(false)}
            onSelectProject={activateProject}
            onSelectFile={selectFile}
          />
        )}

        {settingsOpen && (
          <SettingsModal
            settings={settings}
            systemDark={systemDark}
            onChange={updateSettings}
            onReset={resetSettings}
            onClose={() => setSettingsOpen(false)}
          />
        )}

        {manageDialog?.kind === 'create-workspace' && (
          <CreateWorkspaceDialog
            existingNames={existingWorkspaceNames}
            onConfirm={handleCreateWorkspace}
            onClose={() => setManageDialog(null)}
          />
        )}

        {manageDialog?.kind === 'add-repository' && (
          <AddRepositoryDialog
            workspaceName={activeWorkspaceName}
            existingPaths={existingRepositoryPaths}
            onConfirm={handleAddRepository}
            onClose={() => setManageDialog(null)}
          />
        )}

        {manageDialog?.kind === 'delete-workspace' && (
          <ConfirmRemoveDialog
            title={t('workspace.deleteTitle')}
            description={t('workspace.deleteDescription', {
              name: activeWorkspaceName,
            })}
            confirmLabel={t('sidebar.deleteWorkspace')}
            onConfirm={handleDeleteWorkspace}
            onClose={() => setManageDialog(null)}
          />
        )}

        {manageDialog?.kind === 'delete-repository' && activeProject && (
          <ConfirmRemoveDialog
            title={t('repo.removeTitle')}
            description={t('repo.removeDescription', {
              name: activeProject.name,
            })}
            confirmLabel={t('repo.remove')}
            onConfirm={handleRemoveRepository}
            onClose={() => setManageDialog(null)}
          />
        )}

        {toast && (
          <div className="toast" key={toast.id} role="status">
            <CheckCircle2 size={15} strokeWidth={2} />
            <span>{t(toast.key, toast.vars)}</span>
          </div>
        )}

        <div className="sr-only" aria-live="polite">
          {activeFile && activeProject
            ? t('app.srSummary', {
                project: activeProject.name,
                path: activeFile.path,
                count: totalChanges,
              })
            : activeProject
              ? t('app.srClean')
              : t('workspace.emptyTitle')}
        </div>
      </div>
    </I18nProvider>
  )
}

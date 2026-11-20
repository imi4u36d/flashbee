import { useEffect, useLayoutEffect, useMemo, useRef, useState } from 'react'
import type { MouseEvent as ReactMouseEvent } from 'react'
import { CheckCircle2, LoaderCircle, TriangleAlert } from 'lucide-react'
import { CleanState } from './components/CleanState'
import { CommandPalette } from './components/CommandPalette'
import { ConflictResolver } from './components/ConflictResolver'
import { DiffViewer } from './components/DiffViewer'
import { I18nProvider } from './components/I18nProvider'
import {
  AddRepositoryDialog,
  ConfirmRemoveDialog,
  CreateWorkspaceDialog,
} from './components/ManageDialogs'
import { PushDialog } from './components/PushDialog'
import { RemoteDialog } from './components/RemoteDialog'
import { RepoBar } from './components/RepoBar'
import { SettingsModal } from './components/SettingsModal'
import { Sidebar } from './components/Sidebar'
import { SidebarResizer } from './components/SidebarResizer'
import { WorkspaceEmptyState } from './components/WorkspaceEmptyState'
import { fileStats } from './data/projects'
import { copyText, showContextMenu, type ContextMenuEntry } from './lib/contextMenu'
import { createTranslator, resolveLocale, type TranslationKey } from './lib/i18n'
import { warmHighlighter } from './lib/highlight'
import { loadSidebarWidth, saveSidebarWidth } from './lib/layout'
import { invokeNative, isNative, onMenuCommand } from './lib/native'
import {
  RepositoryApiError,
  abortMerge,
  completeMerge,
  discoverRepositoryTarget,
  fetchRepositorySnapshot,
  pullRepository,
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
  createProjectGroup,
  createRepository,
  createSampleWorkspace,
  createWorkspace,
  initialSelectionForWorkspaces,
  loadWorkspaceState,
  saveWorkspaceState,
  workspaceLabel,
  type RepositoryAddRequest,
  type WorkspaceState,
} from './lib/workspaces'
import type { DiffFile, DiffMode, Project, ProjectGroup } from './types'

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
  | { kind: 'delete-group'; group: ProjectGroup }
  | { kind: 'remote-settings' }
  | { kind: 'abort-merge' }
  | null

function compactErrorMessage(reason: unknown) {
  const message =
    reason instanceof Error
      ? reason.message
      : typeof reason === 'string'
        ? reason
        : ''
  const firstLine =
    message
      .split('\n')
      .map((line) => line.trim())
      .find((line) => line.length > 0) ?? ''
  if (!firstLine) return ''
  return firstLine.length > 160 ? `${firstLine.slice(0, 157)}…` : firstLine
}

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
  const [pushDialogOpen, setPushDialogOpen] = useState(false)
  const [settings, setSettings] = useState<AppSettings>(loadSettings)
  const [systemDark, setSystemDark] = useState(systemPrefersDark)
  const [sidebarCollapsed, setSidebarCollapsed] = useState(false)
  const [sidebarWidth, setSidebarWidth] = useState<number | null>(loadSidebarWidth)
  const sidebarRef = useRef<HTMLElement>(null)
  const [refreshing, setRefreshing] = useState(false)
  const [syncing, setSyncing] = useState<'pull' | null>(null)
  const [merging, setMerging] = useState(false)
  const [resolverOpen, setResolverOpen] = useState(false)
  const [scanningGroupId, setScanningGroupId] = useState<string | null>(null)
  const [loadingRepositoryId, setLoadingRepositoryId] = useState<string | null>(
    () => {
      const initialWorkspaceEntry =
        workspaceState.workspaces.find(
          (workspace) => workspace.id === workspaceState.activeWorkspaceId,
        ) ?? workspaceState.workspaces[0]
      const initialProject = initialWorkspaceEntry?.projects[0]
      return initialProject?.source === 'local' ? initialProject.id : null
    },
  )
  const [toast, setToast] = useState<Toast | null>(null)
  const [, setHighlightReady] = useState(false)
  const previousTheme = useRef<ResolvedTheme | null>(
    (document.documentElement.dataset.theme as ResolvedTheme | undefined) ?? null,
  )
  const themeTransitionTimer = useRef<number | null>(null)
  const menuActionRef = useRef<(command: string) => void>(() => undefined)
  const refreshOnFocusRef = useRef<() => void>(() => undefined)

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
  const activeProjectName = activeProject?.name
  const conflictCount = activeProject?.conflicts?.length ?? 0
  const mergeInProgress = Boolean(activeProject?.mergeInProgress)
  const hasMergeWork =
    activeProject?.source === 'local' &&
    (conflictCount > 0 || mergeInProgress)
  const showConflictResolver = hasMergeWork && resolverOpen
  const showConflictBanner = hasMergeWork && !resolverOpen
  const mergeReady = mergeInProgress && conflictCount === 0

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
    // Opening a repository that still has merge work should land on the
    // resolver instead of hiding it behind the diff view.
    if (hasMergeWork) setResolverOpen(true)
  }, [activeProject?.id, conflictCount, mergeInProgress, hasMergeWork])

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
    if (isNative) {
      void invokeNative('set_menu_locale', { locale }).catch(() => undefined)
    }
  }, [locale])

  useEffect(() => {
    const title = activeProjectName
      ? `${activeProjectName} — FlashBee`
      : 'FlashBee'
    document.title = title
    if (isNative) {
      void invokeNative('set_window_title', { title }).catch(() => undefined)
    }
  }, [activeProjectName])

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

  useEffect(
    () => onMenuCommand((command) => menuActionRef.current(command)),
    [],
  )

  useEffect(() => {
    if (!isNative) return
    const refresh = () => refreshOnFocusRef.current()
    window.addEventListener('focus', refresh)
    return () => window.removeEventListener('focus', refresh)
  }, [])

  const applySnapshot = (
    projectId: string,
    snapshot: Awaited<ReturnType<typeof fetchRepositorySnapshot>>,
  ) => {
    const remoteBranches = snapshot.remoteBranches ?? []
    const knownBranches = [
      ...snapshot.branches,
      ...remoteBranches.map((branch) => branch.name),
    ]
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
            currentBranch: snapshot.currentBranch,
            remoteBranches,
            remotes: snapshot.remotes ?? [],
            upstream: snapshot.upstream,
            ahead: snapshot.ahead ?? 0,
            behind: snapshot.behind ?? 0,
            conflicts: snapshot.conflicts ?? [],
            mergeInProgress: Boolean(snapshot.mergeInProgress),
            branch: knownBranches.includes(project.branch)
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
        const snapshot = await fetchRepositorySnapshot(
          activeProject.path,
          activeProject.branch,
        )
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
    // Refresh when the active repository or comparison branch changes.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [
    activeProject?.id,
    activeProject?.path,
    activeProject?.source,
    activeProject?.branch,
  ])

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

  const revealProjectGroup = (project: Project) => {
    if (!project.groupId) return
    setWorkspaceState((current) => {
      const target = current.workspaces.find(
        (workspace) => workspace.id === activeWorkspace.id,
      )
      const collapsed = target?.groups?.some(
        (group) => group.id === project.groupId && group.collapsed,
      )
      if (!collapsed) return current

      return {
        ...current,
        workspaces: current.workspaces.map((workspace) =>
          workspace.id === activeWorkspace.id
            ? {
                ...workspace,
                groups: (workspace.groups ?? []).map((group) =>
                  group.id === project.groupId
                    ? { ...group, collapsed: false }
                    : group,
                ),
              }
            : workspace,
        ),
      }
    })
  }

  const activateProject = (project: Project) => {
    revealProjectGroup(project)
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
    revealProjectGroup(project)
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
      fetchRepositorySnapshot(activeProject.path, activeProject.branch)
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

  const reloadProjectSnapshot = async (project: Project) => {
    const snapshot = await fetchRepositorySnapshot(project.path, project.branch)
    applySnapshot(project.id, snapshot)
  }

  const handlePull = () => {
    if (!activeProject || activeProject.source !== 'local' || syncing) return
    const project = activeProject
    setSyncing('pull')
    pullRepository(project.path)
      .then((outcome) => {
        if (outcome.conflicts.length > 0) {
          setResolverOpen(true)
          showToast('repo.pullConflicts', { count: outcome.conflicts.length })
        } else {
          showToast('repo.pulled')
        }
      })
      .catch((reason) => {
        const code = reason instanceof RepositoryApiError ? reason.code : ''
        if (code === 'no_remote') {
          showToast('repo.noRemote')
        } else if (code === 'no_upstream') {
          showToast('repo.noUpstream')
        } else {
          showToast('repo.pullFailed', {
            message: compactErrorMessage(reason) || t('repo.actionFailed'),
          })
        }
      })
      .finally(() => {
        setSyncing(null)
        void reloadProjectSnapshot(project).catch(() => undefined)
      })
  }

  const handlePush = () => {
    if (
      !activeProject ||
      activeProject.source !== 'local' ||
      syncing ||
      conflictCount > 0 ||
      mergeInProgress
    ) {
      return
    }
    setPushDialogOpen(true)
  }

  const handlePushed = () => {
    setPushDialogOpen(false)
    showToast('repo.pushed')
    if (activeProject?.source === 'local') {
      void reloadProjectSnapshot(activeProject).catch(() => undefined)
    }
  }

  const handleRemoteSaved = () => {
    setManageDialog(null)
    showToast('repo.remoteSaved')
    if (activeProject?.source === 'local') {
      void reloadProjectSnapshot(activeProject).catch(() => undefined)
    }
  }

  const handleConflictsResolved = async () => {
    if (!activeProject || activeProject.source !== 'local') return
    try {
      await reloadProjectSnapshot(activeProject)
    } catch {
      showToast('repo.refreshFailed')
    }
  }

  const handleCompleteMerge = () => {
    if (!activeProject || activeProject.source !== 'local' || merging) return
    const project = activeProject
    setMerging(true)
    completeMerge(project.path)
      .then(async () => {
        await reloadProjectSnapshot(project)
        setResolverOpen(false)
        showToast('conflict.mergeCompleted')
      })
      .catch(() => showToast('conflict.completeFailed'))
      .finally(() => setMerging(false))
  }

  const handleAbortMerge = () => {
    if (!activeProject || merging) return
    setManageDialog({ kind: 'abort-merge' })
  }

  const confirmAbortMerge = () => {
    if (!activeProject || activeProject.source !== 'local' || merging) return
    const project = activeProject
    setManageDialog(null)
    setMerging(true)
    abortMerge(project.path)
      .then(async () => {
        await reloadProjectSnapshot(project)
        setResolverOpen(false)
        showToast('conflict.mergeAborted')
      })
      .catch(() => showToast('conflict.abortFailed'))
      .finally(() => setMerging(false))
  }

  const updateSettings = (patch: Partial<AppSettings>) => {
    setSettings((current) => ({ ...current, ...patch }))
  }

  const resetSettings = () => {
    setSettings(DEFAULT_SETTINGS)
    showToast('settings.resetToast')
  }

  const commitSidebarWidth = (width: number) => {
    setSidebarWidth(width)
    saveSidebarWidth(width)
  }

  const resetSidebarWidth = () => {
    setSidebarWidth(null)
    saveSidebarWidth(null)
  }

  const openSettings = () => {
    setPaletteOpen(false)
    setManageDialog(null)
    setSettingsOpen(true)
  }

  const openPalette = () => {
    setSettingsOpen(false)
    setManageDialog(null)
    setPaletteOpen(true)
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

  const hydrateProjects = (projects: Project[]) => {
    // Repositories are read one by one so the sidebar shows real change counts
    // without spawning a burst of Git processes.
    void (async () => {
      for (const project of projects) {
        try {
          const snapshot = await fetchRepositorySnapshot(
            project.path,
            project.branch,
          )
          applySnapshot(project.id, snapshot)
        } catch {
          // Leave it empty until the user opens it.
        }
      }
    })()
  }

  const handleAddRepositories = (request: RepositoryAddRequest) => {
    if (!activeWorkspace || request.repositories.length === 0) return

    const knownPaths = new Set(
      activeWorkspace.projects.map((project) => project.path),
    )
    const additions = request.repositories.filter((input) => {
      if (knownPaths.has(input.path)) return false
      knownPaths.add(input.path)
      return true
    })
    if (additions.length === 0) return

    const groups = activeWorkspace.groups ?? []
    let group: ProjectGroup | undefined
    let newGroup: ProjectGroup | undefined
    if (request.group) {
      group = groups.find((entry) => entry.path === request.group?.path)
      if (!group) {
        group = createProjectGroup(request.group)
        newGroup = group
      }
    }

    const created: Project[] = []
    for (const input of additions) {
      created.push(
        createRepository(
          input,
          [...activeWorkspace.projects, ...created],
          group?.id,
        ),
      )
    }

    setWorkspaceState((current) => ({
      ...current,
      workspaces: current.workspaces.map((workspace) => {
        if (workspace.id !== activeWorkspace.id) return workspace

        const nextGroups = newGroup
          ? [...(workspace.groups ?? []), newGroup]
          : group
            ? (workspace.groups ?? []).map((entry) =>
                entry.id === group?.id
                  ? { ...entry, collapsed: false }
                  : entry,
              )
            : workspace.groups

        return {
          ...workspace,
          groups: nextGroups,
          projects: [...workspace.projects, ...created],
        }
      }),
    }))
    setActiveProjectId(created[0].id)
    setExpandedProjectId(created[0].id)
    setSelectedByProject((current) => {
      const next = { ...current }
      for (const project of created) {
        next[project.id] = project.files[0]?.path ?? ''
      }
      return next
    })
    setManageDialog(null)
    setPaletteOpen(false)

    if (group) {
      showToast(
        created.length === 1
          ? 'repo.groupCreatedOne'
          : 'repo.groupCreatedMany',
        { name: group.name, count: created.length },
      )
    } else {
      showToast(
        created.length === 1 ? 'repo.created' : 'repo.createdMany',
        created.length === 1
          ? { name: created[0].name }
          : { count: created.length },
      )
    }

    // The first repository is hydrated by the active-repository effect.
    hydrateProjects(created.slice(1))
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

  const toggleGroup = (group: ProjectGroup) => {
    setWorkspaceState((current) => ({
      ...current,
      workspaces: current.workspaces.map((workspace) =>
        workspace.id === activeWorkspace.id
          ? {
              ...workspace,
              groups: (workspace.groups ?? []).map((entry) =>
                entry.id === group.id
                  ? { ...entry, collapsed: !entry.collapsed }
                  : entry,
              ),
            }
          : workspace,
      ),
    }))
  }

  const rescanGroup = (group: ProjectGroup) => {
    if (scanningGroupId) return

    setScanningGroupId(group.id)
    const workspaceId = activeWorkspace.id
    const knownPaths = new Set(
      activeWorkspace.projects.map((project) => project.path),
    )

    void discoverRepositoryTarget(group.path)
      .then((discovery) => {
        const candidates = discovery.repositories.filter(
          (entry) => entry.path !== group.path && !knownPaths.has(entry.path),
        )
        if (candidates.length === 0) {
          showToast('repo.groupUpToDate', { name: group.name })
          return
        }

        const created: Project[] = []
        for (const entry of candidates) {
          created.push(
            createRepository(
              {
                name: entry.name,
                path: entry.path,
                branch: entry.currentBranch,
                branches: entry.branches,
              },
              [...activeWorkspace.projects, ...created],
              group.id,
            ),
          )
        }

        setWorkspaceState((current) => ({
          ...current,
          workspaces: current.workspaces.map((workspace) =>
            workspace.id === workspaceId
              ? {
                  ...workspace,
                  groups: (workspace.groups ?? []).map((entry) =>
                    entry.id === group.id
                      ? { ...entry, collapsed: false }
                      : entry,
                  ),
                  projects: [...workspace.projects, ...created],
                }
              : workspace,
          ),
        }))
        hydrateProjects(created)
        showToast(
          created.length === 1
            ? 'repo.groupRescannedOne'
            : 'repo.groupRescannedMany',
          { name: group.name, count: created.length },
        )
      })
      .catch(() => showToast('repo.refreshFailed'))
      .finally(() => setScanningGroupId(null))
  }

  const requestDeleteGroup = (group: ProjectGroup) => {
    setManageDialog({ kind: 'delete-group', group })
  }

  const handleRemoveGroup = () => {
    if (manageDialog?.kind !== 'delete-group') return
    const group = manageDialog.group
    const removedIds = new Set(
      activeWorkspace.projects
        .filter((project) => project.groupId === group.id)
        .map((project) => project.id),
    )
    const remaining = activeWorkspace.projects.filter(
      (project) => !removedIds.has(project.id),
    )

    setWorkspaceState((current) => ({
      ...current,
      workspaces: current.workspaces.map((workspace) =>
        workspace.id === activeWorkspace.id
          ? {
              ...workspace,
              projects: remaining,
              groups: (workspace.groups ?? []).filter(
                (entry) => entry.id !== group.id,
              ),
            }
          : workspace,
      ),
    }))

    if (activeProject && removedIds.has(activeProject.id)) {
      setActiveProjectId(remaining[0]?.id ?? '')
      setExpandedProjectId(remaining[0]?.id ?? null)
    } else if (expandedProjectId && removedIds.has(expandedProjectId)) {
      setExpandedProjectId(null)
    }
    setSelectedByProject((current) => {
      const next = { ...current }
      for (const id of removedIds) delete next[id]
      return next
    })
    setManageDialog(null)
    showToast('repo.groupRemoved', { name: group.name })
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

  refreshOnFocusRef.current = () => {
    if (!activeProject || activeProject.source !== 'local' || refreshing) return
    void fetchRepositorySnapshot(activeProject.path, activeProject.branch)
      .then((snapshot) => applySnapshot(activeProject.id, snapshot))
      .catch(() => undefined)
  }

  const refreshProject = (project: Project) => {
    if (project.source !== 'local') {
      showToast('app.refreshed')
      return
    }
    if (activeProject?.id === project.id) {
      handleRefresh()
      return
    }
    fetchRepositorySnapshot(project.path, project.branch)
      .then((snapshot) => {
        applySnapshot(project.id, snapshot)
        showToast('app.repoRefreshed', { count: snapshot.files.length })
      })
      .catch(() => showToast('repo.refreshFailed'))
  }

  const copyPath = (path: string) => {
    void copyText(path)
      .then(() => showToast('diff.pathCopied'))
      .catch(() => showToast('diff.copyFailed'))
  }

  const runPathAction = (command: string, path: string) => {
    if (!isNative) return
    void invokeNative(command, { path }).catch(() => showToast('repo.actionFailed'))
  }

  const handleProjectContextMenu = (
    project: Project,
    event: ReactMouseEvent,
  ) => {
    const local = project.source === 'local'
    const entries: ContextMenuEntry[] = []

    if (local) {
      entries.push({
        label: t('repo.revealInFinder'),
        action: () => runPathAction('reveal_in_finder', project.path),
      })
      entries.push({
        label: t('repo.openInEditor'),
        action: () => runPathAction('open_in_editor', project.path),
      })
      entries.push('separator')
    }

    entries.push({
      label: t('repo.copyPath'),
      action: () => copyPath(project.path),
    })

    if (local) {
      entries.push('separator')
      entries.push({
        label: t('repo.refresh'),
        action: () => refreshProject(project),
      })
    }

    entries.push('separator')
    entries.push({
      label: t('repo.remove'),
      danger: true,
      action: () => {
        activateProject(project)
        setManageDialog({ kind: 'delete-repository' })
      },
    })

    showContextMenu(event, entries)
  }

  const handleGroupContextMenu = (
    group: ProjectGroup,
    event: ReactMouseEvent,
  ) => {
    const entries: ContextMenuEntry[] = [
      {
        label: t('repo.revealInFinder'),
        action: () => runPathAction('reveal_in_finder', group.path),
      },
      {
        label: t('repo.openInEditor'),
        action: () => runPathAction('open_in_editor', group.path),
      },
      'separator',
      {
        label: t('repo.copyPath'),
        action: () => copyPath(group.path),
      },
      'separator',
      {
        label: t('repo.groupRescan'),
        action: () => rescanGroup(group),
      },
      'separator',
      {
        label: t('repo.groupRemove'),
        danger: true,
        action: () => requestDeleteGroup(group),
      },
    ]

    showContextMenu(event, entries)
  }

  const handleFileContextMenu = (
    project: Project,
    file: DiffFile,
    event: ReactMouseEvent,
  ) => {
    activateProject(project)
    setSelectedByProject((current) => ({
      ...current,
      [project.id]: file.path,
    }))

    const local = project.source === 'local'
    const absolutePath = `${project.path.replace(/\/+$/, '')}/${file.path}`
    const entries: ContextMenuEntry[] = []

    if (local) {
      entries.push({
        label: t('repo.revealInFinder'),
        action: () => runPathAction('reveal_in_finder', absolutePath),
      })
      entries.push({
        label: t('repo.openInEditor'),
        action: () => runPathAction('open_in_editor', absolutePath),
      })
      entries.push({
        label: t('repo.openInDefaultApp'),
        action: () => runPathAction('open_path', absolutePath),
      })
      entries.push('separator')
    }

    entries.push({
      label: t('diff.copyPath'),
      action: () => copyPath(local ? absolutePath : file.path),
    })
    entries.push({
      label: t('diff.copyRelativePath'),
      action: () => copyPath(file.path),
    })

    if (local) {
      entries.push('separator')
      entries.push({
        label: t('repo.refresh'),
        action: () => refreshProject(project),
      })
    }

    showContextMenu(event, entries)
  }

  const handleExploreSample = () => {
    const existing = workspaces.find((workspace) => workspace.id === 'sample')
    const sample = existing ?? createSampleWorkspace()

    if (existing) {
      setWorkspaceState((current) => ({
        ...current,
        activeWorkspaceId: 'sample',
      }))
    } else {
      setWorkspaceState((current) => ({
        workspaces: [...current.workspaces, sample],
        activeWorkspaceId: sample.id,
      }))
    }

    setActiveProjectId(sample.projects[0]?.id ?? '')
    setExpandedProjectId(sample.projects[0]?.id ?? null)
    setSelectedByProject((current) => {
      const next = { ...current }
      for (const project of sample.projects) {
        if (!(project.id in next)) {
          next[project.id] = project.files[0]?.path ?? ''
        }
      }
      return next
    })
    setPaletteOpen(false)
    setManageDialog(null)
    showToast('workspace.sampleOpened')
  }

  useEffect(() => {
    const onKeyDown = (event: KeyboardEvent) => {
      const target = event.target as HTMLElement | null
      const typing =
        target instanceof HTMLInputElement ||
        target instanceof HTMLTextAreaElement ||
        target?.isContentEditable

      if (pushDialogOpen) {
        if (event.key === 'Escape') {
          setPushDialogOpen(false)
        }
        return
      }

      if (manageDialog) {
        if (event.key === 'Escape') {
          setManageDialog(null)
        }
        return
      }

      // Native menu accelerators own the Command-key shortcuts.
      if (isNative && (event.metaKey || event.ctrlKey)) return

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

  menuActionRef.current = (command) => {
    switch (command) {
      case 'settings':
        openSettings()
        break
      case 'open-repository':
        openAddRepository()
        break
      case 'new-workspace':
        openCreateWorkspace()
        break
      case 'toggle-sidebar':
        setSidebarCollapsed((current) => !current)
        break
      case 'command-palette':
        openPalette()
        break
      case 'split-diff':
        setMode('split')
        break
      case 'unified-diff':
        setMode('unified')
        break
      case 'refresh':
        handleRefresh()
        break
      case 'previous-file':
        goToFile(-1)
        break
      case 'next-file':
        goToFile(1)
        break
      default:
        break
    }
  }

  return (
    <I18nProvider locale={locale}>
      <div className={`app-shell ${sidebarCollapsed ? 'is-sidebar-collapsed' : ''}`}>
        <Sidebar
          ref={sidebarRef}
          width={sidebarCollapsed ? 0 : sidebarWidth}
          collapsed={sidebarCollapsed}
          workspace={activeWorkspace}
          workspaces={workspaces}
          projects={workspaceProjects}
          groups={activeWorkspace.groups ?? []}
          activeProjectId={activeProject?.id ?? ''}
          expandedProjectId={expandedProjectId}
          selectedFile={activeFile?.path ?? null}
          scanningGroupId={scanningGroupId}
          onProjectClick={handleProjectClick}
          onSelectFile={selectFile}
          onSwitchWorkspace={queryForWorkspace}
          onCreateWorkspace={openCreateWorkspace}
          onRequestDeleteWorkspace={requestDeleteWorkspace}
          onAddRepository={openAddRepository}
          onQuickOpen={openPalette}
          onCollapse={() => setSidebarCollapsed(true)}
          onProjectContextMenu={handleProjectContextMenu}
          onFileContextMenu={handleFileContextMenu}
          onGroupContextMenu={handleGroupContextMenu}
          onToggleGroup={toggleGroup}
          onRescanGroup={rescanGroup}
        />
        {!sidebarCollapsed && (
          <SidebarResizer
            sidebarRef={sidebarRef}
            label={t('sidebar.resize')}
            onCommit={commitSidebarWidth}
            onReset={resetSidebarWidth}
          />
        )}

        <div className="app-main">
          <RepoBar
            project={activeProject}
            workspaceName={activeWorkspaceName}
            sidebarCollapsed={sidebarCollapsed}
            refreshing={refreshing}
            syncing={syncing}
            conflictCount={conflictCount}
            mergeInProgress={mergeInProgress}
            onOpenSidebar={() => setSidebarCollapsed(false)}
            onOpenSettings={openSettings}
            onQuickOpen={openPalette}
            onRefresh={handleRefresh}
            onPull={handlePull}
            onPush={handlePush}
            onSelectBranch={handleSelectBranch}
            onOpenRemoteSettings={() => setManageDialog({ kind: 'remote-settings' })}
            onResolveConflicts={() => setResolverOpen(true)}
            onAddRepository={openAddRepository}
            onRequestRemoveRepository={requestDeleteRepository}
            onRevealInFinder={() => {
              if (activeProject) {
                runPathAction('reveal_in_finder', activeProject.path)
              }
            }}
            onOpenInEditor={() => {
              if (activeProject) {
                runPathAction('open_in_editor', activeProject.path)
              }
            }}
            onCopyPath={() => {
              if (activeProject) copyPath(activeProject.path)
            }}
            onContextMenu={(event) => {
              if (activeProject) {
                handleProjectContextMenu(activeProject, event)
              }
            }}
          />

          <div className="workbench">
            <main className="main-pane">
              {showConflictBanner && (
                <div
                  className={`conflict-banner ${mergeReady ? 'is-ready' : ''}`}
                  role="alert"
                >
                  <span className="conflict-banner-icon">
                    {mergeReady ? (
                      <CheckCircle2 size={15} strokeWidth={2} />
                    ) : (
                      <TriangleAlert size={15} strokeWidth={2} />
                    )}
                  </span>
                  <span className="conflict-banner-copy">
                    <strong>
                      {mergeReady
                        ? t('conflict.bannerComplete')
                        : t('conflict.banner', { count: conflictCount })}
                    </strong>
                    <span>
                      {mergeReady
                        ? t('conflict.bannerReadyHint')
                        : t('conflict.bannerHint')}
                    </span>
                  </span>
                  <button
                    type="button"
                    className="conflict-banner-action"
                    onClick={
                      mergeReady
                        ? handleCompleteMerge
                        : () => setResolverOpen(true)
                    }
                  >
                    {mergeReady
                      ? t('conflict.completeShort')
                      : t('conflict.open')}
                  </button>
                </div>
              )}

              {showConflictResolver && activeProject ? (
                <ConflictResolver
                  project={activeProject}
                  conflicts={activeProject.conflicts ?? []}
                  mergeInProgress={Boolean(activeProject.mergeInProgress)}
                  mode={mode}
                  appTheme={resolvedTheme}
                  onModeChange={setMode}
                  onToast={showToast}
                  onResolved={handleConflictsResolved}
                  onCompleteMerge={handleCompleteMerge}
                  onAbortMerge={handleAbortMerge}
                  onClose={() => setResolverOpen(false)}
                />
              ) : activeFile && activeProject ? (
                <DiffViewer
                  key={`${activeProject.id}-${activeFile.path}`}
                  file={activeFile}
                  mode={mode}
                  baseBranch={activeProject.branch}
                  repositoryPath={
                    activeProject.source === 'local' ? activeProject.path : null
                  }
                  appTheme={resolvedTheme}
                  fileIndex={fileIndex}
                  fileCount={activeProject.files.length}
                  onModeChange={setMode}
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
                  onExploreSample={handleExploreSample}
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
            onConfirm={handleAddRepositories}
            onClose={() => setManageDialog(null)}
          />
        )}

        {manageDialog?.kind === 'remote-settings' &&
          activeProject &&
          activeProject.source === 'local' && (
            <RemoteDialog
              project={activeProject}
              onSaved={handleRemoteSaved}
              onClose={() => setManageDialog(null)}
            />
          )}

        {pushDialogOpen && activeProject && activeProject.source === 'local' && (
          <PushDialog
            projectName={activeProject.name}
            repositoryPath={activeProject.path}
            appTheme={resolvedTheme}
            onPushed={handlePushed}
            onClose={() => setPushDialogOpen(false)}
          />
        )}

        {manageDialog?.kind === 'abort-merge' && (
          <ConfirmRemoveDialog
            title={t('conflict.abortTitle')}
            description={t('conflict.abortDescription')}
            confirmLabel={t('conflict.abortConfirm')}
            onConfirm={confirmAbortMerge}
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

        {manageDialog?.kind === 'delete-group' && (
          <ConfirmRemoveDialog
            title={t('repo.groupRemoveTitle')}
            description={t('repo.groupRemoveDescription', {
              name: manageDialog.group.name,
            })}
            confirmLabel={t('repo.groupRemove')}
            onConfirm={handleRemoveGroup}
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

import { persistNativeStore } from './native'
import { projects } from '../data/projects'
import type { Translate } from './i18n'
import type { Project, ProjectGroup, Workspace } from '../types'

export interface WorkspaceState {
  workspaces: Workspace[]
  activeWorkspaceId: string
}

export interface NewRepositoryInput {
  name: string
  path: string
  branch: string
  branches?: string[]
}

export interface NewRepositoryGroup {
  name: string
  path: string
}

export interface RepositoryAddRequest {
  /** Set when the repositories were discovered inside one parent folder. */
  group?: NewRepositoryGroup
  repositories: NewRepositoryInput[]
}

const STORAGE_KEY = 'flashbee:workspaces:v1'
const STORAGE_VERSION = 2
// Version 1 stored flat repository lists; version 2 adds folder groups.
const SUPPORTED_STORAGE_VERSIONS = new Set([1, 2])

const projectAccents = [
  { accent: '#f6bd60', accentSoft: 'rgba(246, 189, 96, 0.12)' },
  { accent: '#64d2e8', accentSoft: 'rgba(100, 210, 232, 0.11)' },
  { accent: '#b49cff', accentSoft: 'rgba(180, 156, 255, 0.11)' },
  { accent: '#83d4a4', accentSoft: 'rgba(131, 212, 164, 0.11)' },
  { accent: '#ff9f76', accentSoft: 'rgba(255, 159, 118, 0.11)' },
  { accent: '#8fc7ff', accentSoft: 'rgba(143, 199, 255, 0.11)' },
]

function createId(prefix: string) {
  const random =
    typeof crypto !== 'undefined' && 'randomUUID' in crypto
      ? crypto.randomUUID().slice(0, 8)
      : Math.random().toString(36).slice(2, 10)
  return `${prefix}-${Date.now().toString(36)}-${random}`
}

export function seedWorkspaceState(): WorkspaceState {
  return {
    workspaces: [
      {
        id: 'personal',
        name: 'Personal workspace',
        projects: [],
        groups: [],
      },
    ],
    activeWorkspaceId: 'personal',
  }
}

export function createSampleWorkspace(): Workspace {
  return {
    id: 'sample',
    name: 'Sample workspace',
    projects: projects.map((project) => ({ ...project, source: 'demo' })),
    groups: [],
  }
}

function isValidProjectGroup(value: unknown): value is ProjectGroup {
  if (!value || typeof value !== 'object') return false
  const group = value as Partial<ProjectGroup>
  return (
    typeof group.id === 'string' &&
    typeof group.name === 'string' &&
    typeof group.path === 'string' &&
    (group.collapsed === undefined || typeof group.collapsed === 'boolean')
  )
}

function isValidWorkspaceState(value: unknown): value is WorkspaceState {
  if (!value || typeof value !== 'object') return false
  const state = value as Partial<WorkspaceState>
  return (
    Array.isArray(state.workspaces) &&
    state.workspaces.length > 0 &&
    state.workspaces.every(
      (workspace) =>
        workspace &&
        typeof workspace.id === 'string' &&
        typeof workspace.name === 'string' &&
        Array.isArray(workspace.projects) &&
        (workspace.groups === undefined ||
          (Array.isArray(workspace.groups) &&
            workspace.groups.every(isValidProjectGroup))) &&
        workspace.projects.every(
          (project) =>
            project &&
            typeof project.id === 'string' &&
            Array.isArray(project.files) &&
            (project.groupId === undefined ||
              typeof project.groupId === 'string'),
        ),
    ) &&
    typeof state.activeWorkspaceId === 'string'
  )
}

/** Fills in groups for version-1 state and drops dangling group references. */
function normalizeWorkspace(workspace: Workspace): Workspace {
  const groups = Array.isArray(workspace.groups)
    ? workspace.groups.filter(isValidProjectGroup)
    : []
  const groupIds = new Set(groups.map((group) => group.id))
  return {
    ...workspace,
    groups,
    projects: workspace.projects.map((project) =>
      project.groupId && !groupIds.has(project.groupId)
        ? { ...project, groupId: undefined }
        : project,
    ),
  }
}

export function loadWorkspaceState(): WorkspaceState {
  try {
    const raw = window.localStorage.getItem(STORAGE_KEY)
    if (!raw) return seedWorkspaceState()

    const parsed = JSON.parse(raw) as {
      version?: number
      workspaces?: unknown
      activeWorkspaceId?: unknown
    }
    if (
      typeof parsed.version !== 'number' ||
      !SUPPORTED_STORAGE_VERSIONS.has(parsed.version) ||
      !isValidWorkspaceState({
        workspaces: parsed.workspaces,
        activeWorkspaceId: parsed.activeWorkspaceId,
      })
    ) {
      return seedWorkspaceState()
    }

    const state = {
      workspaces: (parsed.workspaces as Workspace[]).map(normalizeWorkspace),
      activeWorkspaceId: parsed.activeWorkspaceId as string,
    }
    if (!state.workspaces.some((workspace) => workspace.id === state.activeWorkspaceId)) {
      state.activeWorkspaceId = state.workspaces[0].id
    }
    return state
  } catch {
    return seedWorkspaceState()
  }
}

export function saveWorkspaceState(state: WorkspaceState) {
  // Local repositories are re-read from disk on launch; persisting their diffs
  // and transient merge state would bloat the store and could resurrect stale
  // conflicts after the user resolved them outside the app.
  const persistable: WorkspaceState = {
    ...state,
    workspaces: state.workspaces.map((workspace) => ({
      ...workspace,
      // Worktree cards are derived from the repository snapshot on every
      // launch; persisting them would resurrect checkouts that no longer exist.
      projects: workspace.projects
        .filter((project) => !project.worktreeOf)
        .map((project) =>
          project.source === 'local'
            ? { ...project, files: [], conflicts: [], mergeInProgress: false }
            : project,
        ),
    })),
  }
  const contents = JSON.stringify({ version: STORAGE_VERSION, ...persistable })
  try {
    window.localStorage.setItem(STORAGE_KEY, contents)
    persistNativeStore('workspaces', contents)
  } catch {
    // Changes stay available for the current session.
  }
}

export function workspaceLabel(workspace: Workspace, t: Translate) {
  if (workspace.id === 'personal') return t('sidebar.personalWorkspace')
  if (workspace.id === 'sample') return t('workspace.sampleName')
  return workspace.name
}

export function workspaceInitial(workspace: Workspace, t: Translate) {
  if (workspace.id === 'personal') return 'W'
  if (workspace.id === 'sample') return 'S'
  return workspaceLabel(workspace, t).trim().charAt(0).toUpperCase() || 'W'
}

export function createWorkspace(name: string): Workspace {
  return {
    id: createId('workspace'),
    name: name.trim(),
    projects: [],
    groups: [],
  }
}

export function createProjectGroup(input: NewRepositoryGroup): ProjectGroup {
  return {
    id: createId('group'),
    name: input.name.trim(),
    path: input.path.trim(),
    collapsed: false,
  }
}

export function createRepository(
  input: NewRepositoryInput,
  existingProjects: Project[],
  groupId?: string,
): Project {
  const accent = projectAccents[existingProjects.length % projectAccents.length]
  const branch = input.branch.trim() || 'main'

  return {
    id: createId('repo'),
    name: input.name.trim(),
    path: input.path.trim(),
    branch,
    ahead: 0,
    behind: 0,
    accent: accent.accent,
    accentSoft: accent.accentSoft,
    updated: 'now',
    files: [],
    branches: input.branches ?? [branch],
    source: 'local',
    ...(groupId ? { groupId } : {}),
  }
}

export function repositoryNameFromPath(path: string) {
  const normalized = path.trim().replace(/[\\/]+$/, '')
  const parts = normalized.split(/[\\/]/)
  return parts[parts.length - 1] || normalized
}

export function initialSelectionForWorkspaces(workspaces: Workspace[]) {
  return Object.fromEntries(
    workspaces.flatMap((workspace) =>
      workspace.projects.map((project) => [
        project.id,
        project.files[0]?.path ?? '',
      ]),
    ),
  )
}

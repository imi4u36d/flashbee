import { projects } from '../data/projects'
import type { Translate } from './i18n'
import type { Project, Workspace } from '../types'

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

const STORAGE_KEY = 'flashbee:workspaces:v1'
const STORAGE_VERSION = 1

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
        projects,
      },
    ],
    activeWorkspaceId: 'personal',
  }
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
        workspace.projects.every(
          (project) =>
            project &&
            typeof project.id === 'string' &&
            Array.isArray(project.files),
        ),
    ) &&
    typeof state.activeWorkspaceId === 'string'
  )
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
      parsed.version !== STORAGE_VERSION ||
      !isValidWorkspaceState({
        workspaces: parsed.workspaces,
        activeWorkspaceId: parsed.activeWorkspaceId,
      })
    ) {
      return seedWorkspaceState()
    }

    const state = {
      workspaces: parsed.workspaces as Workspace[],
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
  try {
    window.localStorage.setItem(
      STORAGE_KEY,
      JSON.stringify({ version: STORAGE_VERSION, ...state }),
    )
  } catch {
    // Changes stay available for the current session.
  }
}

export function workspaceLabel(workspace: Workspace, t: Translate) {
  return workspace.id === 'personal'
    ? t('sidebar.personalWorkspace')
    : workspace.name
}

export function workspaceInitial(workspace: Workspace, t: Translate) {
  if (workspace.id === 'personal') return 'W'
  return workspaceLabel(workspace, t).trim().charAt(0).toUpperCase() || 'W'
}

export function createWorkspace(name: string): Workspace {
  return {
    id: createId('workspace'),
    name: name.trim(),
    projects: [],
  }
}

export function createRepository(
  input: NewRepositoryInput,
  existingProjects: Project[],
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

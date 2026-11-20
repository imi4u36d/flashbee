import { open as openDialog } from '@tauri-apps/plugin-dialog'
import { invokeNative, isNative } from './native'
import type {
  BlameLine,
  ChangeStatus,
  ConflictEntry,
  ConflictSegment,
  ConflictSide,
  DiffFile,
  Hunk,
  RemoteBranch,
  RemoteInfo,
} from '../types'

export interface RepositoryInspection {
  path: string
  name: string
  currentBranch: string
  branches: string[]
  remoteBranches: RemoteBranch[]
  remotes: RemoteInfo[]
  upstream: string | null
  ahead: number
  behind: number
}

export interface RepositoryDiscovery {
  path: string
  name: string
  isRepository: boolean
  repositories: RepositoryInspection[]
  truncated: boolean
}

interface RemoteFile {
  path: string
  status: ChangeStatus
  language: string
  staged: boolean
  hunks: Hunk[]
}

export interface RepositorySnapshot extends RepositoryInspection {
  files: RemoteFile[]
  conflicts: ConflictEntry[]
  mergeInProgress: boolean
  fetchedAt: string
}

export interface SyncOutcome {
  message: string
  conflicts: string[]
}

export interface ConflictDetail {
  path: string
  language: string
  binary: boolean
  oursExists: boolean
  theirsExists: boolean
  baseExists: boolean
  trailingNewline: boolean
  segments: ConflictSegment[]
}

export class RepositoryApiError extends Error {
  code: string

  constructor(code: string, message: string) {
    super(message)
    this.name = 'RepositoryApiError'
    this.code = code
  }
}

function normalizeError(reason: unknown): RepositoryApiError {
  if (reason instanceof RepositoryApiError) return reason

  if (reason && typeof reason === 'object') {
    const payload = reason as { code?: unknown; message?: unknown }
    if (typeof payload.code === 'string') {
      return new RepositoryApiError(
        payload.code,
        typeof payload.message === 'string' ? payload.message : 'The request failed.',
      )
    }
  }

  return new RepositoryApiError(
    'native_unavailable',
    typeof reason === 'string' ? reason : 'The native Git service is unavailable.',
  )
}

function nativeRequest<T>(command: string, args: Record<string, unknown>) {
  if (!isNative) {
    return Promise.reject(
      new RepositoryApiError(
        'native_unavailable',
        'Repository access is only available in the FlashBee desktop app.',
      ),
    )
  }

  return invokeNative<T>(command, args).catch((reason) => {
    throw normalizeError(reason)
  })
}

export function inspectRepositoryPath(repositoryPath: string) {
  return nativeRequest<RepositoryInspection>('inspect_repository', {
    path: repositoryPath,
  })
}

export function discoverRepositoryTarget(repositoryPath: string) {
  return nativeRequest<RepositoryDiscovery>('discover_repositories', {
    path: repositoryPath,
  })
}

export function initializeRepository(repositoryPath: string) {
  return nativeRequest<RepositoryInspection>('initialize_repository', {
    path: repositoryPath,
  })
}

export async function pickRepositoryFolder() {
  if (!isNative) {
    throw new RepositoryApiError(
      'picker_unsupported',
      'The native folder picker is only available in the desktop app.',
    )
  }

  let selection: string | string[] | null
  try {
    selection = await openDialog({
      directory: true,
      multiple: false,
      title: 'Choose a folder',
    })
  } catch (reason) {
    throw normalizeError(reason)
  }

  if (!selection || Array.isArray(selection)) {
    throw new RepositoryApiError('cancelled', 'Folder selection was cancelled.')
  }

  return selection
}

export function fetchRepositorySnapshot(
  repositoryPath: string,
  base?: string,
): Promise<RepositorySnapshot> {
  return nativeRequest<RepositorySnapshot>('repository_snapshot', {
    path: repositoryPath,
    base: base ?? null,
  })
}

export function fetchBlame(
  repositoryPath: string,
  filePath: string,
  lines: number[],
  revision?: string,
): Promise<BlameLine[]> {
  return nativeRequest<BlameLine[]>('blame_file', {
    path: repositoryPath,
    filePath,
    revision: revision ?? null,
    lines,
  })
}

export function fetchRemotes(repositoryPath: string): Promise<RemoteInfo[]> {
  return nativeRequest<RemoteInfo[]>('list_remotes', { path: repositoryPath })
}

export function setRemoteUrl(
  repositoryPath: string,
  name: string,
  url: string,
): Promise<RemoteInfo[]> {
  return nativeRequest<RemoteInfo[]>('set_remote_url', {
    path: repositoryPath,
    name,
    url,
  })
}

export function pullRepository(repositoryPath: string): Promise<SyncOutcome> {
  return nativeRequest<SyncOutcome>('pull_repository', {
    path: repositoryPath,
  })
}

export function pushRepository(repositoryPath: string): Promise<SyncOutcome> {
  return nativeRequest<SyncOutcome>('push_repository', {
    path: repositoryPath,
  })
}

export function fetchConflict(
  repositoryPath: string,
  filePath: string,
): Promise<ConflictDetail> {
  return nativeRequest<ConflictDetail>('read_conflict', {
    path: repositoryPath,
    filePath,
  })
}

export function saveConflictResolution(
  repositoryPath: string,
  filePath: string,
  content: string,
): Promise<void> {
  return nativeRequest<void>('save_conflict_resolution', {
    path: repositoryPath,
    filePath,
    content,
  })
}

export function resolveConflictSide(
  repositoryPath: string,
  filePath: string,
  side: ConflictSide,
): Promise<void> {
  return nativeRequest<void>('resolve_conflict_side', {
    path: repositoryPath,
    filePath,
    side,
  })
}

export function completeMerge(repositoryPath: string): Promise<SyncOutcome> {
  return nativeRequest<SyncOutcome>('complete_merge', {
    path: repositoryPath,
  })
}

export function abortMerge(repositoryPath: string): Promise<void> {
  return nativeRequest<void>('abort_merge', {
    path: repositoryPath,
  })
}

export function snapshotFiles(snapshot: RepositorySnapshot): DiffFile[] {
  return snapshot.files.map((file) => ({
    path: file.path,
    status: file.status,
    language: file.language,
    staged: file.staged,
    hunks: file.hunks,
  }))
}

import type { ChangeStatus, DiffFile, Hunk } from '../types'

export interface RepositoryInspection {
  path: string
  name: string
  currentBranch: string
  branches: string[]
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
  fetchedAt: string
}

interface ApiErrorPayload {
  code?: string
  message?: string
}

export class RepositoryApiError extends Error {
  code: string

  constructor(code: string, message: string) {
    super(message)
    this.name = 'RepositoryApiError'
    this.code = code
  }
}

async function requestJson<T>(url: string, init?: RequestInit): Promise<T> {
  let response: Response
  try {
    response = await fetch(url, init)
  } catch {
    throw new RepositoryApiError(
      'api_unavailable',
      'The local repository service is unavailable.',
    )
  }

  let payload: unknown = null
  try {
    payload = await response.json()
  } catch {
    // Fall through to a generic error.
  }

  if (!response.ok) {
    const error = (payload ?? {}) as ApiErrorPayload
    throw new RepositoryApiError(
      error.code ?? 'request_failed',
      error.message ?? 'The repository request failed.',
    )
  }

  return payload as T
}

export function inspectRepositoryPath(repositoryPath: string) {
  return requestJson<RepositoryInspection>(
    `/api/repository/inspect?path=${encodeURIComponent(repositoryPath)}`,
  )
}

export function pickRepositoryDirectory() {
  return requestJson<RepositoryInspection>('/api/repository/pick', {
    method: 'POST',
  })
}

export async function fetchRepositorySnapshot(
  repositoryPath: string,
): Promise<RepositorySnapshot> {
  return requestJson<RepositorySnapshot>(
    `/api/repository/snapshot?path=${encodeURIComponent(repositoryPath)}`,
  )
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

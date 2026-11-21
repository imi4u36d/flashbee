export type ChangeStatus = 'modified' | 'added' | 'deleted' | 'renamed'
export type DiffMode = 'split' | 'unified'
export type LineType = 'context' | 'add' | 'del'

export interface DiffLine {
  type: LineType
  content: string
  oldNo?: number
  newNo?: number
}

export interface Hunk {
  header: string
  lines: DiffLine[]
}

export interface DiffFile {
  path: string
  status: ChangeStatus
  language: string
  staged: boolean
  hunks: Hunk[]
  viewed?: boolean
}

export interface BlameLine {
  line: number
  author: string
  authorMail: string
  time: number
  commit: string
  summary: string
  uncommitted: boolean
}

export interface RemoteBranch {
  name: string
  remote: string
  branch: string
}

export interface RemoteInfo {
  name: string
  url: string
}

/** One checkout attached to a repository through `git worktree`. */
export interface WorktreeInfo {
  path: string
  name: string
  /** Checked-out branch, or `HEAD` when detached. */
  branch: string
  head: string
  detached: boolean
  isMain: boolean
  isCurrent: boolean
}

export type ConflictSide = 'ours' | 'theirs'

export interface ConflictEntry {
  path: string
  kind: string
  base: boolean
  ours: boolean
  theirs: boolean
}

export type ConflictSegment =
  | { kind: 'common'; lines: string[] }
  | {
      kind: 'conflict'
      ours: string[]
      base: string[]
      theirs: string[]
      suggestion: ConflictSide | null
    }

export interface Project {
  id: string
  name: string
  path: string
  branch: string
  currentBranch?: string
  ahead: number
  behind: number
  accent: string
  accentSoft: string
  updated: string
  files: DiffFile[]
  branches?: string[]
  remoteBranches?: RemoteBranch[]
  remotes?: RemoteInfo[]
  upstream?: string | null
  conflicts?: ConflictEntry[]
  mergeInProgress?: boolean
  source?: 'demo' | 'local'
  /** Folder group this repository was discovered in, if any. */
  groupId?: string
  /** Primary working tree this linked worktree was discovered from. */
  worktreeOf?: string
  /** Short HEAD hash, shown when the linked worktree is detached. */
  worktreeHead?: string
  /** True once this derived worktree has read its own snapshot. */
  hydrated?: boolean
}

/** A parent folder the user added to keep its sub-repositories together. */
export interface ProjectGroup {
  id: string
  name: string
  path: string
  collapsed?: boolean
}

export interface Workspace {
  id: string
  name: string
  projects: Project[]
  groups?: ProjectGroup[]
}

export interface SplitRow {
  left?: DiffLine
  right?: DiffLine
}

export type DiffRow =
  | { kind: 'hunk'; hunk: Hunk }
  | { kind: 'line'; line: DiffLine }

export type SplitDiffRow =
  | { kind: 'hunk'; hunk: Hunk; hunkIndex: number }
  | { kind: 'split'; row: SplitRow }

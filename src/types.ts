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

export interface Project {
  id: string
  name: string
  path: string
  branch: string
  ahead: number
  behind: number
  accent: string
  accentSoft: string
  updated: string
  files: DiffFile[]
  branches?: string[]
  source?: 'demo' | 'local'
}

export interface Workspace {
  id: string
  name: string
  projects: Project[]
}

export interface SplitRow {
  left?: DiffLine
  right?: DiffLine
}

export type DiffRow =
  | { kind: 'hunk'; hunk: Hunk }
  | { kind: 'line'; line: DiffLine }

export type SplitDiffRow =
  | { kind: 'hunk'; hunk: Hunk }
  | { kind: 'split'; row: SplitRow }

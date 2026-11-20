import { execFile } from 'node:child_process'
import { readFile, stat } from 'node:fs/promises'
import { homedir } from 'node:os'
import path from 'node:path'
import { promisify } from 'node:util'
import type { IncomingMessage, ServerResponse } from 'node:http'
import type { Plugin } from 'vite'

const execFileAsync = promisify(execFile)

interface RepositoryInspection {
  path: string
  name: string
  currentBranch: string
  branches: string[]
}

type ChangeStatus = 'modified' | 'added' | 'deleted' | 'renamed'
type LineType = 'context' | 'add' | 'del'

interface DiffLine {
  type: LineType
  content: string
  oldNo?: number
  newNo?: number
}

interface DiffHunk {
  header: string
  lines: DiffLine[]
}

interface RepositoryFile {
  path: string
  status: ChangeStatus
  language: string
  staged: boolean
  hunks: DiffHunk[]
}

interface RepositorySnapshot extends RepositoryInspection {
  files: RepositoryFile[]
  fetchedAt: string
}

interface StatusEntry {
  path: string
  oldPath?: string
  status: ChangeStatus
  staged: boolean
  untracked: boolean
}

const languageByExtension: Record<string, string> = {
  ts: 'typescript',
  tsx: 'tsx',
  js: 'javascript',
  mjs: 'javascript',
  cjs: 'javascript',
  jsx: 'jsx',
  css: 'css',
  scss: 'scss',
  less: 'less',
  json: 'json',
  md: 'markdown',
  mdx: 'markdown',
  py: 'python',
  go: 'go',
  rs: 'rust',
  toml: 'toml',
  wgsl: 'wgsl',
  html: 'html',
  htm: 'html',
  yml: 'yaml',
  yaml: 'yaml',
  sh: 'bash',
  bash: 'bash',
  zsh: 'bash',
  sql: 'sql',
  java: 'java',
  kt: 'kotlin',
  swift: 'swift',
  c: 'c',
  h: 'c',
  cpp: 'cpp',
  cc: 'cpp',
  hpp: 'cpp',
  cs: 'csharp',
  rb: 'ruby',
  php: 'php',
  xml: 'xml',
  vue: 'vue',
  svelte: 'svelte',
}

function languageForPath(filePath: string) {
  const name = path.basename(filePath)
  const extension = name.includes('.') ? name.split('.').pop()?.toLowerCase() : ''
  return (extension && languageByExtension[extension]) || 'text'
}

function statusFromCode(code: string): ChangeStatus {
  const normalized = code.replaceAll('?', 'A')
  if (normalized.includes('D')) return 'deleted'
  if (normalized.includes('R') || normalized.includes('C')) return 'renamed'
  if (normalized.includes('A')) return 'added'
  return 'modified'
}

async function readStatusEntries(root: string): Promise<StatusEntry[]> {
  const output = await runGit(
    ['status', '--porcelain=v1', '-z', '--untracked-files=all'],
    root,
  )
  if (!output) return []

  const fields = output.split('\0')
  const entries: StatusEntry[] = []
  for (let index = 0; index < fields.length; index += 1) {
    const value = fields[index]
    if (!value) continue
    const code = value.slice(0, 2)
    const firstPath = value.slice(3)
    let oldPath: string | undefined
    let filePath = firstPath

    if (code.includes('R') || code.includes('C')) {
      const secondPath = fields[index + 1]
      if (secondPath) {
        index += 1
        // With porcelain v1 -z, the second field is the destination path.
        oldPath = firstPath
        filePath = secondPath
      }
    }

    const untracked = code === '??'
    const staged = !untracked && code[0] !== ' ' && code[0] !== '?' && code[1] === ' '
    entries.push({
      path: filePath,
      oldPath,
      status: statusFromCode(code),
      staged,
      untracked,
    })
  }
  return entries
}

function stripDiffPrefix(value: string) {
  const trimmed = value.trim()
  if (trimmed === '/dev/null') return undefined
  return trimmed.replace(/^[ab]\//, '')
}

function parseHunksFromDiff(lines: string[]): DiffHunk[] {
  const hunks: DiffHunk[] = []
  let current: DiffHunk | null = null
  let oldNo = 0
  let newNo = 0

  for (const line of lines) {
    if (line.startsWith('@@ ')) {
      const match = line.match(/^@@ -(\d+)(?:,\d+)? \+(\d+)(?:,\d+)? @@/)
      if (!match) continue
      oldNo = Number(match[1])
      newNo = Number(match[2])
      current = { header: line, lines: [] }
      hunks.push(current)
      continue
    }

    if (!current) continue
    if (line.startsWith('\\')) continue

    if (line.startsWith('+')) {
      current.lines.push({ type: 'add', content: line.slice(1), newNo })
      newNo += 1
      continue
    }

    if (line.startsWith('-')) {
      current.lines.push({ type: 'del', content: line.slice(1), oldNo })
      oldNo += 1
      continue
    }

    if (line.startsWith(' ') || line === '') {
      current.lines.push({
        type: 'context',
        content: line.slice(1),
        oldNo,
        newNo,
      })
      oldNo += 1
      newNo += 1
    }
  }

  return hunks
}

function parseDiffSections(diff: string) {
  const sections: Array<{
    path: string
    oldPath?: string
    hunks: DiffHunk[]
  }> = []

  const lines = diff.split('\n')
  let currentLines: string[] = []
  let currentOldPath: string | undefined
  let currentNewPath: string | undefined

  const flush = () => {
    if (!currentNewPath && !currentOldPath) {
      currentLines = []
      currentOldPath = undefined
      currentNewPath = undefined
      return
    }
    const filePath = currentNewPath || currentOldPath || ''
    if (!filePath) {
      currentLines = []
      currentOldPath = undefined
      currentNewPath = undefined
      return
    }
    const hunks = parseHunksFromDiff(currentLines)
    if (hunks.length > 0) {
      sections.push({
        path: filePath,
        oldPath:
          currentOldPath && currentOldPath !== filePath
            ? currentOldPath
            : undefined,
        hunks,
      })
    }
    currentLines = []
    currentOldPath = undefined
    currentNewPath = undefined
  }

  for (const line of lines) {
    if (line.startsWith('diff --git ')) {
      flush()
      continue
    }
    if (line.startsWith('--- ')) {
      currentOldPath = stripDiffPrefix(line.slice(4))
      continue
    }
    if (line.startsWith('+++ ')) {
      currentNewPath = stripDiffPrefix(line.slice(4))
      continue
    }
    currentLines.push(line)
  }
  flush()
  return sections
}

async function readUntrackedFile(
  root: string,
  filePath: string,
): Promise<RepositoryFile | null> {
  const absolutePath = path.join(root, filePath)
  let info
  try {
    info = await stat(absolutePath)
  } catch {
    return null
  }
  if (!info.isFile() || info.size > 512 * 1024) return null

  let content: string
  try {
    content = await readFile(absolutePath, 'utf8')
  } catch {
    return null
  }
  if (content.includes('\u0000')) return null

  const rawLines = content.split('\n')
  if (rawLines[rawLines.length - 1] === '') rawLines.pop()
  const limited = rawLines.slice(0, 2000)
  const lines: DiffLine[] = limited.map((line, index) => ({
    type: 'add',
    content: line,
    newNo: index + 1,
  }))

  if (lines.length === 0) {
    return {
      path: filePath,
      status: 'added',
      language: languageForPath(filePath),
      staged: false,
      hunks: [
        {
          header: '@@ -0,0 +0,0 @@',
          lines: [],
        },
      ],
    }
  }

  return {
    path: filePath,
    status: 'added',
    language: languageForPath(filePath),
    staged: false,
    hunks: [
      {
        header: `@@ -0,0 +1,${lines.length} @@`,
        lines,
      },
    ],
  }
}

async function readRepositorySnapshot(
  inputPath: string,
): Promise<RepositorySnapshot> {
  const inspection = await inspectRepository(inputPath)
  const statusEntries = await readStatusEntries(inspection.path)
  const statusByPath = new Map(statusEntries.map((entry) => [entry.path, entry]))
  const filesByPath = new Map<string, RepositoryFile>()

  let diff = ''
  try {
    diff = await runGit(
      [
        '-c',
        'core.quotePath=false',
        'diff',
        'HEAD',
        '--no-color',
        '--no-ext-diff',
        '--unified=3',
        '--find-renames',
      ],
      inspection.path,
    )
  } catch {
    diff = ''
  }

  for (const section of parseDiffSections(diff)) {
    const statusEntry =
      statusByPath.get(section.path) ??
      (section.oldPath ? statusByPath.get(section.oldPath) : undefined)
    filesByPath.set(section.path, {
      path: section.path,
      status: statusEntry?.status ?? 'modified',
      language: languageForPath(section.path),
      staged: statusEntry?.staged ?? false,
      hunks: section.hunks,
    })
  }

  for (const entry of statusEntries) {
    if (!entry.untracked) continue
    if (filesByPath.has(entry.path)) continue
    const file = await readUntrackedFile(inspection.path, entry.path)
    if (file) filesByPath.set(file.path, file)
  }

  return {
    ...inspection,
    files: Array.from(filesByPath.values())
      .sort((a, b) => a.path.localeCompare(b.path))
      .slice(0, 120),
    fetchedAt: new Date().toISOString(),
  }
}

class ApiError extends Error {
  status: number
  code: string

  constructor(status: number, code: string, message: string) {
    super(message)
    this.status = status
    this.code = code
  }
}

function sendJson(res: ServerResponse, status: number, body: unknown) {
  res.statusCode = status
  res.setHeader('Content-Type', 'application/json; charset=utf-8')
  res.setHeader('Cache-Control', 'no-store')
  res.end(JSON.stringify(body))
}

function expandHome(input: string) {
  if (input === '~') return homedir()
  if (input.startsWith('~/')) return path.join(homedir(), input.slice(2))
  return input
}

async function runGit(args: string[], cwd: string) {
  try {
    const { stdout } = await execFileAsync('git', args, {
      cwd,
      maxBuffer: 1024 * 1024 * 4,
      timeout: 15_000,
    })
    return stdout.trim()
  } catch (error) {
    const detail = error instanceof Error ? error.message : String(error)
    if (/ENOENT/.test(detail)) {
      throw new ApiError(500, 'git_unavailable', 'Git is not installed or not available.')
    }
    throw new ApiError(422, 'not_git', 'This folder is not a Git repository.')
  }
}

async function inspectRepository(inputPath: string): Promise<RepositoryInspection> {
  const requested = expandHome(inputPath.trim())
  if (!requested) {
    throw new ApiError(400, 'bad_request', 'A repository path is required.')
  }

  let info
  try {
    info = await stat(requested)
  } catch {
    throw new ApiError(404, 'not_found', 'This folder does not exist.')
  }

  if (!info.isDirectory()) {
    throw new ApiError(400, 'not_directory', 'Select a folder, not a file.')
  }

  const root = await runGit(['rev-parse', '--show-toplevel'], requested)
  if (!root) {
    throw new ApiError(422, 'not_git', 'This folder is not inside a Git repository.')
  }

  const [currentBranchRaw, branchesRaw] = await Promise.all([
    runGit(['branch', '--show-current'], root).catch(() => ''),
    runGit(['for-each-ref', '--format=%(refname:short)', 'refs/heads'], root),
  ])

  const localBranches = branchesRaw
    .split('\n')
    .map((branch) => branch.trim())
    .filter(Boolean)

  const currentBranch = currentBranchRaw || localBranches[0] || 'HEAD'
  const remaining = localBranches
    .filter((branch) => branch !== currentBranch)
    .sort((a, b) => a.localeCompare(b))
  const branches = [currentBranch, ...remaining]

  return {
    path: root,
    name: path.basename(root),
    currentBranch,
    branches,
  }
}

async function pickFolder(): Promise<string> {
  if (process.platform !== 'darwin') {
    throw new ApiError(
      501,
      'picker_unsupported',
      'The native folder picker is available on macOS. Enter a path instead.',
    )
  }

  try {
    const { stdout } = await execFileAsync(
      'osascript',
      [
        '-e',
        'POSIX path of (choose folder with prompt "Choose a Git repository")',
      ],
      { timeout: 10 * 60 * 1000 },
    )
    return stdout.trim()
  } catch (error) {
    const detail = error instanceof Error ? error.message : String(error)
    if (/User canceled|\(-128\)|cancelled|canceled/i.test(detail)) {
      throw new ApiError(400, 'cancelled', 'Folder selection was cancelled.')
    }
    if (/ENOENT/.test(detail)) {
      throw new ApiError(501, 'picker_unsupported', 'The native folder picker is unavailable.')
    }
    throw new ApiError(500, 'picker_failed', 'Could not open the folder picker.')
  }
}

async function handleApi(
  req: IncomingMessage,
  res: ServerResponse,
  next: (error?: unknown) => void,
) {
  const url = new URL(req.url ?? '/', 'http://localhost')
  if (!url.pathname.startsWith('/api/repository/')) {
    next()
    return
  }

  try {
    if (url.pathname === '/api/repository/inspect') {
      const repositoryPath = url.searchParams.get('path')
      if (!repositoryPath) {
        throw new ApiError(400, 'bad_request', 'A repository path is required.')
      }
      sendJson(res, 200, await inspectRepository(repositoryPath))
      return
    }

    if (url.pathname === '/api/repository/pick') {
      const pickedPath = await pickFolder()
      sendJson(res, 200, await inspectRepository(pickedPath))
      return
    }

    if (url.pathname === '/api/repository/snapshot') {
      const repositoryPath = url.searchParams.get('path')
      if (!repositoryPath) {
        throw new ApiError(400, 'bad_request', 'A repository path is required.')
      }
      sendJson(res, 200, await readRepositorySnapshot(repositoryPath))
      return
    }

    sendJson(res, 404, { code: 'not_found', message: 'Unknown repository API route.' })
  } catch (error) {
    if (error instanceof ApiError) {
      sendJson(res, error.status, { code: error.code, message: error.message })
      return
    }
    sendJson(res, 500, {
      code: 'unknown_error',
      message: error instanceof Error ? error.message : 'Unexpected error.',
    })
  }
}

export function repositoryApiPlugin(): Plugin {
  return {
    name: 'flashbee-repository-api',
    configureServer(server) {
      server.middlewares.use((req, res, next) => {
        void handleApi(req, res, next)
      })
    },
    configurePreviewServer(server) {
      server.middlewares.use((req, res, next) => {
        void handleApi(req, res, next)
      })
    },
  }
}

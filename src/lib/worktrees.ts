import type { Project, WorktreeInfo } from '../types'

/** Deterministic id so a worktree card keeps its selection across refreshes. */
export function worktreeProjectId(primaryId: string, path: string) {
  let hash = 5381
  for (let index = 0; index < path.length; index += 1) {
    hash = (hash * 33 + path.charCodeAt(index)) | 0
  }
  return `${primaryId}-wt-${(hash >>> 0).toString(36)}`
}

/**
 * Linked checkouts only nest below a repository card when that card is the
 * repository's main worktree. A linked worktree added on its own stays a plain
 * repository instead of sprouting the main checkout as a child.
 */
export function nestedWorktrees(worktrees: WorktreeInfo[] | undefined) {
  const list = worktrees ?? []
  if (!list.some((worktree) => worktree.isMain && worktree.isCurrent)) {
    return []
  }
  return list.filter((worktree) => !worktree.isCurrent)
}

export function worktreeBranch(worktree: WorktreeInfo) {
  return worktree.detached ? 'HEAD' : worktree.branch
}

/**
 * A checkout such as `.worktrees/c58f/flashBee` shares its folder name with the
 * repository; the parent folder keeps the two cards distinguishable.
 */
export function worktreeDisplayName(primary: Project, worktree: WorktreeInfo) {
  if (worktree.name !== primary.name) return worktree.name
  const parts = worktree.path.replace(/[\\/]+$/, '').split(/[\\/]/)
  return parts.length > 1 ? parts.slice(-2).join('/') : worktree.name
}

export function createWorktreeProject(
  primary: Project,
  worktree: WorktreeInfo,
): Project {
  const branch = worktreeBranch(worktree)
  return {
    id: worktreeProjectId(primary.id, worktree.path),
    name: worktreeDisplayName(primary, worktree),
    path: worktree.path,
    branch,
    currentBranch: branch,
    ahead: 0,
    behind: 0,
    accent: primary.accent,
    accentSoft: primary.accentSoft,
    updated: 'now',
    files: [],
    branches: [branch],
    source: 'local',
    worktreeOf: primary.path,
    worktreeHead: worktree.head,
    hydrated: false,
    ...(primary.groupId ? { groupId: primary.groupId } : {}),
  }
}

/**
 * Reconciles the cards nested under `primary` with the repository's current
 * worktree list: new checkouts are added, renamed branches are followed, and
 * worktrees that were removed stop showing up.
 */
export function mergeWorktreeProjects(
  projects: Project[],
  primary: Project,
  worktrees: WorktreeInfo[] | undefined,
): Project[] {
  const linked = nestedWorktrees(worktrees)
  const linkedByPath = new Map(
    linked.map((worktree) => [worktree.path, worktree]),
  )
  const knownPaths = new Set(projects.map((project) => project.path))
  const next: Project[] = []

  for (const project of projects) {
    if (
      project.worktreeOf === primary.path &&
      !linkedByPath.has(project.path)
    ) {
      continue
    }

    if (project.path === primary.path) {
      next.push(project)
      continue
    }

    const worktree =
      project.worktreeOf === primary.path
        ? linkedByPath.get(project.path)
        : undefined
    if (!worktree) {
      next.push(project)
      continue
    }

    // Keep an explicit comparison branch, but follow the checkout when the
    // card was still diffing against its own HEAD.
    const branch = worktreeBranch(worktree)
    const followingHead =
      !project.currentBranch || project.branch === project.currentBranch
    next.push({
      ...project,
      name: worktreeDisplayName(primary, worktree),
      branch: followingHead ? branch : project.branch,
      currentBranch: branch,
      worktreeHead: worktree.head,
      groupId: project.groupId ?? primary.groupId,
    })
  }

  const additions = linked
    .filter((worktree) => !knownPaths.has(worktree.path))
    .map((worktree) => createWorktreeProject(primary, worktree))
  if (additions.length > 0) {
    // Keep state order aligned with the sidebar: children right after the card.
    const index = next.findIndex((project) => project.id === primary.id)
    if (index >= 0) {
      next.splice(index + 1, 0, ...additions)
    } else {
      next.push(...additions)
    }
  }

  return next
}

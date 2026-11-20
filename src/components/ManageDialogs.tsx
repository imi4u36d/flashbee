import { useEffect, useRef, useState, type FormEvent, type ReactNode } from 'react'
import {
  Check,
  CircleAlert,
  FolderGit2,
  FolderPlus,
  FolderOpen,
  GitBranch,
  GitBranchPlus,
  Layers3,
  LoaderCircle,
  Trash2,
  X,
} from 'lucide-react'
import { BranchSelect } from './BranchSelect'
import { useI18n } from '../lib/i18n'
import {
  RepositoryApiError,
  discoverRepositoryTarget,
  initializeRepository,
  pickRepositoryFolder,
  type RepositoryDiscovery,
  type RepositoryInspection,
} from '../lib/repositoryApi'
import type { RepositoryAddRequest } from '../lib/workspaces'

interface DialogFrameProps {
  title: string
  description?: string
  icon: ReactNode
  iconTone?: 'amber' | 'danger'
  onClose: () => void
  wide?: boolean
  large?: boolean
  children: ReactNode
}

export function DialogFrame({
  title,
  description,
  icon,
  iconTone = 'amber',
  onClose,
  wide = false,
  large = false,
  children,
}: DialogFrameProps) {
  const dialogRef = useRef<HTMLDivElement>(null)
  const { t } = useI18n()

  useEffect(() => {
    const dialog = dialogRef.current
    if (!dialog) return
    const opener = document.activeElement as HTMLElement | null
    const focusables = () =>
      Array.from(
        dialog.querySelectorAll<HTMLElement>(
          'button:not([disabled]), input:not([disabled]), [href], [tabindex]:not([tabindex="-1"])',
        ),
      ).filter((element) => element.getClientRects().length > 0)

    const frame = window.requestAnimationFrame(() => {
      const preferred =
        dialog.querySelector<HTMLElement>('[data-autofocus]') ?? focusables()[0]
      preferred?.focus()
    })

    const trapFocus = (event: KeyboardEvent) => {
      if (event.key !== 'Tab') return
      const elements = focusables()
      if (elements.length === 0) return
      const first = elements[0]
      const last = elements[elements.length - 1]

      if (event.shiftKey && document.activeElement === first) {
        event.preventDefault()
        last.focus()
      } else if (!event.shiftKey && document.activeElement === last) {
        event.preventDefault()
        first.focus()
      }
    }

    dialog.addEventListener('keydown', trapFocus)
    return () => {
      window.cancelAnimationFrame(frame)
      dialog.removeEventListener('keydown', trapFocus)
      opener?.focus?.()
    }
  }, [])

  return (
    <div className="manage-layer">
      <button
        className="manage-backdrop"
        onClick={onClose}
        aria-label={t('common.close')}
        tabIndex={-1}
      />
      <div
        ref={dialogRef}
        className={`manage-dialog ${wide ? 'is-wide' : ''} ${large ? 'is-large' : ''}`}
        role="dialog"
        aria-modal="true"
        aria-labelledby="manage-dialog-title"
      >
        <header className="manage-header">
          <div className="manage-heading">
            <span className={`manage-icon is-${iconTone}`}>{icon}</span>
            <div className="manage-heading-copy">
              <h2 id="manage-dialog-title">{title}</h2>
              {description && <p>{description}</p>}
            </div>
          </div>
          <button
            className="icon-button subtle"
            onClick={onClose}
            aria-label={t('common.close')}
            title={t('common.close')}
          >
            <X size={16} strokeWidth={1.9} />
          </button>
        </header>
        {children}
      </div>
    </div>
  )
}

export function FieldError({ message }: { message: string | null }) {
  if (!message) return null
  return (
    <span className="manage-error" role="alert">
      <CircleAlert size={12} strokeWidth={2} />
      {message}
    </span>
  )
}

interface CreateWorkspaceDialogProps {
  existingNames: string[]
  onConfirm: (name: string) => void
  onClose: () => void
}

export function CreateWorkspaceDialog({
  existingNames,
  onConfirm,
  onClose,
}: CreateWorkspaceDialogProps) {
  const { t } = useI18n()
  const [name, setName] = useState('')
  const [error, setError] = useState<string | null>(null)

  const submit = (event: FormEvent) => {
    event.preventDefault()
    const trimmed = name.trim()
    if (!trimmed) {
      setError(t('workspace.nameRequired'))
      return
    }
    if (
      existingNames.some(
        (existing) => existing.toLowerCase() === trimmed.toLowerCase(),
      )
    ) {
      setError(t('workspace.nameExists'))
      return
    }
    onConfirm(trimmed)
  }

  return (
    <DialogFrame
      title={t('workspace.createTitle')}
      description={t('workspace.createDescription')}
      icon={<Layers3 size={16} strokeWidth={1.9} />}
      onClose={onClose}
    >
      <form className="manage-form" onSubmit={submit}>
        <div className="manage-body">
          <label className="manage-field">
            <span>{t('workspace.nameLabel')}</span>
            <input
              data-autofocus
              autoFocus
              value={name}
              onChange={(event) => {
                setName(event.target.value)
                setError(null)
              }}
              placeholder={t('workspace.namePlaceholder')}
              aria-invalid={Boolean(error)}
              aria-describedby={error ? 'workspace-name-error' : undefined}
              autoComplete="off"
            />
          </label>
          <span id="workspace-name-error">
            <FieldError message={error} />
          </span>
        </div>
        <footer className="manage-footer">
          <button type="button" className="manage-cancel" onClick={onClose}>
            {t('common.cancel')}
          </button>
          <button type="submit" className="manage-submit">
            <Layers3 size={13} strokeWidth={2} />
            {t('common.create')}
          </button>
        </footer>
      </form>
    </DialogFrame>
  )
}

interface AddRepositoryDialogProps {
  workspaceName: string
  existingPaths: string[]
  onConfirm: (request: RepositoryAddRequest) => void
  onClose: () => void
}

function relativeToFolder(folderPath: string, repositoryPath: string) {
  const base = folderPath.replace(/[\\/]+$/, '')
  if (repositoryPath === base) return '.'
  if (
    repositoryPath.startsWith(`${base}/`) ||
    repositoryPath.startsWith(`${base}\\`)
  ) {
    return repositoryPath.slice(base.length + 1)
  }
  return repositoryPath
}

export function AddRepositoryDialog({
  workspaceName,
  existingPaths,
  onConfirm,
  onClose,
}: AddRepositoryDialogProps) {
  const { t } = useI18n()
  const [pathInput, setPathInput] = useState('')
  const [repository, setRepository] = useState<RepositoryInspection | null>(null)
  const [folder, setFolder] = useState<RepositoryDiscovery | null>(null)
  const [branch, setBranch] = useState('')
  const [selectedPaths, setSelectedPaths] = useState<string[]>([])
  const [busy, setBusy] = useState<
    'idle' | 'picking' | 'inspecting' | 'initializing'
  >('idle')
  const [error, setError] = useState<string | null>(null)

  const existing = new Set(existingPaths)
  const duplicate = Boolean(repository && existing.has(repository.path))
  const displayedError = duplicate ? t('repo.pathExists') : error

  const availableRepositories = folder
    ? folder.repositories.filter((entry) => !existing.has(entry.path))
    : []
  const selectedRepositories = folder
    ? folder.repositories.filter(
        (entry) =>
          !existing.has(entry.path) && selectedPaths.includes(entry.path),
      )
    : []
  const allSelected =
    availableRepositories.length > 0 &&
    availableRepositories.every((entry) => selectedPaths.includes(entry.path))

  const errorFor = (reason: unknown) => {
    const code = reason instanceof RepositoryApiError ? reason.code : 'unknown'
    switch (code) {
      case 'not_git':
        return t('repo.notGit')
      case 'not_found':
        return t('repo.notFound')
      case 'not_directory':
        return t('repo.notDirectory')
      case 'picker_unsupported':
        return t('repo.pickerUnsupported')
      case 'api_unavailable':
        return t('repo.serviceUnavailable')
      case 'git_unavailable':
        return t('repo.gitUnavailable')
      default:
        return t('repo.inspectFailed')
    }
  }

  const applyDiscovery = (result: RepositoryDiscovery) => {
    setError(null)
    setPathInput(result.path)

    const inspection = result.repositories[0]
    if (result.isRepository && inspection) {
      setRepository(inspection)
      setFolder(null)
      setSelectedPaths([])
      setBranch(inspection.currentBranch)
      return
    }

    setRepository(null)
    setFolder(result)
    setBranch('')
    setSelectedPaths(
      result.repositories
        .map((entry) => entry.path)
        .filter((path) => !existingPaths.includes(path)),
    )
  }

  const chooseFolder = async () => {
    setBusy('picking')
    setError(null)
    try {
      const path = await pickRepositoryFolder()
      setBusy('inspecting')
      applyDiscovery(await discoverRepositoryTarget(path))
    } catch (reason) {
      if (reason instanceof RepositoryApiError && reason.code === 'cancelled') {
        setError(null)
      } else {
        setError(errorFor(reason))
      }
    } finally {
      setBusy('idle')
    }
  }

  const inspectPath = async (event: FormEvent) => {
    event.preventDefault()
    if (!pathInput.trim()) {
      setError(t('repo.pathRequired'))
      return
    }
    setBusy('inspecting')
    setError(null)
    try {
      applyDiscovery(await discoverRepositoryTarget(pathInput))
    } catch (reason) {
      setError(errorFor(reason))
    } finally {
      setBusy('idle')
    }
  }

  const initialize = async () => {
    if (!folder || busy !== 'idle') return
    setBusy('initializing')
    setError(null)
    try {
      const inspection = await initializeRepository(folder.path)
      setRepository(inspection)
      setFolder(null)
      setSelectedPaths([])
      setBranch(inspection.currentBranch)
      setPathInput(inspection.path)
    } catch (reason) {
      const code = reason instanceof RepositoryApiError ? reason.code : 'unknown'
      if (code === 'not_found') {
        setError(t('repo.notFound'))
      } else if (code === 'not_directory') {
        setError(t('repo.notDirectory'))
      } else if (code === 'git_unavailable') {
        setError(t('repo.gitUnavailable'))
      } else {
        setError(t('repo.initFailed'))
      }
    } finally {
      setBusy('idle')
    }
  }

  const toggleRepository = (path: string) => {
    setSelectedPaths((current) =>
      current.includes(path)
        ? current.filter((entry) => entry !== path)
        : [...current, path],
    )
  }

  const toggleAll = () => {
    setSelectedPaths(
      allSelected ? [] : availableRepositories.map((entry) => entry.path),
    )
  }

  const resetSelection = () => {
    setRepository(null)
    setFolder(null)
    setPathInput('')
    setBranch('')
    setSelectedPaths([])
    setError(null)
  }

  return (
    <DialogFrame
      title={t('repo.addTitle')}
      description={t('repo.addDescription')}
      icon={<FolderPlus size={16} strokeWidth={1.9} />}
      onClose={onClose}
      wide
    >
      <div className="manage-body">
        <div className="manage-workspace-target">
          <Layers3 size={12} strokeWidth={1.9} />
          {workspaceName}
        </div>

        {!repository && !folder ? (
          <div className="repo-picker">
            <button
              type="button"
              className="repo-choose-button"
              data-autofocus
              onClick={chooseFolder}
              disabled={busy !== 'idle'}
            >
              {busy === 'picking' || busy === 'inspecting' ? (
                <LoaderCircle size={17} strokeWidth={2} className="is-spinning" />
              ) : (
                <FolderOpen size={17} strokeWidth={1.9} />
              )}
              <span>
                {busy === 'picking'
                  ? t('repo.pickingFolder')
                  : busy === 'inspecting'
                    ? t('repo.discovering')
                    : t('repo.chooseFolder')}
              </span>
            </button>

            <div className="repo-picker-divider">
              <span>{t('repo.orPath')}</span>
            </div>

            <form className="repo-path-form" onSubmit={inspectPath}>
              <input
                data-testid="repo-path-input"
                value={pathInput}
                onChange={(event) => {
                  setPathInput(event.target.value)
                  setError(null)
                }}
                placeholder={t('repo.pathPlaceholder')}
                autoComplete="off"
                spellCheck={false}
                disabled={busy !== 'idle'}
                aria-label={t('repo.pathLabel')}
              />
              <button
                type="submit"
                disabled={busy !== 'idle'}
                className="repo-inspect-button"
              >
                {busy === 'inspecting' ? (
                  <LoaderCircle size={13} strokeWidth={2} className="is-spinning" />
                ) : (
                  t('repo.inspectPath')
                )}
              </button>
            </form>
            <FieldError message={displayedError} />
          </div>
        ) : null}

        {repository ? (
          <div className="repo-picked">
            <div className="repo-picked-heading">
              <span className="repo-picked-mark">
                {repository.name.charAt(0).toUpperCase()}
              </span>
              <span className="repo-picked-copy">
                <strong>{repository.name}</strong>
                <span>{repository.path}</span>
              </span>
              <button
                type="button"
                className="repo-change-folder"
                onClick={resetSelection}
              >
                {t('repo.changeFolder')}
              </button>
            </div>

            <div className="repo-branch-row">
              <div className="repo-branch-copy">
                <span>{t('repo.branchLabel')}</span>
                <small>{t('repo.currentBranchHint')}</small>
              </div>
              <BranchSelect
                variant="form"
                branches={repository.branches}
                value={branch}
                onChange={setBranch}
              />
            </div>
            <FieldError message={displayedError} />
          </div>
        ) : null}

        {folder ? (
          <div className="repo-picked">
            <div className="repo-picked-heading">
              <span className="repo-picked-mark">
                {folder.name.charAt(0).toUpperCase()}
              </span>
              <span className="repo-picked-copy">
                <strong>{folder.name}</strong>
                <span>{folder.path}</span>
              </span>
              <button
                type="button"
                className="repo-change-folder"
                onClick={resetSelection}
              >
                {t('repo.changeFolder')}
              </button>
            </div>

            {folder.repositories.length > 0 ? (
              <>
                <div className="repo-folder-note">
                  <span className="repo-folder-note-icon">
                    <FolderGit2 size={16} strokeWidth={1.8} />
                  </span>
                  <span className="repo-folder-note-copy">
                    <strong>{t('repo.notGit')}</strong>
                    <span>
                      {t(
                        folder.repositories.length === 1
                          ? 'repo.folderFoundOne'
                          : 'repo.folderFoundMany',
                        { count: folder.repositories.length },
                      )}
                    </span>
                  </span>
                </div>

                <div className="repo-discovery">
                  <div className="repo-discovery-toolbar">
                    <span>{t('repo.selectRepositories')}</span>
                    <button type="button" onClick={toggleAll}>
                      {allSelected
                        ? t('repo.deselectAll')
                        : t('repo.selectAll')}
                    </button>
                  </div>

                  <div
                    className="repo-discovery-list"
                    role="group"
                    aria-label={t('repo.selectRepositories')}
                  >
                    {folder.repositories.map((entry) => {
                      const added = existing.has(entry.path)
                      const selected =
                        !added && selectedPaths.includes(entry.path)
                      return (
                        <button
                          key={entry.path}
                          type="button"
                          role="checkbox"
                          aria-checked={selected}
                          disabled={added}
                          className={`repo-discovery-row ${
                            selected ? 'is-selected' : ''
                          } ${added ? 'is-added' : ''}`}
                          onClick={() => toggleRepository(entry.path)}
                        >
                          <span className="repo-discovery-check">
                            {selected || added ? (
                              <Check size={11} strokeWidth={2.8} />
                            ) : null}
                          </span>
                          <span className="repo-discovery-copy">
                            <span className="repo-discovery-name">
                              <span className="repo-discovery-label">
                                {entry.name}
                              </span>
                              {added ? (
                                <span className="repo-discovery-added">
                                  {t('repo.alreadyAdded')}
                                </span>
                              ) : null}
                            </span>
                            <span className="repo-discovery-path">
                              {relativeToFolder(folder.path, entry.path)}
                            </span>
                          </span>
                          <span className="repo-discovery-branch">
                            <GitBranch size={10} strokeWidth={2} />
                            {entry.currentBranch}
                          </span>
                        </button>
                      )
                    })}
                  </div>

                  {folder.truncated ? (
                    <span className="repo-discovery-hint">
                      {t('repo.discoveryTruncated')}
                    </span>
                  ) : null}
                </div>

                <button
                  type="button"
                  className="repo-folder-init-link"
                  onClick={() => void initialize()}
                  disabled={busy !== 'idle'}
                >
                  {busy === 'initializing' ? (
                    <LoaderCircle
                      size={13}
                      strokeWidth={2}
                      className="is-spinning"
                    />
                  ) : (
                    <GitBranchPlus size={13} strokeWidth={1.9} />
                  )}
                  {busy === 'initializing'
                    ? t('repo.initializing')
                    : t('repo.initInstead')}
                </button>
                <FieldError message={displayedError} />
              </>
            ) : (
              <>
                <div className="repo-folder-empty">
                  <span className="repo-folder-empty-icon">
                    <FolderGit2 size={21} strokeWidth={1.7} />
                  </span>
                  <strong>{t('repo.notGit')}</strong>
                  <p>
                    {folder.truncated
                      ? t('repo.discoveryTruncated')
                      : t('repo.folderEmpty')}
                  </p>
                  <small>{t('repo.initHint')}</small>
                </div>
                <FieldError message={displayedError} />
              </>
            )}
          </div>
        ) : null}
      </div>

      <footer className="manage-footer">
        <button type="button" className="manage-cancel" onClick={onClose}>
          {t('common.cancel')}
        </button>

        {repository ? (
          <button
            type="button"
            className="manage-submit"
            disabled={duplicate || busy !== 'idle'}
            onClick={() =>
              onConfirm({
                repositories: [
                  {
                    name: repository.name,
                    path: repository.path,
                    branch: branch || repository.currentBranch,
                    branches: repository.branches,
                  },
                ],
              })
            }
          >
            <FolderPlus size={13} strokeWidth={2} />
            {t('repo.add')}
          </button>
        ) : null}

        {folder && folder.repositories.length > 0 ? (
          <button
            type="button"
            className="manage-submit"
            disabled={selectedRepositories.length === 0 || busy !== 'idle'}
            onClick={() =>
              onConfirm({
                group: { name: folder.name, path: folder.path },
                repositories: selectedRepositories.map((entry) => ({
                  name: entry.name,
                  path: entry.path,
                  branch: entry.currentBranch,
                  branches: entry.branches,
                })),
              })
            }
          >
            <FolderPlus size={13} strokeWidth={2} />
            {selectedRepositories.length > 1
              ? t('repo.addMany', { count: selectedRepositories.length })
              : t('repo.add')}
          </button>
        ) : null}

        {folder && folder.repositories.length === 0 ? (
          <button
            type="button"
            className="manage-submit"
            disabled={busy !== 'idle'}
            onClick={() => void initialize()}
          >
            {busy === 'initializing' ? (
              <LoaderCircle size={13} strokeWidth={2} className="is-spinning" />
            ) : (
              <GitBranchPlus size={13} strokeWidth={2} />
            )}
            {busy === 'initializing'
              ? t('repo.initializing')
              : t('repo.initHere')}
          </button>
        ) : null}
      </footer>
    </DialogFrame>
  )
}

interface ConfirmRemoveDialogProps {
  title: string
  description: string
  confirmLabel: string
  onConfirm: () => void
  onClose: () => void
}

export function ConfirmRemoveDialog({
  title,
  description,
  confirmLabel,
  onConfirm,
  onClose,
}: ConfirmRemoveDialogProps) {
  const { t } = useI18n()

  return (
    <DialogFrame
      title={title}
      icon={<Trash2 size={16} strokeWidth={1.9} />}
      iconTone="danger"
      onClose={onClose}
    >
      <div className="manage-body manage-confirm-body">
        <p>{description}</p>
      </div>
      <footer className="manage-footer">
        <button className="manage-cancel" onClick={onClose} data-autofocus>
          {t('common.cancel')}
        </button>
        <button className="manage-submit is-danger" onClick={onConfirm}>
          <Trash2 size={13} strokeWidth={2} />
          {confirmLabel}
        </button>
      </footer>
    </DialogFrame>
  )
}

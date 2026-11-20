import { useEffect, useRef, useState, type FormEvent, type ReactNode } from 'react'
import {
  CircleAlert,
  FolderPlus,
  FolderOpen,
  Layers3,
  LoaderCircle,
  Trash2,
  X,
} from 'lucide-react'
import { BranchSelect } from './BranchSelect'
import { useI18n } from '../lib/i18n'
import {
  RepositoryApiError,
  inspectRepositoryPath,
  pickRepositoryDirectory,
  type RepositoryInspection,
} from '../lib/repositoryApi'
import type { NewRepositoryInput } from '../lib/workspaces'

interface DialogFrameProps {
  title: string
  description?: string
  icon: ReactNode
  iconTone?: 'amber' | 'danger'
  onClose: () => void
  wide?: boolean
  children: ReactNode
}

function DialogFrame({
  title,
  description,
  icon,
  iconTone = 'amber',
  onClose,
  wide = false,
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
        className={`manage-dialog ${wide ? 'is-wide' : ''}`}
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

function FieldError({ message }: { message: string | null }) {
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
  onConfirm: (input: NewRepositoryInput) => void
  onClose: () => void
}

export function AddRepositoryDialog({
  workspaceName,
  existingPaths,
  onConfirm,
  onClose,
}: AddRepositoryDialogProps) {
  const { t } = useI18n()
  const [pathInput, setPathInput] = useState('')
  const [inspection, setInspection] = useState<RepositoryInspection | null>(null)
  const [branch, setBranch] = useState('')
  const [busy, setBusy] = useState<'idle' | 'picking' | 'inspecting'>('idle')
  const [error, setError] = useState<string | null>(null)

  const duplicate = Boolean(
    inspection && existingPaths.includes(inspection.path),
  )
  const displayedError = duplicate ? t('repo.pathExists') : error

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
      default:
        return t('repo.inspectFailed')
    }
  }

  const applyInspection = (result: RepositoryInspection) => {
    setInspection(result)
    setPathInput(result.path)
    setBranch(result.currentBranch)
    setError(null)
  }

  const chooseFolder = async () => {
    setBusy('picking')
    setError(null)
    try {
      applyInspection(await pickRepositoryDirectory())
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
      applyInspection(await inspectRepositoryPath(pathInput))
    } catch (reason) {
      setError(errorFor(reason))
    } finally {
      setBusy('idle')
    }
  }

  const resetSelection = () => {
    setInspection(null)
    setPathInput('')
    setBranch('')
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

        {!inspection ? (
          <div className="repo-picker">
            <button
              type="button"
              className="repo-choose-button"
              data-autofocus
              onClick={chooseFolder}
              disabled={busy !== 'idle'}
            >
              {busy === 'picking' ? (
                <LoaderCircle size={17} strokeWidth={2} className="is-spinning" />
              ) : (
                <FolderOpen size={17} strokeWidth={1.9} />
              )}
              <span>
                {busy === 'picking'
                  ? t('repo.pickingFolder')
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
        ) : (
          <div className="repo-picked">
            <div className="repo-picked-heading">
              <span className="repo-picked-mark">
                {inspection.name.charAt(0).toUpperCase()}
              </span>
              <span className="repo-picked-copy">
                <strong>{inspection.name}</strong>
                <span>{inspection.path}</span>
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
                branches={inspection.branches}
                value={branch}
                onChange={setBranch}
              />
            </div>
            <FieldError message={displayedError} />
          </div>
        )}
      </div>
      <footer className="manage-footer">
        <button type="button" className="manage-cancel" onClick={onClose}>
          {t('common.cancel')}
        </button>
        <button
          type="button"
          className="manage-submit"
          disabled={!inspection || duplicate || busy !== 'idle'}
          onClick={() => {
            if (!inspection) return
            onConfirm({
              name: inspection.name,
              path: inspection.path,
              branch: branch || inspection.currentBranch,
              branches: inspection.branches,
            })
          }}
        >
          <FolderPlus size={13} strokeWidth={2} />
          {t('repo.add')}
        </button>
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

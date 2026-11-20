import { useEffect, useState, type FormEvent } from 'react'
import { Cloud, Link2, LoaderCircle, Plus } from 'lucide-react'
import { DialogFrame, FieldError } from './ManageDialogs'
import { useI18n } from '../lib/i18n'
import {
  RepositoryApiError,
  fetchRemotes,
  setRemoteUrl,
} from '../lib/repositoryApi'
import type { Project, RemoteInfo } from '../types'

interface RemoteDialogProps {
  project: Project
  onSaved: (remotes: RemoteInfo[]) => void
  onClose: () => void
}

export function RemoteDialog({ project, onSaved, onClose }: RemoteDialogProps) {
  const { t } = useI18n()
  const [remotes, setRemotes] = useState<RemoteInfo[]>(project.remotes ?? [])
  const [name, setName] = useState(project.remotes?.[0]?.name ?? 'origin')
  const [url, setUrl] = useState(project.remotes?.[0]?.url ?? '')
  const [loading, setLoading] = useState(true)
  const [saving, setSaving] = useState(false)
  const [error, setError] = useState<string | null>(null)

  useEffect(() => {
    let cancelled = false
    fetchRemotes(project.path)
      .then((entries) => {
        if (cancelled) return
        setRemotes(entries)
        const preferred =
          entries.find((entry) => entry.name === 'origin') ?? entries[0]
        if (preferred) {
          setName(preferred.name)
          setUrl(preferred.url)
        }
      })
      .catch(() => {
        if (!cancelled) setError(t('remote.loadFailed'))
      })
      .finally(() => {
        if (!cancelled) setLoading(false)
      })
    return () => {
      cancelled = true
    }
  }, [project.path, t])

  const startNew = () => {
    setName(remotes.some((remote) => remote.name === 'origin') ? 'upstream' : 'origin')
    setUrl('')
    setError(null)
  }

  const selectRemote = (remote: RemoteInfo) => {
    setName(remote.name)
    setUrl(remote.url)
    setError(null)
  }

  const submit = async (event: FormEvent) => {
    event.preventDefault()
    if (saving) return

    const trimmedName = name.trim()
    const trimmedUrl = url.trim()
    if (!trimmedName || !/^[A-Za-z0-9._-]+$/.test(trimmedName)) {
      setError(t('remote.invalidName'))
      return
    }
    if (!trimmedUrl) {
      setError(t('remote.urlRequired'))
      return
    }

    setSaving(true)
    setError(null)
    try {
      const entries = await setRemoteUrl(project.path, trimmedName, trimmedUrl)
      onSaved(entries)
      onClose()
    } catch (reason) {
      if (reason instanceof RepositoryApiError && reason.code === 'bad_request') {
        setError(t('remote.invalidName'))
      } else if (reason instanceof Error && reason.message) {
        setError(reason.message)
      } else {
        setError(t('remote.saveFailed'))
      }
    } finally {
      setSaving(false)
    }
  }

  const exists = remotes.some((remote) => remote.name === name.trim())

  return (
    <DialogFrame
      title={t('remote.title')}
      description={t('remote.description', { name: project.name })}
      icon={<Cloud size={16} strokeWidth={1.9} />}
      onClose={onClose}
      wide
    >
      <form className="manage-form" onSubmit={submit}>
        <div className="manage-body remote-body">
          <div className="remote-list-heading">
            <span>{t('remote.current')}</span>
            <button type="button" className="remote-new" onClick={startNew}>
              <Plus size={12} strokeWidth={2.1} />
              {t('remote.add')}
            </button>
          </div>

          {loading ? (
            <div className="remote-loading">
              <LoaderCircle size={15} strokeWidth={2} className="is-spinning" />
              {t('remote.loading')}
            </div>
          ) : remotes.length === 0 ? (
            <div className="remote-empty">
              <Cloud size={18} strokeWidth={1.6} />
              <span>{t('remote.empty')}</span>
            </div>
          ) : (
            <div className="remote-list" role="listbox" aria-label={t('remote.current')}>
              {remotes.map((remote) => {
                const active = remote.name === name.trim()
                return (
                  <button
                    key={remote.name}
                    type="button"
                    role="option"
                    aria-selected={active}
                    className={`remote-row ${active ? 'is-selected' : ''}`}
                    onClick={() => selectRemote(remote)}
                  >
                    <span className="remote-row-icon">
                      <Cloud size={13} strokeWidth={1.9} />
                    </span>
                    <span className="remote-row-copy">
                      <strong>{remote.name}</strong>
                      <span>{remote.url}</span>
                    </span>
                    {active && <span className="remote-row-badge">{t('remote.editing')}</span>}
                  </button>
                )
              })}
            </div>
          )}

          <div className="remote-form-divider" />

          <label className="manage-field">
            <span>{t('remote.nameLabel')}</span>
            <input
              data-autofocus
              value={name}
              onChange={(event) => {
                setName(event.target.value)
                setError(null)
              }}
              placeholder="origin"
              autoComplete="off"
              spellCheck={false}
            />
          </label>

          <label className="manage-field">
            <span>{t('remote.urlLabel')}</span>
            <div className="remote-url-field">
              <Link2 size={13} strokeWidth={1.9} />
              <input
                value={url}
                onChange={(event) => {
                  setUrl(event.target.value)
                  setError(null)
                }}
                placeholder="git@github.com:owner/repository.git"
                autoComplete="off"
                spellCheck={false}
              />
            </div>
          </label>

          <p className="remote-hint">{t('remote.hint')}</p>
          <FieldError message={error} />
        </div>

        <footer className="manage-footer">
          <button type="button" className="manage-cancel" onClick={onClose}>
            {t('common.cancel')}
          </button>
          <button
            type="submit"
            className="manage-submit"
            disabled={loading || saving || !url.trim()}
          >
            {saving ? (
              <LoaderCircle size={13} strokeWidth={2} className="is-spinning" />
            ) : (
              <Cloud size={13} strokeWidth={2} />
            )}
            {exists ? t('remote.update') : t('remote.save')}
          </button>
        </footer>
      </form>
    </DialogFrame>
  )
}

import { useEffect, useRef, useState } from 'react'
import { Check, ChevronDown, Cloud, GitBranch } from 'lucide-react'
import { useI18n } from '../lib/i18n'
import type { RemoteBranch } from '../types'

interface BranchSelectProps {
  branches: string[]
  remoteBranches?: RemoteBranch[]
  value: string
  onChange: (branch: string) => void
  variant?: 'toolbar' | 'form'
  disabled?: boolean
}

export function BranchSelect({
  branches,
  remoteBranches = [],
  value,
  onChange,
  variant = 'toolbar',
  disabled = false,
}: BranchSelectProps) {
  const { t } = useI18n()
  const [open, setOpen] = useState(false)
  const rootRef = useRef<HTMLDivElement>(null)
  const localOptions = branches.length > 0 ? branches : value ? [value] : []
  const remoteOptions = remoteBranches.filter(
    (branch) => !branches.includes(branch.name),
  )

  useEffect(() => {
    if (!open) return

    const closeOnOutsideClick = (event: PointerEvent) => {
      if (!rootRef.current?.contains(event.target as Node)) {
        setOpen(false)
      }
    }
    const closeOnEscape = (event: KeyboardEvent) => {
      if (event.key === 'Escape') {
        event.stopPropagation()
        setOpen(false)
      }
    }

    document.addEventListener('pointerdown', closeOnOutsideClick)
    window.addEventListener('keydown', closeOnEscape, true)
    return () => {
      document.removeEventListener('pointerdown', closeOnOutsideClick)
      window.removeEventListener('keydown', closeOnEscape, true)
    }
  }, [open])

  return (
    <div className={`branch-select branch-select-${variant}`} ref={rootRef}>
      <button
        type="button"
        className={variant === 'toolbar' ? 'branch-selector' : 'branch-form-trigger'}
        title={t('repo.switchBranch')}
        onClick={() => setOpen((current) => !current)}
        disabled={disabled}
        aria-haspopup="menu"
        aria-expanded={open}
      >
        <GitBranch size={13} strokeWidth={1.9} />
        <span>{value || '—'}</span>
        <ChevronDown size={12} className={`branch-chevron ${open ? 'is-open' : ''}`} />
      </button>

      {open && (
        <div className={`branch-menu ${variant === 'form' ? 'is-form' : ''}`} role="menu">
          <div className="branch-menu-label">{t('repo.localBranches')}</div>
          {localOptions.length === 0 ? (
            <div className="branch-menu-empty">{t('repo.noBranches')}</div>
          ) : (
            localOptions.map((branch) => {
              const selected = branch === value
              return (
                <button
                  key={`local-${branch}`}
                  type="button"
                  role="menuitemradio"
                  aria-checked={selected}
                  className={`branch-menu-item ${selected ? 'is-selected' : ''}`}
                  onClick={() => {
                    onChange(branch)
                    setOpen(false)
                  }}
                >
                  <GitBranch size={12} strokeWidth={1.9} />
                  <span>{branch}</span>
                  {selected && <Check size={12} strokeWidth={2.3} />}
                </button>
              )
            })
          )}

          {remoteOptions.length > 0 && (
            <>
              <div className="branch-menu-label is-remote">
                {t('repo.remoteBranches')}
              </div>
              {remoteOptions.map((branch) => {
                const selected = branch.name === value
                return (
                  <button
                    key={`remote-${branch.name}`}
                    type="button"
                    role="menuitemradio"
                    aria-checked={selected}
                    className={`branch-menu-item is-remote ${
                      selected ? 'is-selected' : ''
                    }`}
                    title={branch.name}
                    onClick={() => {
                      onChange(branch.name)
                      setOpen(false)
                    }}
                  >
                    <Cloud size={12} strokeWidth={1.9} />
                    <span>
                      <span className="branch-remote-prefix">
                        {branch.remote}/
                      </span>
                      {branch.branch}
                    </span>
                    {selected && <Check size={12} strokeWidth={2.3} />}
                  </button>
                )
              })}
            </>
          )}
        </div>
      )}
    </div>
  )
}

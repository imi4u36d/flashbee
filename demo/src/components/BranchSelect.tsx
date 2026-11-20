import { useEffect, useRef, useState } from 'react'
import { Check, ChevronDown, GitBranch } from 'lucide-react'
import { useI18n } from '../lib/i18n'

interface BranchSelectProps {
  branches: string[]
  value: string
  onChange: (branch: string) => void
  variant?: 'toolbar' | 'form'
  disabled?: boolean
}

export function BranchSelect({
  branches,
  value,
  onChange,
  variant = 'toolbar',
  disabled = false,
}: BranchSelectProps) {
  const { t } = useI18n()
  const [open, setOpen] = useState(false)
  const rootRef = useRef<HTMLDivElement>(null)
  const options = branches.length > 0 ? branches : value ? [value] : []

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
          <div className="branch-menu-label">{t('repo.branches')}</div>
          {options.length === 0 ? (
            <div className="branch-menu-empty">{t('repo.noBranches')}</div>
          ) : (
            options.map((branch) => {
              const selected = branch === value
              return (
                <button
                  key={branch}
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
        </div>
      )}
    </div>
  )
}

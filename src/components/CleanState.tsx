import { Check, GitBranch, RefreshCw } from 'lucide-react'
import { useI18n } from '../lib/i18n'
import type { Project } from '../types'

interface CleanStateProps {
  project: Project
  onRefresh: () => void
}

export function CleanState({ project, onRefresh }: CleanStateProps) {
  const { t } = useI18n()

  return (
    <div className="clean-workspace">
      <div className="clean-visual" aria-hidden="true">
        <span className="clean-ring clean-ring-outer" />
        <span className="clean-ring clean-ring-inner" />
        <span className="clean-orbit clean-orbit-one">
          <span />
        </span>
        <span className="clean-orbit clean-orbit-two">
          <span />
        </span>
        <span className="clean-check">
          <Check size={28} strokeWidth={2.2} />
        </span>
        <span className="clean-branch-chip">
          <GitBranch size={11} strokeWidth={2} />
          {project.branch}
        </span>
      </div>

      <span className="eyebrow clean-eyebrow">{t('clean.eyebrow')}</span>
      <h2>{t('clean.title')}</h2>
      <p>{t('clean.description', { project: project.name })}</p>

      <div className="clean-facts">
        <div>
          <span>{t('clean.repository')}</span>
          <strong>{project.name}</strong>
        </div>
        <div>
          <span>{t('clean.branch')}</span>
          <strong>
            <GitBranch size={12} strokeWidth={1.9} />
            {project.branch}
          </strong>
        </div>
        <div>
          <span>{t('clean.lastChecked')}</span>
          <strong className="clean-live">{t('clean.justNow')}</strong>
        </div>
      </div>

      <button className="clean-refresh" onClick={onRefresh}>
        <RefreshCw size={14} strokeWidth={1.9} />
        {t('clean.check')}
      </button>
    </div>
  )
}

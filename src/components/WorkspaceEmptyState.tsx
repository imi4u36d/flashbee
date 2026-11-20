import { FolderPlus, Layers3, Sparkles } from 'lucide-react'
import { useI18n } from '../lib/i18n'

interface WorkspaceEmptyStateProps {
  workspaceName: string
  onAddRepository: () => void
  onExploreSample: () => void
}

export function WorkspaceEmptyState({
  workspaceName,
  onAddRepository,
  onExploreSample,
}: WorkspaceEmptyStateProps) {
  const { t } = useI18n()

  return (
    <div className="clean-pane">
      <div className="clean-workspace workspace-empty">
        <div className="clean-visual">
          <span className="clean-ring clean-ring-outer" />
          <span className="clean-ring clean-ring-inner" />
          <span className="clean-orbit clean-orbit-one">
            <span />
          </span>
          <span className="clean-orbit clean-orbit-two">
            <span />
          </span>
          <span className="clean-check is-empty">
            <Layers3 size={26} strokeWidth={1.8} />
          </span>
          <span className="clean-branch-chip">
            <Layers3 size={11} strokeWidth={2} />
            {workspaceName}
          </span>
        </div>

        <span className="eyebrow clean-eyebrow">{t('workspace.emptyEyebrow')}</span>
        <h2>{t('workspace.emptyTitle')}</h2>
        <p>{t('workspace.emptyDescription')}</p>

        <div className="empty-actions">
          <button className="empty-add-button" onClick={onAddRepository}>
            <FolderPlus size={14} strokeWidth={2} />
            {t('workspace.emptyCta')}
          </button>
          <button className="empty-sample-button" onClick={onExploreSample}>
            <Sparkles size={13} strokeWidth={1.9} />
            {t('workspace.emptySample')}
          </button>
        </div>
      </div>
    </div>
  )
}

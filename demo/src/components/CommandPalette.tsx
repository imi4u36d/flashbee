import { useEffect, useMemo, useRef, useState } from 'react'
import type { KeyboardEvent } from 'react'
import {
  CornerDownLeft,
  FileCode2,
  GitBranch,
  Search,
} from 'lucide-react'
import { fileStats, pathParts, statusLabels } from '../data/projects'
import { useI18n } from '../lib/i18n'
import type { DiffFile, Project } from '../types'

type PaletteItem =
  | { kind: 'project'; project: Project }
  | { kind: 'file'; project: Project; file: DiffFile }

interface CommandPaletteProps {
  open: boolean
  projects: Project[]
  activeProjectId: string
  onClose: () => void
  onSelectProject: (project: Project) => void
  onSelectFile: (project: Project, file: DiffFile) => void
}

export function CommandPalette({
  open,
  projects,
  activeProjectId,
  onClose,
  onSelectProject,
  onSelectFile,
}: CommandPaletteProps) {
  const { t } = useI18n()
  const [query, setQuery] = useState('')
  const [selectedIndex, setSelectedIndex] = useState(0)
  const inputRef = useRef<HTMLInputElement>(null)

  const { projectResults, fileResults, results } = useMemo(() => {
    const normalized = query.trim().toLowerCase()

    const matchedProjects = projects.filter((project) => {
      if (!normalized) return true
      return (
        project.name.toLowerCase().includes(normalized) ||
        project.path.toLowerCase().includes(normalized) ||
        project.branch.toLowerCase().includes(normalized)
      )
    })

    const matchedFiles = projects
      .flatMap((project) =>
        project.files.map((file) => ({ project, file })),
      )
      .filter(({ project, file }) => {
        if (!normalized) return project.id === activeProjectId
        return (
          file.path.toLowerCase().includes(normalized) ||
          project.name.toLowerCase().includes(normalized)
        )
      })
      .slice(0, 10)

    const items: PaletteItem[] = [
      ...matchedProjects.map(
        (project): PaletteItem => ({ kind: 'project', project }),
      ),
      ...matchedFiles.map(
        ({ project, file }): PaletteItem => ({ kind: 'file', project, file }),
      ),
    ]

    return {
      projectResults: matchedProjects,
      fileResults: matchedFiles,
      results: items,
    }
  }, [query, projects, activeProjectId])

  useEffect(() => {
    if (!open) return
    const frame = window.requestAnimationFrame(() => inputRef.current?.focus())
    return () => window.cancelAnimationFrame(frame)
  }, [open])

  if (!open) return null

  const activate = (item: PaletteItem | undefined) => {
    if (!item) return
    if (item.kind === 'project') {
      onSelectProject(item.project)
    } else {
      onSelectFile(item.project, item.file)
    }
    onClose()
  }

  const handleKeyDown = (event: KeyboardEvent<HTMLInputElement>) => {
    if (event.key === 'ArrowDown') {
      event.preventDefault()
      setSelectedIndex((current) =>
        results.length ? (current + 1) % results.length : 0,
      )
    }
    if (event.key === 'ArrowUp') {
      event.preventDefault()
      setSelectedIndex((current) =>
        results.length ? (current - 1 + results.length) % results.length : 0,
      )
    }
    if (event.key === 'Enter') {
      event.preventDefault()
      activate(results[selectedIndex])
    }
    if (event.key === 'Escape') {
      event.preventDefault()
      onClose()
    }
  }

  const projectStart = 0
  const fileStart = projectResults.length

  return (
    <div className="palette-layer" role="presentation">
      <button
        className="palette-backdrop"
        onClick={onClose}
        aria-label={t('palette.close')}
        tabIndex={-1}
      />
      <div className="command-palette" role="dialog" aria-modal="true" aria-label={t('palette.dialog')}>
        <div className="palette-input-row">
          <Search size={17} strokeWidth={1.9} className="palette-search-icon" />
          <input
            ref={inputRef}
            value={query}
            onChange={(event) => {
              setQuery(event.target.value)
              setSelectedIndex(0)
            }}
            onKeyDown={handleKeyDown}
            placeholder={t('palette.placeholder')}
            aria-label={t('palette.searchAria')}
          />
          <kbd className="palette-esc">esc</kbd>
        </div>

        <div className="palette-results">
          {projectResults.length > 0 && (
            <div className="palette-group">
              <div className="palette-group-label">
                <span>{t('palette.repositories')}</span>
                <span>{projectResults.length}</span>
              </div>
              {projectResults.map((project, index) => {
                const itemIndex = projectStart + index
                const isSelected = selectedIndex === itemIndex
                return (
                  <button
                    key={project.id}
                    className={`palette-item ${isSelected ? 'is-selected' : ''}`}
                    onMouseEnter={() => setSelectedIndex(itemIndex)}
                    onClick={() => activate({ kind: 'project', project })}
                  >
                    <span
                      className="palette-project-mark"
                      style={{
                        color: project.accent,
                        background: project.accentSoft,
                      }}
                    >
                      {project.name.charAt(0).toUpperCase()}
                    </span>
                    <span className="palette-item-copy">
                      <span className="palette-item-title">
                        {project.name}
                        {project.id === activeProjectId && (
                          <span className="current-chip">{t('common.current')}</span>
                        )}
                      </span>
                      <span className="palette-item-sub">
                        <GitBranch size={10} strokeWidth={2} />
                        {project.branch}
                        <span className="palette-dot">·</span>
                        {t('palette.changed', { count: project.files.length })}
                      </span>
                    </span>
                    <span className="palette-open-hint">
                      <CornerDownLeft size={12} strokeWidth={1.9} />
                    </span>
                  </button>
                )
              })}
            </div>
          )}

          {fileResults.length > 0 && (
            <div className="palette-group">
              <div className="palette-group-label">
                <span>{t('palette.changedFiles')}</span>
                <span>{fileResults.length}</span>
              </div>
              {fileResults.map(({ project, file }, index) => {
                const itemIndex = fileStart + index
                const isSelected = selectedIndex === itemIndex
                const { name, dir } = pathParts(file.path)
                const stats = fileStats(file)
                return (
                  <button
                    key={`${project.id}-${file.path}`}
                    className={`palette-item palette-file-item ${
                      isSelected ? 'is-selected' : ''
                    }`}
                    onMouseEnter={() => setSelectedIndex(itemIndex)}
                    onClick={() => activate({ kind: 'file', project, file })}
                  >
                    <span className="palette-file-icon">
                      <FileCode2 size={14} strokeWidth={1.8} />
                    </span>
                    <span className="palette-item-copy">
                      <span className="palette-item-title">
                        <span className={`palette-status status-${file.status}`}>
                          {statusLabels[file.status]}
                        </span>
                        {name}
                      </span>
                      <span className="palette-item-sub">
                        {dir || project.name}
                        <span className="palette-dot">·</span>
                        {project.name}
                      </span>
                    </span>
                    <span className="palette-file-stats">
                      <span className="delta-add">+{stats.additions}</span>
                      <span className="delta-del">−{stats.deletions}</span>
                    </span>
                  </button>
                )
              })}
            </div>
          )}

          {results.length === 0 && (
            <div className="palette-empty">
              <span className="palette-empty-icon">
                <Search size={17} strokeWidth={1.7} />
              </span>
              <strong>{t('palette.noMatches')}</strong>
              <p>{t('palette.noMatchesHint')}</p>
            </div>
          )}
        </div>

        <footer className="palette-footer">
          <span className="palette-brand">
            <span className="palette-logo">⚡</span>
            {t('palette.brand')}
          </span>
          <span className="palette-keys">
            <span><kbd>↑</kbd><kbd>↓</kbd> {t('palette.navigate')}</span>
            <span><kbd>↵</kbd> {t('palette.open')}</span>
            <span><kbd>esc</kbd> {t('palette.closeKey')}</span>
          </span>
        </footer>
      </div>
    </div>
  )
}

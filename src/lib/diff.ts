import type { DiffFile, DiffLine, Hunk, SplitDiffRow } from '../types'

export function buildSplitRows(hunks: Hunk[]): SplitDiffRow[] {
  const rows: SplitDiffRow[] = []

  for (const hunk of hunks) {
    rows.push({ kind: 'hunk', hunk })

    let cursor = 0
    while (cursor < hunk.lines.length) {
      const line = hunk.lines[cursor]

      if (line.type === 'context') {
        rows.push({ kind: 'split', row: { left: line, right: line } })
        cursor += 1
        continue
      }

      const deleted: DiffLine[] = []
      const added: DiffLine[] = []

      while (hunk.lines[cursor]?.type === 'del') {
        deleted.push(hunk.lines[cursor])
        cursor += 1
      }

      while (hunk.lines[cursor]?.type === 'add') {
        added.push(hunk.lines[cursor])
        cursor += 1
      }

      if (deleted.length === 0 && added.length === 0) {
        cursor += 1
        continue
      }

      const depth = Math.max(deleted.length, added.length)
      for (let index = 0; index < depth; index += 1) {
        rows.push({
          kind: 'split',
          row: { left: deleted[index], right: added[index] },
        })
      }
    }
  }

  return rows
}

export function fileExtension(file: DiffFile) {
  const name = file.path.split('/').pop() ?? file.path
  if (name.startsWith('.') && !name.slice(1).includes('.')) return name.slice(1)
  return name.split('.').pop()?.toLowerCase() ?? 'text'
}

export function pluralize(count: number, singular: string, plural = `${singular}s`) {
  return `${count} ${count === 1 ? singular : plural}`
}

import { useEffect, useRef, useState } from 'react'
import type {
  KeyboardEvent as ReactKeyboardEvent,
  PointerEvent as ReactPointerEvent,
  RefObject,
} from 'react'
import {
  SIDEBAR_WIDTH_MAX,
  SIDEBAR_WIDTH_MIN,
  clampSidebarWidth,
} from '../lib/layout'

interface SidebarResizerProps {
  sidebarRef: RefObject<HTMLElement | null>
  label: string
  onCommit: (width: number) => void
  onReset: () => void
}

interface DragState {
  pointerId: number
  startX: number
  startWidth: number
  startInlineWidth: string
  width: number
}

export function SidebarResizer({
  sidebarRef,
  label,
  onCommit,
  onReset,
}: SidebarResizerProps) {
  const [dragging, setDragging] = useState(false)
  const drag = useRef<DragState | null>(null)

  useEffect(() => {
    return () => document.body.classList.remove('is-resizing-sidebar')
  }, [])

  const applyWidth = (width: number) => {
    const sidebar = sidebarRef.current
    if (sidebar) sidebar.style.width = `${width}px`
  }

  const beginDrag = (event: ReactPointerEvent<HTMLDivElement>) => {
    if (event.button !== 0) return
    const sidebar = sidebarRef.current
    if (!sidebar) return

    const startWidth = sidebar.getBoundingClientRect().width
    drag.current = {
      pointerId: event.pointerId,
      startX: event.clientX,
      startWidth,
      startInlineWidth: sidebar.style.width,
      width: startWidth,
    }
    event.currentTarget.setPointerCapture(event.pointerId)
    document.body.classList.add('is-resizing-sidebar')
    setDragging(true)
  }

  const moveDrag = (event: ReactPointerEvent<HTMLDivElement>) => {
    const state = drag.current
    if (!state || state.pointerId !== event.pointerId) return

    const width = clampSidebarWidth(
      state.startWidth + event.clientX - state.startX,
    )
    state.width = width
    applyWidth(width)
  }

  const endDrag = (event: ReactPointerEvent<HTMLDivElement>) => {
    const state = drag.current
    if (!state || state.pointerId !== event.pointerId) return

    drag.current = null
    if (event.currentTarget.hasPointerCapture(event.pointerId)) {
      event.currentTarget.releasePointerCapture(event.pointerId)
    }
    document.body.classList.remove('is-resizing-sidebar')
    setDragging(false)

    if (Math.round(state.width) === Math.round(state.startWidth)) {
      // A click, not a drag: restore whatever width was in effect before it.
      const sidebar = sidebarRef.current
      if (sidebar) {
        if (state.startInlineWidth) {
          sidebar.style.width = state.startInlineWidth
        } else {
          sidebar.style.removeProperty('width')
        }
      }
      return
    }
    onCommit(Math.round(state.width))
  }

  const nudge = (event: ReactKeyboardEvent<HTMLDivElement>) => {
    const sidebar = sidebarRef.current
    if (!sidebar) return

    if (event.key === 'Home' || event.key === 'End') {
      event.preventDefault()
      const width = clampSidebarWidth(
        event.key === 'Home' ? SIDEBAR_WIDTH_MIN : SIDEBAR_WIDTH_MAX,
      )
      applyWidth(width)
      onCommit(width)
      return
    }

    const direction =
      event.key === 'ArrowRight' ? 1 : event.key === 'ArrowLeft' ? -1 : 0
    if (direction === 0) return

    event.preventDefault()
    const current = sidebar.getBoundingClientRect().width
    const width = clampSidebarWidth(
      current + direction * (event.shiftKey ? 32 : 12),
    )
    if (width === Math.round(current)) return
    applyWidth(width)
    onCommit(width)
  }

  const resetWidth = () => {
    sidebarRef.current?.style.removeProperty('width')
    onReset()
  }

  return (
    <div
      className={`sidebar-resizer ${dragging ? 'is-dragging' : ''}`}
      role="separator"
      aria-orientation="vertical"
      aria-label={label}
      tabIndex={0}
      onPointerDown={beginDrag}
      onPointerMove={moveDrag}
      onPointerUp={endDrag}
      onPointerCancel={endDrag}
      onLostPointerCapture={() => {
        drag.current = null
        document.body.classList.remove('is-resizing-sidebar')
        setDragging(false)
      }}
      onKeyDown={nudge}
      onDoubleClick={resetWidth}
    />
  )
}

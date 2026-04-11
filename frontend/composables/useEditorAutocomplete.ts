import { ref, computed, watch, nextTick, onBeforeUnmount } from 'vue'
import { searchUnicodeEmojis } from '~/data/emojis'
import { useMentionAutocomplete } from '~/composables/useMentionAutocomplete'
import { useBoardMentions } from '~/composables/useBoardMentions'
import { useEmoji } from '~/composables/useEmoji'
import type { Board } from '~/types/generated'

export type TriggerType = 'emoji' | 'user' | 'board'

export interface AutocompleteSuggestion {
  type: TriggerType
  label: string
  // For emoji: the unicode character or custom emoji image URL
  // For user: the username
  // For board: the board name
  value: string
  icon?: string       // emoji character, avatar url, board icon url
  secondary?: string  // subtitle text (board title, etc.)
}

export interface AutocompleteState {
  active: boolean
  type: TriggerType | null
  query: string
  triggerStart: number  // character index where the trigger begins
  triggerEnd: number    // character index where the query ends
}

interface CaretCoords {
  top: number
  left: number
  height: number
}

export function useEditorAutocomplete () {
  const state = ref<AutocompleteState>({
    active: false,
    type: null,
    query: '',
    triggerStart: 0,
    triggerEnd: 0,
  })

  const suggestions = ref<AutocompleteSuggestion[]>([])
  const selectedIndex = ref(0)
  const dropdownPosition = ref<CaretCoords>({ top: 0, left: 0, height: 0 })

  const userAutocomplete = useMentionAutocomplete()
  const boardAutocomplete = useBoardMentions()
  const customEmoji = useEmoji()
  const currentBoard = useState<Board | null>('current-board', () => null)

  // Fetch custom emojis once on init (includes board-scoped emojis when in a board context)
  let customEmojisFetched = false

  function ensureCustomEmojis (): void {
    if (!customEmojisFetched) {
      customEmojisFetched = true
      const boardId = currentBoard.value?.id
      customEmoji.fetchAllAvailableEmojis(boardId)
    }
  }

  // Watch user autocomplete results
  watch(userAutocomplete.suggestions, (users) => {
    if (state.value.type !== 'user') return
    suggestions.value = users.map(name => ({
      type: 'user' as TriggerType,
      label: name,
      value: name,
      icon: undefined,
    }))
    selectedIndex.value = 0
  })

  // Watch board autocomplete results
  watch(boardAutocomplete.suggestions, (boards) => {
    if (state.value.type !== 'board') return
    suggestions.value = boards.map(b => ({
      type: 'board' as TriggerType,
      label: `b/${b.name}`,
      value: b.name,
      icon: b.icon ?? undefined,
      secondary: b.title,
    }))
    selectedIndex.value = 0
  })

  const isActive = computed(() => state.value.active && suggestions.value.length > 0)

  // Detect trigger patterns in text at cursor position.
  // Returns the trigger info or null if no trigger is active.
  function detectTrigger (text: string, cursorPos: number): { type: TriggerType, query: string, start: number } | null {
    if (cursorPos <= 0 || !text) return null

    const beforeCursor = text.substring(0, cursorPos)

    // Check for emoji trigger: :query (no closing colon yet)
    // Must be preceded by whitespace or start of text
    const emojiMatch = beforeCursor.match(/(?:^|[\s\n]):([\w+-]{1,32})$/)
    if (emojiMatch) {
      const query = emojiMatch[1]
      const start = cursorPos - query.length - 1 // -1 for the colon
      return { type: 'emoji', query, start }
    }

    // Check for user mention: @query
    const userMatch = beforeCursor.match(/(?:^|[\s\n])@([\w.-]{1,30})$/)
    if (userMatch) {
      const query = userMatch[1]
      const start = cursorPos - query.length - 1 // -1 for the @
      return { type: 'user', query, start }
    }

    // Check for board mention: b/query
    const boardMatch = beforeCursor.match(/(?:^|[\s\n])b\/([\w.-]{1,30})$/)
    if (boardMatch) {
      const query = boardMatch[1]
      const start = cursorPos - query.length - 2 // -2 for "b/"
      return { type: 'board', query, start }
    }

    return null
  }

  function activate (type: TriggerType, query: string, triggerStart: number, cursorPos: number): void {
    state.value = {
      active: true,
      type,
      query,
      triggerStart,
      triggerEnd: cursorPos,
    }

    // Trigger search based on type
    if (type === 'emoji') {
      ensureCustomEmojis()
      // Combine unicode and custom emoji results
      const unicode = searchUnicodeEmojis(query, 6)
      const custom = customEmoji.emojis.value
        .filter(e => e.shortcode.includes(query.toLowerCase()))
        .slice(0, 4)

      suggestions.value = [
        ...unicode.map(e => ({
          type: 'emoji' as TriggerType,
          label: `:${e.shortcode}:`,
          value: e.emoji,
          icon: e.emoji,
        })),
        ...custom.map(e => ({
          type: 'emoji' as TriggerType,
          label: `:${e.shortcode}:`,
          value: e.shortcode,
          icon: e.imageUrl,
          secondary: 'custom',
        })),
      ]
      selectedIndex.value = 0
    } else if (type === 'user') {
      userAutocomplete.search(query)
    } else if (type === 'board') {
      boardAutocomplete.search(query)
    }
  }

  function deactivate (): void {
    state.value = { active: false, type: null, query: '', triggerStart: 0, triggerEnd: 0 }
    suggestions.value = []
    selectedIndex.value = 0
    userAutocomplete.clear()
    boardAutocomplete.clear()
  }

  function moveUp (): void {
    if (suggestions.value.length === 0) return
    selectedIndex.value = (selectedIndex.value - 1 + suggestions.value.length) % suggestions.value.length
  }

  function moveDown (): void {
    if (suggestions.value.length === 0) return
    selectedIndex.value = (selectedIndex.value + 1) % suggestions.value.length
  }

  function getSelected (): AutocompleteSuggestion | null {
    if (suggestions.value.length === 0) return null
    return suggestions.value[selectedIndex.value] ?? null
  }

  // Build the replacement text for a selected suggestion
  function getReplacementText (suggestion: AutocompleteSuggestion): string {
    switch (suggestion.type) {
      case 'emoji':
        if (suggestion.secondary === 'custom') {
          return `:${suggestion.value}: `
        }
        return `${suggestion.value} `
      case 'user':
        return `@${suggestion.value} `
      case 'board':
        return `b/${suggestion.value} `
      default:
        return suggestion.value
    }
  }

  // Get caret pixel coordinates for a textarea element
  function getTextareaCaretCoords (textarea: HTMLTextAreaElement, position: number): CaretCoords {
    const mirror = document.createElement('div')
    const style = window.getComputedStyle(textarea)

    // Copy textarea styles to mirror
    const properties = [
      'fontFamily', 'fontSize', 'fontWeight', 'fontStyle', 'letterSpacing',
      'textTransform', 'wordSpacing', 'textIndent', 'lineHeight',
      'paddingTop', 'paddingRight', 'paddingBottom', 'paddingLeft',
      'borderTopWidth', 'borderRightWidth', 'borderBottomWidth', 'borderLeftWidth',
      'boxSizing', 'whiteSpace', 'wordWrap', 'overflowWrap', 'tabSize',
    ] as const

    mirror.style.position = 'absolute'
    mirror.style.visibility = 'hidden'
    mirror.style.overflow = 'hidden'
    mirror.style.width = style.width

    for (const prop of properties) {
      mirror.style[prop as any] = style[prop as any]
    }
    mirror.style.whiteSpace = 'pre-wrap'
    mirror.style.wordWrap = 'break-word'

    const text = textarea.value.substring(0, position)
    const textNode = document.createTextNode(text)
    mirror.appendChild(textNode)

    const marker = document.createElement('span')
    marker.textContent = '|'
    mirror.appendChild(marker)

    document.body.appendChild(mirror)

    const textareaRect = textarea.getBoundingClientRect()
    const markerRect = marker.getBoundingClientRect()
    const mirrorRect = mirror.getBoundingClientRect()

    const coords: CaretCoords = {
      top: textareaRect.top + (markerRect.top - mirrorRect.top) - textarea.scrollTop,
      left: textareaRect.left + (markerRect.left - mirrorRect.left),
      height: markerRect.height,
    }

    document.body.removeChild(mirror)
    return coords
  }

  // Get caret pixel coordinates for a contenteditable element
  function getContentEditableCaretCoords (): CaretCoords | null {
    const selection = window.getSelection()
    if (!selection || selection.rangeCount === 0) return null

    const range = selection.getRangeAt(0).cloneRange()
    range.collapse(true)

    // Insert a zero-width space to get coordinates, then remove it
    const span = document.createElement('span')
    span.textContent = '\u200B'
    range.insertNode(span)

    const rect = span.getBoundingClientRect()
    const coords: CaretCoords = {
      top: rect.top,
      left: rect.left,
      height: rect.height || 16,
    }

    span.parentNode?.removeChild(span)

    // Restore selection
    selection.removeAllRanges()
    selection.addRange(range)

    return coords
  }

  // Process input for a textarea-based editor
  function handleTextareaInput (textarea: HTMLTextAreaElement): void {
    const text = textarea.value
    const cursorPos = textarea.selectionStart

    const trigger = detectTrigger(text, cursorPos)

    if (trigger) {
      const coords = getTextareaCaretCoords(textarea, cursorPos)
      dropdownPosition.value = coords
      activate(trigger.type, trigger.query, trigger.start, cursorPos)
    } else {
      deactivate()
    }
  }

  // Process input for a contenteditable editor.
  // Returns the text content and cursor offset within the current text node.
  function handleContentEditableInput (editorEl: HTMLElement): void {
    const selection = window.getSelection()
    if (!selection || selection.rangeCount === 0) {
      deactivate()
      return
    }

    const range = selection.getRangeAt(0)
    if (!range.collapsed) {
      deactivate()
      return
    }

    // Get text content up to cursor in the current text node
    const node = range.startContainer
    if (node.nodeType !== Node.TEXT_NODE) {
      deactivate()
      return
    }

    const text = node.textContent ?? ''
    const cursorPos = range.startOffset

    const trigger = detectTrigger(text, cursorPos)

    if (trigger) {
      const coords = getContentEditableCaretCoords()
      if (coords) {
        dropdownPosition.value = coords
      }
      activate(trigger.type, trigger.query, trigger.start, cursorPos)
    } else {
      deactivate()
    }
  }

  // Apply a selected suggestion in a textarea
  function applyToTextarea (
    suggestion: AutocompleteSuggestion,
    textarea: HTMLTextAreaElement,
    content: { value: string },
  ): void {
    const text = content.value
    const replacement = getReplacementText(suggestion)
    const { triggerStart, triggerEnd } = state.value

    content.value = text.substring(0, triggerStart) + replacement + text.substring(triggerEnd)

    const newCursorPos = triggerStart + replacement.length
    deactivate()

    // Restore cursor position after Vue updates the textarea
    nextTick(() => {
      textarea.focus()
      textarea.setSelectionRange(newCursorPos, newCursorPos)
    })
  }

  // Apply a selected suggestion in a contenteditable editor
  function applyToContentEditable (
    suggestion: AutocompleteSuggestion,
    editorEl: HTMLElement,
    emitUpdate: () => void,
  ): void {
    const selection = window.getSelection()
    if (!selection || selection.rangeCount === 0) return

    const range = selection.getRangeAt(0)
    const node = range.startContainer
    if (node.nodeType !== Node.TEXT_NODE) return

    const text = node.textContent ?? ''
    const replacement = getReplacementText(suggestion)
    const { triggerStart, triggerEnd } = state.value

    node.textContent = text.substring(0, triggerStart) + replacement + text.substring(triggerEnd)

    // Set cursor position after the replacement
    const newOffset = triggerStart + replacement.length
    const newRange = document.createRange()
    newRange.setStart(node, Math.min(newOffset, node.textContent.length))
    newRange.collapse(true)
    selection.removeAllRanges()
    selection.addRange(newRange)

    deactivate()
    emitUpdate()
  }

  // Apply a selected suggestion using a TipTap editor instance.
  // The editor parameter should have chain().focus().insertContent() etc.
  function applyToTipTap (
    suggestion: AutocompleteSuggestion,
    tiptapEditor: { view: { state: { selection: { from: number } }; dom: HTMLElement }; chain: () => any },
  ): void {
    const selection = window.getSelection()
    if (!selection || selection.rangeCount === 0) return

    const range = selection.getRangeAt(0)
    const node = range.startContainer
    if (node.nodeType !== Node.TEXT_NODE) return

    const text = node.textContent ?? ''
    const replacement = getReplacementText(suggestion)
    const { triggerStart, triggerEnd } = state.value

    // Replace the trigger text with the suggestion
    node.textContent = text.substring(0, triggerStart) + replacement + text.substring(triggerEnd)

    // Set cursor position
    const newOffset = triggerStart + replacement.length
    const newRange = document.createRange()
    newRange.setStart(node, Math.min(newOffset, node.textContent.length))
    newRange.collapse(true)
    selection.removeAllRanges()
    selection.addRange(newRange)

    // Force TipTap to sync with DOM changes
    tiptapEditor.view.dom.dispatchEvent(new Event('input', { bubbles: true }))

    deactivate()
  }

  // Handle keyboard events — returns true if the event was consumed
  function handleKeyDown (e: KeyboardEvent): boolean {
    if (!isActive.value) return false

    switch (e.key) {
      case 'ArrowUp':
        e.preventDefault()
        moveUp()
        return true
      case 'ArrowDown':
        e.preventDefault()
        moveDown()
        return true
      case 'Enter':
      case 'Tab':
        e.preventDefault()
        return true // caller should apply the selection
      case 'Escape':
        e.preventDefault()
        deactivate()
        return true
      default:
        return false
    }
  }

  onBeforeUnmount(() => {
    deactivate()
  })

  return {
    state,
    suggestions,
    selectedIndex,
    dropdownPosition,
    isActive,

    handleTextareaInput,
    handleContentEditableInput,
    handleKeyDown,
    applyToTextarea,
    applyToContentEditable,
    applyToTipTap,
    getSelected,
    deactivate,
  }
}

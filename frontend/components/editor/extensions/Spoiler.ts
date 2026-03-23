import { Node, mergeAttributes } from '@tiptap/core'

export interface SpoilerOptions {
  HTMLAttributes: Record<string, unknown>
}

declare module '@tiptap/core' {
  interface Commands<ReturnType> {
    spoiler: {
      setSpoiler: () => ReturnType
      toggleSpoiler: () => ReturnType
    }
  }
}

/**
 * TipTap node wrapping <details>/<summary> for collapsible spoiler blocks.
 *
 * Schema:
 *   <details class="spoiler-block">
 *     <summary>Spoiler</summary>
 *     <div class="spoiler-content">...block content...</div>
 *   </details>
 */
const SpoilerSummary = Node.create({
  name: 'spoilerSummary',
  group: 'block',
  content: 'inline*',
  defining: true,
  selectable: false,
  isolating: true,

  parseHTML () {
    return [{ tag: 'summary' }]
  },

  renderHTML ({ HTMLAttributes }) {
    return ['summary', mergeAttributes(HTMLAttributes, { class: 'spoiler-summary' }), 0]
  },
})

const SpoilerContent = Node.create({
  name: 'spoilerContent',
  group: 'block',
  content: 'block+',
  defining: true,

  parseHTML () {
    return [
      { tag: 'div.spoiler-content' },
    ]
  },

  renderHTML ({ HTMLAttributes }) {
    return ['div', mergeAttributes(HTMLAttributes, { class: 'spoiler-content' }), 0]
  },
})

const SpoilerBlock = Node.create<SpoilerOptions>({
  name: 'spoilerBlock',
  group: 'block',
  content: 'spoilerSummary spoilerContent',
  defining: true,

  addOptions () {
    return { HTMLAttributes: {} }
  },

  parseHTML () {
    return [
      { tag: 'details' },
      { tag: 'details.spoiler-block' },
    ]
  },

  renderHTML ({ HTMLAttributes }) {
    return ['details', mergeAttributes(this.options.HTMLAttributes, HTMLAttributes, { class: 'spoiler-block', open: true }), 0]
  },

  addCommands () {
    return {
      setSpoiler: () => ({ chain }) => {
        return chain()
          .insertContent({
            type: 'spoilerBlock',
            content: [
              {
                type: 'spoilerSummary',
                content: [{ type: 'text', text: 'Spoiler' }],
              },
              {
                type: 'spoilerContent',
                content: [
                  {
                    type: 'paragraph',
                    content: [{ type: 'text', text: 'Hidden content here' }],
                  },
                ],
              },
            ],
          })
          .run()
      },
      toggleSpoiler: () => ({ chain }) => {
        return chain()
          .insertContent({
            type: 'spoilerBlock',
            content: [
              {
                type: 'spoilerSummary',
                content: [{ type: 'text', text: 'Spoiler' }],
              },
              {
                type: 'spoilerContent',
                content: [
                  {
                    type: 'paragraph',
                    content: [{ type: 'text', text: 'Hidden content here' }],
                  },
                ],
              },
            ],
          })
          .run()
      },
    }
  },
})

export { SpoilerBlock, SpoilerSummary, SpoilerContent }
export default SpoilerBlock

import { Node, mergeAttributes } from '@tiptap/core'

export interface ForumQuoteOptions {
  HTMLAttributes: Record<string, unknown>
}

declare module '@tiptap/core' {
  interface Commands<ReturnType> {
    forumQuote: {
      setForumQuote: (attrs: { author: string; postNumber: number }) => ReturnType
      unsetForumQuote: () => ReturnType
    }
  }
}

export const ForumQuote = Node.create<ForumQuoteOptions>({
  name: 'forumQuote',
  group: 'block',
  content: 'block+',
  defining: true,

  addOptions () {
    return {
      HTMLAttributes: {},
    }
  },

  addAttributes () {
    return {
      author: {
        default: '',
        parseHTML: element => element.getAttribute('data-author') ?? '',
        renderHTML: attrs => ({ 'data-author': attrs.author }),
      },
      postNumber: {
        default: 0,
        parseHTML: element => parseInt(element.getAttribute('data-post-number') ?? '0', 10),
        renderHTML: attrs => ({ 'data-post-number': attrs.postNumber }),
      },
    }
  },

  parseHTML () {
    return [
      {
        tag: 'blockquote.forum-quote',
      },
      {
        tag: 'blockquote[data-author]',
      },
    ]
  },

  renderHTML ({ HTMLAttributes }) {
    const author = HTMLAttributes['data-author'] ?? ''
    const postNumber = HTMLAttributes['data-post-number'] ?? 0

    return [
      'blockquote',
      mergeAttributes(this.options.HTMLAttributes, HTMLAttributes, { class: 'forum-quote' }),
      [
        'div',
        { class: 'forum-quote-header', contenteditable: 'false' },
        ['a', { href: `#post-${postNumber}`, class: 'forum-quote-author' }, `@${author}`],
        ' said:',
      ],
      ['div', { class: 'forum-quote-body' }, 0],
    ]
  },

  addCommands () {
    return {
      setForumQuote: (attrs) => ({ commands }) => {
        return commands.wrapIn(this.name, attrs)
      },
      unsetForumQuote: () => ({ commands }) => {
        return commands.lift(this.name)
      },
    }
  },
})

export default ForumQuote

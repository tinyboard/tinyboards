/**
 * Client-side CSS validation — mirrors the backend sanitizer rules.
 * Used for instant feedback in the CSS editor before submitting.
 */

export interface CssValidationResult {
  valid: boolean
  errors: string[]
  warnings: string[]
}

const DANGEROUS_PATTERNS: Array<{ pattern: RegExp; message: string }> = [
  { pattern: /expression\s*\(/i, message: 'CSS expressions are not allowed (XSS risk)' },
  { pattern: /javascript\s*:/i, message: 'JavaScript protocol is not allowed' },
  { pattern: /vbscript\s*:/i, message: 'VBScript protocol is not allowed' },
  { pattern: /behavior\s*:/i, message: 'IE behavior property is not allowed' },
  { pattern: /-moz-binding\s*:/i, message: 'Mozilla binding property is not allowed' },
  { pattern: /@import/i, message: '@import is not allowed — all styles must be inline' },
  { pattern: /@charset/i, message: '@charset is not allowed' },
  { pattern: /@namespace/i, message: '@namespace is not allowed' },
  { pattern: /url\s*\(/i, message: 'url() is not allowed — external resources cannot be loaded' },
  { pattern: /<\s*\/?(?:script|style|link|meta|iframe)/i, message: 'HTML tags are not allowed in CSS' },
]

const BLOCKED_VALUES: Array<{ pattern: RegExp; message: string }> = [
  { pattern: /position\s*:\s*(fixed|sticky)/i, message: 'position: fixed/sticky is not allowed — it can overlay navigation' },
  { pattern: /z-index\s*:\s*\d{5,}/i, message: 'Very high z-index values (5+ digits) are not allowed' },
]

export function validateCss (css: string, maxBytes: number = 50 * 1024): CssValidationResult {
  const errors: string[] = []
  const warnings: string[] = []

  if (!css.trim()) {
    return { valid: true, errors, warnings }
  }

  if (new Blob([css]).size > maxBytes) {
    errors.push(`CSS exceeds maximum size of ${Math.round(maxBytes / 1024)} KB`)
  }

  for (const { pattern, message } of DANGEROUS_PATTERNS) {
    if (pattern.test(css)) {
      errors.push(message)
    }
  }

  for (const { pattern, message } of BLOCKED_VALUES) {
    if (pattern.test(css)) {
      warnings.push(message)
    }
  }

  // Check balanced braces
  const open = (css.match(/\{/g) || []).length
  const close = (css.match(/\}/g) || []).length
  if (open !== close) {
    errors.push(`Unbalanced braces: ${open} opening vs ${close} closing`)
  }

  return { valid: errors.length === 0, errors, warnings }
}

/**
 * CSS snippet presets that users can insert with one click.
 * Grouped by category for the wizard UI.
 */
export interface CssSnippet {
  name: string
  description: string
  css: string
}

export interface CssSnippetCategory {
  name: string
  icon: string
  snippets: CssSnippet[]
}

export const CSS_SNIPPET_CATEGORIES: CssSnippetCategory[] = [
  {
    name: 'Cards & Posts',
    icon: 'rectangle-stack',
    snippets: [
      {
        name: 'Rounded post cards',
        description: 'Add rounded corners and subtle shadow to post cards',
        css: `.post-card {
  border-radius: 12px;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.08);
  overflow: hidden;
}`,
      },
      {
        name: 'Card hover effect',
        description: 'Lift cards slightly on hover',
        css: `.post-card {
  transition: transform 0.15s ease, box-shadow 0.15s ease;
}
.post-card:hover {
  transform: translateY(-2px);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
}`,
      },
      {
        name: 'Bordered cards',
        description: 'Add a colored left border to post cards',
        css: `.post-card {
  border-left: 3px solid rgb(var(--color-primary));
  border-radius: 0 8px 8px 0;
}`,
      },
    ],
  },
  {
    name: 'Typography',
    icon: 'font',
    snippets: [
      {
        name: 'Larger post titles',
        description: 'Make post titles more prominent',
        css: `.post-title {
  font-size: 1.25rem;
  font-weight: 700;
  letter-spacing: -0.01em;
}`,
      },
      {
        name: 'Serif headings',
        description: 'Use serif font for headings (elegant look)',
        css: `h1, h2, h3, .post-title {
  font-family: Georgia, 'Times New Roman', serif;
}`,
      },
      {
        name: 'Compact text',
        description: 'Tighter line spacing for denser content',
        css: `.post-body, .comment-body {
  line-height: 1.5;
  font-size: 0.9rem;
}`,
      },
    ],
  },
  {
    name: 'Colors & Backgrounds',
    icon: 'palette',
    snippets: [
      {
        name: 'Subtle page background',
        description: 'Add a slight color tint to the page background',
        css: `body {
  background-color: #f8f9fa;
}`,
      },
      {
        name: 'Dark sidebar',
        description: 'Give the sidebar a dark background',
        css: `.sidebar {
  background-color: #1f2937;
  color: #e5e7eb;
  border-radius: 12px;
  padding: 1rem;
}
.sidebar a { color: #93c5fd; }
.sidebar a:hover { color: #bfdbfe; }`,
      },
      {
        name: 'Gradient header',
        description: 'Add a gradient to the site header',
        css: `.app-header {
  background: linear-gradient(135deg, rgb(var(--color-primary)), rgb(var(--color-primary-hover)));
}`,
      },
    ],
  },
  {
    name: 'Layout & Spacing',
    icon: 'layout',
    snippets: [
      {
        name: 'Wider content area',
        description: 'Increase the max width of the main content',
        css: `.max-w-8xl {
  max-width: 96rem;
}`,
      },
      {
        name: 'Compact spacing',
        description: 'Reduce spacing between posts',
        css: `.post-list > * + * {
  margin-top: 0.5rem;
}`,
      },
      {
        name: 'Hide sidebar on all pages',
        description: 'Remove the sidebar entirely',
        css: `.sidebar-container {
  display: none;
}
main {
  max-width: 100%;
}`,
      },
    ],
  },
  {
    name: 'Buttons & Links',
    icon: 'cursor-click',
    snippets: [
      {
        name: 'Pill-shaped buttons',
        description: 'Make all buttons fully rounded',
        css: `.button, button[type="submit"] {
  border-radius: 9999px;
}`,
      },
      {
        name: 'Underline links on hover',
        description: 'Add underline effect to links on hover',
        css: `a:not(.button):not(.no-underline) {
  text-decoration: none;
  transition: text-decoration 0.15s;
}
a:not(.button):not(.no-underline):hover {
  text-decoration: underline;
}`,
      },
    ],
  },
  {
    name: 'Comments',
    icon: 'chat-bubble',
    snippets: [
      {
        name: 'Colorful thread lines',
        description: 'Color-code nested comment thread lines',
        css: `.comment-thread-line { border-left-color: rgb(var(--color-primary) / 0.3); }
.comment-depth-1 .comment-thread-line { border-left-color: #3b82f6; }
.comment-depth-2 .comment-thread-line { border-left-color: #8b5cf6; }
.comment-depth-3 .comment-thread-line { border-left-color: #ec4899; }
.comment-depth-4 .comment-thread-line { border-left-color: #f59e0b; }
.comment-depth-5 .comment-thread-line { border-left-color: #10b981; }`,
      },
      {
        name: 'Rounded comment bubbles',
        description: 'Style comments as chat-like bubbles',
        css: `.comment-body {
  background: #f3f4f6;
  border-radius: 12px;
  padding: 0.75rem 1rem;
  margin-top: 0.25rem;
}`,
      },
    ],
  },
  {
    name: 'Animations',
    icon: 'sparkles',
    snippets: [
      {
        name: 'Fade-in posts',
        description: 'Posts fade in when they appear',
        css: `@keyframes fadeIn {
  from { opacity: 0; transform: translateY(8px); }
  to { opacity: 1; transform: translateY(0); }
}
.post-card {
  animation: fadeIn 0.3s ease forwards;
}`,
      },
      {
        name: 'Smooth vote animation',
        description: 'Add a pop effect when voting',
        css: `.vote-button:active {
  transform: scale(1.3);
  transition: transform 0.1s ease;
}
.vote-button {
  transition: transform 0.15s ease;
}`,
      },
    ],
  },
  {
    name: 'Dark Mode Tweaks',
    icon: 'moon',
    snippets: [
      {
        name: 'Custom dark mode colors',
        description: 'Adjust dark mode background and text colors',
        css: `body.dark {
  background-color: #0f172a;
  color: #e2e8f0;
}
body.dark .post-card {
  background-color: #1e293b;
  border-color: #334155;
}`,
      },
      {
        name: 'OLED dark mode',
        description: 'True black background for OLED screens',
        css: `body.dark {
  background-color: #000000;
}
body.dark .post-card,
body.dark .sidebar {
  background-color: #111111;
  border-color: #222222;
}`,
      },
    ],
  },
]

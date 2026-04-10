import type { Config } from 'tailwindcss'
import colors from 'tailwindcss/colors'
import forms from '@tailwindcss/forms'
import typography from '@tailwindcss/typography'

export default {
  darkMode: 'class',
  content: [
    './components/**/*.{vue,ts}',
    './layouts/**/*.vue',
    './pages/**/*.vue',
    './plugins/**/*.ts',
    './app.vue',
  ],
  safelist: ['lite-youtube'],
  theme: {
    colors: {
      primary: 'rgb(var(--color-primary) / <alpha-value>)',
      'primary-hover': 'rgb(var(--color-primary-hover) / <alpha-value>)',
      secondary: 'rgb(var(--color-secondary) / <alpha-value>)',
      transparent: 'transparent',
      current: 'currentColor',
      black: colors.black,
      white: colors.white,
      gray: colors.zinc,
      red: colors.red,
      blue: colors.sky,
      yellow: colors.yellow,
      orange: colors.orange,
      green: colors.green,
      pink: colors.pink,
    },
    fontFamily: {
      sans: ['Mona Sans', 'Helvetica Neue', 'Helvetica', 'Arial', 'sans-serif'],
      serif: ['Georgia'],
      mono: ['ui-monospace', 'SFMono-Regular', 'menlo'],
    },
    extend: {
      borderRadius: {
        md: '6px',
      },
      boxShadow: {
        xs: '0 1px 2px rgba(0, 0, 0, 0.07)',
        'inner-xs': 'inset 0 1px 2px rgba(0,0,0,.03)',
        'inner-white': 'inset 0 1px 0 rgba(255,255,255,.4)',
      },
      fontSize: {
        xs: '11px',
        sm: ['12px', '16px'],
        base: ['14px', '20px'],
        lg: ['16px', '24px'],
      },
      maxWidth: {
        '8xl': '96rem',
      },
      typography: (theme: (path: string) => string) => ({
        DEFAULT: {
          css: {
            a: {
              fontWeight: 'normal',
              color: theme('colors.secondary'),
              textDecoration: 'none',
            },
            blockquote: {
              borderLeftWidth: '2px',
              fontStyle: 'normal',
              fontWeight: 'normal',
            },
            img: {
              display: 'inline-block',
            },
          },
        },
      }),
    },
  },
  plugins: [forms, typography],
} satisfies Config

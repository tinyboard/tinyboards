import { useSiteStore } from '~/stores/site'
import { useUIStore } from '~/stores/ui'
import type { ThemeMode } from '~/stores/ui'

/**
 * Client-side plugin that:
 * 1. Restores user's theme preference from localStorage
 * 2. Applies custom theme colors from site config to CSS custom properties
 * 3. Persists theme changes to localStorage
 */
export default defineNuxtPlugin(() => {
  const siteStore = useSiteStore()
  const uiStore = useUIStore()

  // Restore theme from localStorage
  const savedTheme = localStorage.getItem('tb_theme') as ThemeMode | null
  if (savedTheme) {
    uiStore.setTheme(savedTheme)
  } else if (siteStore.defaultTheme && siteStore.defaultTheme !== 'light') {
    uiStore.setTheme(siteStore.defaultTheme as ThemeMode)
  }

  // Persist theme changes to localStorage
  watch(() => uiStore.theme, (newTheme) => {
    localStorage.setItem('tb_theme', newTheme)
  })

  // Custom color support
  function hexToRgb (hex: string): string | null {
    const cleaned = hex.replace('#', '')
    if (cleaned.length !== 6 && cleaned.length !== 3) { return null }
    const full = cleaned.length === 3
      ? cleaned[0] + cleaned[0] + cleaned[1] + cleaned[1] + cleaned[2] + cleaned[2]
      : cleaned
    const num = parseInt(full, 16)
    if (isNaN(num)) { return null }
    return `${(num >> 16) & 255} ${(num >> 8) & 255} ${num & 255}`
  }

  function applyThemeColors () {
    const root = document.documentElement
    if (siteStore.primaryColor) {
      const rgb = hexToRgb(siteStore.primaryColor)
      if (rgb) { root.style.setProperty('--color-primary', rgb) }
    }
    if (siteStore.secondaryColor) {
      const rgb = hexToRgb(siteStore.secondaryColor)
      if (rgb) { root.style.setProperty('--color-secondary', rgb) }
    }
    if (siteStore.hoverColor) {
      const rgb = hexToRgb(siteStore.hoverColor)
      if (rgb) { root.style.setProperty('--color-primary-hover', rgb) }
    }
  }

  // Apply on load and watch for changes
  watch(() => [siteStore.primaryColor, siteStore.secondaryColor, siteStore.hoverColor], applyThemeColors, { immediate: true })
})

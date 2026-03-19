import { defineStore } from 'pinia'
import { ref } from 'vue'

export type ThemeMode = 'light' | 'dark' | 'ocean' | 'forest' | 'sunset' | 'purple'

export interface Toast {
  id: string
  type: 'success' | 'error' | 'info' | 'warning'
  message: string
  duration: number
}

/**
 * UI store — global UI state: theme, sidebar, modals, toasts.
 */
export type PostViewMode = 'expanded' | 'compact'

export const useUIStore = defineStore('ui', () => {
  const theme = ref<ThemeMode>('light')
  const sidebarOpen = ref(false)
  const postViewMode = ref<PostViewMode>('expanded')
  const activeModal = ref<string | null>(null)
  const modalData = ref<unknown>(null)
  const toasts = ref<Toast[]>([])

  function setTheme (newTheme: ThemeMode): void {
    theme.value = newTheme

    // Apply theme class to document body
    if (import.meta.client) {
      const body = document.body
      // Remove all theme classes
      body.classList.remove('light', 'dark', 'ocean', 'forest', 'sunset', 'purple')
      // Add new theme class (light is default, no class needed)
      if (newTheme !== 'light') {
        body.classList.add(newTheme)
      }
    }
  }

  function toggleSidebar (): void {
    sidebarOpen.value = !sidebarOpen.value
  }

  function setPostViewMode (mode: PostViewMode): void {
    postViewMode.value = mode
  }

  function openModal (name: string, data?: unknown): void {
    activeModal.value = name
    modalData.value = data ?? null
  }

  function closeModal (): void {
    activeModal.value = null
    modalData.value = null
  }

  function addToast (toast: Omit<Toast, 'id'>): void {
    const id = Date.now().toString(36) + Math.random().toString(36).slice(2, 7)
    const entry: Toast = { ...toast, id }
    toasts.value.push(entry)

    if (toast.duration > 0) {
      setTimeout(() => removeToast(id), toast.duration)
    }
  }

  function removeToast (id: string): void {
    toasts.value = toasts.value.filter(t => t.id !== id)
  }

  return {
    theme,
    sidebarOpen,
    postViewMode,
    activeModal,
    modalData,
    toasts,
    setTheme,
    toggleSidebar,
    setPostViewMode,
    openModal,
    closeModal,
    addToast,
    removeToast,
  }
})

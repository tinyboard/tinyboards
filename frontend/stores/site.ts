import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { LocalSite } from '~/types/generated'

/**
 * Site store — holds global site configuration.
 * Loaded on every navigation via middleware/site.global.ts.
 */
export const useSiteStore = defineStore('site', () => {
  const site = ref<LocalSite | null>(null)
  const loaded = ref(false)

  const name = computed(() => site.value?.name ?? 'tinyboards')
  const description = computed(() => site.value?.description ?? '')
  const icon = computed(() => site.value?.icon ?? null)
  const welcomeMessage = computed(() => site.value?.welcomeMessage ?? null)

  // Registration mode: 'open' | 'invite_only' | 'application' | 'email_verification' | 'closed'
  const registrationMode = computed<string>(() => {
    if (!site.value) { return 'closed' }
    return site.value.registrationMode ?? 'closed'
  })

  // Feature flags
  const enableDownvotes = computed(() => site.value?.enableDownvotes ?? true)
  const enableNSFW = computed(() => site.value?.enableNSFW ?? false)
  const captchaEnabled = computed(() => site.value?.captchaEnabled ?? false)
  const boardCreationAdminOnly = computed(() => site.value?.boardCreationAdminOnly ?? false)
  const requireEmailVerification = computed(() => site.value?.requireEmailVerification ?? false)
  const isPrivate = computed(() => site.value?.isPrivate ?? false)

  // Theme colors
  const primaryColor = computed(() => site.value?.primaryColor ?? null)
  const secondaryColor = computed(() => site.value?.secondaryColor ?? null)
  const hoverColor = computed(() => site.value?.hoverColor ?? null)
  const defaultTheme = computed(() => site.value?.defaultTheme ?? 'light')

  function setSite (newSite: LocalSite): void {
    site.value = newSite
    loaded.value = true
  }

  function clearSite (): void {
    site.value = null
    loaded.value = false
  }

  return {
    site,
    loaded,
    name,
    description,
    icon,
    welcomeMessage,
    registrationMode,
    enableDownvotes,
    enableNSFW,
    captchaEnabled,
    boardCreationAdminOnly,
    requireEmailVerification,
    isPrivate,
    primaryColor,
    secondaryColor,
    hoverColor,
    defaultTheme,
    setSite,
    clearSite,
  }
})

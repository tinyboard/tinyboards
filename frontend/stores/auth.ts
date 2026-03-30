import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { User } from '~/types/generated'

interface SubscribedBoard {
  name: string
  title: string
  icon: string | null
  subscribers: number
  mode?: string
}

/**
 * Auth store — holds current user session state.
 * Populated server-side via middleware, never reads cookies client-side.
 */
export const useAuthStore = defineStore('auth', () => {
  const user = ref<User | null>(null)
  const subscribedBoards = ref<SubscribedBoard[]>([])
  const unreadNotificationCount = ref(0)

  const isLoggedIn = computed(() => user.value !== null)
  const isAdmin = computed(() => (user.value?.adminLevel ?? 0) > 0)
  const isBanned = computed(() => user.value?.isBanned ?? false)

  function setUser (newUser: User): void {
    user.value = newUser
  }

  function clearUser (): void {
    user.value = null
    subscribedBoards.value = []
    unreadNotificationCount.value = 0
  }

  function setUnreadNotificationCount (count: number): void {
    unreadNotificationCount.value = count
  }

  function updateUser (updates: Partial<User>): void {
    if (user.value) {
      user.value = { ...user.value, ...updates }
    }
  }

  function setSubscribedBoards (boards: SubscribedBoard[]): void {
    subscribedBoards.value = boards
  }

  return {
    user,
    subscribedBoards,
    unreadNotificationCount,
    isLoggedIn,
    isAdmin,
    isBanned,
    setUser,
    clearUser,
    updateUser,
    setSubscribedBoards,
    setUnreadNotificationCount,
  }
})

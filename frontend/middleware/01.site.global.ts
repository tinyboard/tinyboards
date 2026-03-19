import { useSiteStore } from '~/stores/site'
import { useGraphQL } from '~/composables/useGraphQL'
import type { LocalSite } from '~/types/generated'

const SITE_QUERY = `
  query GetSite {
    site {
      id
      name
      description
      icon
      primaryColor
      secondaryColor
      hoverColor
      enableDownvotes
      enableNSFW
      boardCreationAdminOnly
      requireEmailVerification
      applicationQuestion
      isPrivate
      registrationMode
      defaultTheme
      defaultPostListingType
      defaultAvatar
      captchaEnabled
      captchaDifficulty
      boardsEnabled
      boardCreationMode
      allowedPostTypes
      enableNSFWTagging
      wordFilterEnabled
      welcomeMessage
    }
  }
`

interface SiteResponse {
  site: LocalSite
}

/**
 * Global middleware that loads site configuration into the store on every navigation.
 * Runs first (01) so other middleware can depend on site config.
 */
export default defineNuxtRouteMiddleware(async () => {
  const siteStore = useSiteStore()

  // Only fetch once per SSR request or on first client navigation
  if (siteStore.loaded) { return }

  const { execute, error } = useGraphQL<SiteResponse>()
  const result = await execute(SITE_QUERY)

  if (!error.value && result?.site) {
    siteStore.setSite(result.site)
  }
})

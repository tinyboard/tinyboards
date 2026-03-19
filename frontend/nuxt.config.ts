import { readFileSync, existsSync } from 'node:fs'
import { defineNuxtConfig } from 'nuxt/config'
import hjson from 'hjson'

// Load the shared tinyboards.hjson configuration.
// This is the single source of truth for the entire stack.
const configPath = process.env.TB_CONFIG_LOCATION || '/app/config/defaults.hjson'
const fallbackPath = './tinyboards.hjson'

let cfg: Record<string, any> = {}
const resolvedPath = existsSync(configPath) ? configPath : existsSync(fallbackPath) ? fallbackPath : null
if (resolvedPath) {
  cfg = hjson.parse(readFileSync(resolvedPath, 'utf-8'))
} else {
  console.warn(`[config] tinyboards.hjson not found at ${configPath} or ${fallbackPath} — using env vars / defaults`)
}

const hostname = cfg.hostname || process.env.NUXT_PUBLIC_DOMAIN || 'localhost'
const useHttps = cfg.frontend?.use_https ?? (process.env.NUXT_PUBLIC_USE_HTTPS === 'true')
const backendPort = cfg.port || 8536
const internalApiHost = cfg.frontend?.internal_api_host
  || process.env.NUXT_INTERNAL_API_HOST
  || `http://localhost:${backendPort}`

export default defineNuxtConfig({
  devtools: { enabled: true },

  ssr: true,

  devServer: {
    port: 3000,
    host: 'localhost',
  },

  app: {
    head: {
      link: [
        {
          rel: 'preload',
          href: '/fonts/Mona-Sans.woff2',
          as: 'font',
          type: 'font/woff2',
          crossorigin: 'anonymous',
        },
      ],
    },
    pageTransition: { name: 'page', mode: 'out-in' },
    layoutTransition: { name: 'layout', mode: 'out-in' },
  },

  css: ['~/assets/css/main.css'],

  modules: [
    '@nuxtjs/tailwindcss',
    '@pinia/nuxt',
  ],

  routeRules: {
    '/': { redirect: '/home' },
  },

  runtimeConfig: {
    // Server-side only — derived from tinyboards.hjson
    backendUrl: internalApiHost,
    internalGqlHost: `${internalApiHost}/api/v2/graphql`,
    internalApiHost,

    public: {
      domain: hostname,
      useHttps,
    },
  },

  compatibilityDate: '2025-01-01',

  vite: {
    server: {
      allowedHosts: ['localhost', 'tinyboards.test', hostname],
    },
  },
})

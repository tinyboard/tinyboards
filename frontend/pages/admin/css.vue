<script setup lang="ts">
import { useGraphQL, useGraphQLMutation } from '~/composables/useGraphQL'
import { useToast } from '~/composables/useToast'
import { validateCss, CSS_SNIPPET_CATEGORIES } from '~/utils/css-validator'
import type { CssSnippet } from '~/utils/css-validator'

definePageMeta({ layout: 'admin' })
useHead({ title: 'Admin - Custom CSS' })

const toast = useToast()

// ---------------------------------------------------------------------------
// Data fetching
// ---------------------------------------------------------------------------
const SITE_CSS_QUERY = `
  query GetSiteCss {
    site {
      customCss
      customCssEnabled
    }
  }
`

const UPDATE_CSS_MUTATION = `
  mutation UpdateSiteCSS($input: UpdateSiteConfigInput!) {
    updateSiteConfig(input: $input) {
      customCss
      customCssEnabled
    }
  }
`

const { execute, loading } = useGraphQL<{ site: { customCss: string | null; customCssEnabled: boolean } }>()
const { execute: executeMutation, loading: saving } = useGraphQLMutation()

// ---------------------------------------------------------------------------
// State
// ---------------------------------------------------------------------------
const cssCode = ref('')
const cssEnabled = ref(false)
const activeTab = ref<'editor' | 'wizard' | 'preview'>('wizard')
const expandedCategory = ref<string | null>(null)
const previewCss = ref('')
const showPreview = ref(false)

// Validation
const validation = computed(() => validateCss(cssCode.value))
const charCount = computed(() => new Blob([cssCode.value]).size)
const maxBytes = 50 * 1024

// ---------------------------------------------------------------------------
// Load current CSS
// ---------------------------------------------------------------------------
onMounted(async () => {
  const result = await execute(SITE_CSS_QUERY)
  if (result?.site) {
    cssCode.value = result.site.customCss ?? ''
    cssEnabled.value = result.site.customCssEnabled
  }
})

// ---------------------------------------------------------------------------
// Actions
// ---------------------------------------------------------------------------
function insertSnippet (snippet: CssSnippet) {
  if (cssCode.value && !cssCode.value.endsWith('\n')) {
    cssCode.value += '\n'
  }
  cssCode.value += `\n/* ${snippet.name} */\n${snippet.css}\n`
  activeTab.value = 'editor'
  toast.success(`Inserted: ${snippet.name}`)
}

function toggleCategory (name: string) {
  expandedCategory.value = expandedCategory.value === name ? null : name
}

function updatePreview () {
  previewCss.value = cssCode.value
  showPreview.value = true
  activeTab.value = 'preview'
}

function clearCss () {
  cssCode.value = ''
  toast.info('CSS cleared')
}

async function saveCss () {
  if (!validation.value.valid) {
    toast.error('Please fix CSS errors before saving')
    return
  }

  const result = await executeMutation(UPDATE_CSS_MUTATION, {
    variables: {
      input: {
        customCss: cssCode.value || '',
        customCssEnabled: cssEnabled.value,
      },
    },
  })

  if (result) {
    toast.success('Custom CSS saved successfully')
    // Reload site store so changes take effect immediately
    const siteStore = useSiteStore()
    if (siteStore.site) {
      siteStore.site.customCss = cssCode.value || null
      siteStore.site.customCssEnabled = cssEnabled.value
    }
  }
}

// Category icon mapping
function getCategoryIcon (icon: string): string {
  const icons: Record<string, string> = {
    'rectangle-stack': '\u25A1',
    'font': 'Aa',
    'palette': '\u25CE',
    'layout': '\u2B1C',
    'cursor-click': '\u25B6',
    'chat-bubble': '\u25AC',
    'sparkles': '\u2728',
    'moon': '\u25D0',
  }
  return icons[icon] || '\u2022'
}
</script>

<template>
  <div class="max-w-5xl">
    <!-- Header -->
    <div class="flex items-center justify-between mb-6">
      <div>
        <h2 class="text-lg font-semibold text-gray-900">
          Custom CSS
        </h2>
        <p class="text-sm text-gray-500 mt-1">
          Add custom styles to your site. Board moderators can also add board-level CSS that overrides these styles.
        </p>
      </div>

      <!-- Enable toggle -->
      <label class="flex items-center gap-3 cursor-pointer select-none">
        <span class="text-sm font-medium" :class="cssEnabled ? 'text-green-700' : 'text-gray-500'">
          {{ cssEnabled ? 'Enabled' : 'Disabled' }}
        </span>
        <button
          type="button"
          role="switch"
          :aria-checked="cssEnabled"
          :class="cssEnabled ? 'bg-green-500' : 'bg-gray-300'"
          class="relative inline-flex h-6 w-11 shrink-0 rounded-full transition-colors duration-200 ease-in-out focus:outline-none focus:ring-2 focus:ring-primary focus:ring-offset-2"
          @click="cssEnabled = !cssEnabled"
        >
          <span
            :class="cssEnabled ? 'translate-x-5' : 'translate-x-0'"
            class="pointer-events-none inline-block h-5 w-5 translate-y-0.5 transform rounded-full bg-white shadow ring-0 transition duration-200 ease-in-out ml-0.5"
          />
        </button>
      </label>
    </div>

    <CommonLoadingSpinner v-if="loading" size="lg" />

    <template v-else>
      <!-- Safety notice -->
      <div class="rounded-lg bg-blue-50 border border-blue-200 p-4 mb-6">
        <div class="flex gap-3">
          <div class="text-blue-500 text-lg shrink-0">&#9432;</div>
          <div class="text-sm text-blue-800">
            <p class="font-medium mb-1">Safety Information</p>
            <ul class="list-disc ml-4 space-y-0.5 text-blue-700">
              <li>CSS is sanitized on the server — dangerous patterns are automatically blocked</li>
              <li><code class="text-xs bg-blue-100 rounded px-1">@import</code>, <code class="text-xs bg-blue-100 rounded px-1">url()</code>, <code class="text-xs bg-blue-100 rounded px-1">expression()</code>, and <code class="text-xs bg-blue-100 rounded px-1">position: fixed</code> are not allowed</li>
              <li>Maximum size: 50 KB. Use the wizard tab for quick, safe customizations</li>
              <li><code class="text-xs bg-blue-100 rounded px-1">@media</code> queries and <code class="text-xs bg-blue-100 rounded px-1">@keyframes</code> animations are allowed</li>
            </ul>
          </div>
        </div>
      </div>

      <!-- Tabs -->
      <div class="flex gap-1 border-b border-gray-200 mb-4">
        <button
          v-for="tab in [
            { id: 'wizard', label: 'Style Wizard' },
            { id: 'editor', label: 'CSS Editor' },
            { id: 'preview', label: 'Live Preview' },
          ]"
          :key="tab.id"
          :class="activeTab === tab.id
            ? 'border-primary text-primary'
            : 'border-transparent text-gray-500 hover:text-gray-700'"
          class="px-4 py-2 text-sm font-medium border-b-2 transition-colors"
          @click="activeTab = tab.id as 'editor' | 'wizard' | 'preview'"
        >
          {{ tab.label }}
        </button>
      </div>

      <!-- ============================================ -->
      <!-- WIZARD TAB -->
      <!-- ============================================ -->
      <div v-if="activeTab === 'wizard'" class="space-y-3">
        <p class="text-sm text-gray-600 mb-4">
          Click a snippet to add it to your CSS. You can then customize it in the editor tab.
        </p>

        <div
          v-for="category in CSS_SNIPPET_CATEGORIES"
          :key="category.name"
          class="border border-gray-200 rounded-lg overflow-hidden"
        >
          <!-- Category header -->
          <button
            class="w-full flex items-center gap-3 px-4 py-3 text-left hover:bg-gray-50 transition-colors"
            @click="toggleCategory(category.name)"
          >
            <span class="text-lg w-6 text-center opacity-70">{{ getCategoryIcon(category.icon) }}</span>
            <span class="font-medium text-sm text-gray-900 flex-1">{{ category.name }}</span>
            <span class="text-xs text-gray-400">{{ category.snippets.length }} snippets</span>
            <span
              class="text-gray-400 transition-transform duration-200"
              :class="expandedCategory === category.name ? 'rotate-90' : ''"
            >&#9656;</span>
          </button>

          <!-- Snippets -->
          <div v-if="expandedCategory === category.name" class="border-t border-gray-100 divide-y divide-gray-100">
            <div
              v-for="snippet in category.snippets"
              :key="snippet.name"
              class="p-4 hover:bg-gray-50 transition-colors"
            >
              <div class="flex items-start justify-between gap-4">
                <div class="flex-1 min-w-0">
                  <div class="font-medium text-sm text-gray-900">{{ snippet.name }}</div>
                  <div class="text-xs text-gray-500 mt-0.5">{{ snippet.description }}</div>
                  <pre class="mt-2 text-xs bg-gray-900 text-green-400 rounded-md p-3 overflow-x-auto font-mono leading-relaxed"><code>{{ snippet.css }}</code></pre>
                </div>
                <button
                  class="shrink-0 button primary button-sm"
                  @click="insertSnippet(snippet)"
                >
                  Insert
                </button>
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- ============================================ -->
      <!-- EDITOR TAB -->
      <!-- ============================================ -->
      <div v-if="activeTab === 'editor'" class="space-y-4">
        <!-- Toolbar -->
        <div class="flex items-center justify-between">
          <div class="flex items-center gap-2">
            <button class="button white button-sm" @click="updatePreview">
              Preview
            </button>
            <button class="button white button-sm text-red-600" :disabled="!cssCode" @click="clearCss">
              Clear All
            </button>
          </div>
          <div class="text-xs text-gray-400">
            {{ (charCount / 1024).toFixed(1) }} KB / {{ maxBytes / 1024 }} KB
          </div>
        </div>

        <!-- Editor -->
        <div class="relative">
          <textarea
            v-model="cssCode"
            class="w-full h-96 font-mono text-sm bg-gray-900 text-green-400 rounded-lg p-4 border border-gray-700 focus:border-primary focus:ring-1 focus:ring-primary resize-y leading-relaxed"
            placeholder="/* Write your custom CSS here */

.post-card {
  border-radius: 12px;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.08);
}"
            spellcheck="false"
            autocomplete="off"
            autocorrect="off"
            autocapitalize="off"
          />
          <!-- Line count indicator -->
          <div class="absolute bottom-3 right-3 text-xs text-gray-500 bg-gray-800 rounded px-2 py-1">
            {{ cssCode.split('\n').length }} lines
          </div>
        </div>

        <!-- Validation feedback -->
        <div v-if="cssCode && !validation.valid" class="rounded-lg bg-red-50 border border-red-200 p-3">
          <div class="text-sm font-medium text-red-800 mb-1">Validation Errors</div>
          <ul class="list-disc ml-4 text-sm text-red-700 space-y-0.5">
            <li v-for="(err, i) in validation.errors" :key="i">{{ err }}</li>
          </ul>
        </div>

        <div v-if="cssCode && validation.warnings.length > 0" class="rounded-lg bg-yellow-50 border border-yellow-200 p-3">
          <div class="text-sm font-medium text-yellow-800 mb-1">Warnings</div>
          <ul class="list-disc ml-4 text-sm text-yellow-700 space-y-0.5">
            <li v-for="(warn, i) in validation.warnings" :key="i">{{ warn }}</li>
          </ul>
        </div>

        <div v-if="cssCode && validation.valid && validation.warnings.length === 0" class="flex items-center gap-2 text-sm text-green-700">
          <span>&#10003;</span> CSS is valid
        </div>
      </div>

      <!-- ============================================ -->
      <!-- PREVIEW TAB -->
      <!-- ============================================ -->
      <div v-if="activeTab === 'preview'" class="space-y-4">
        <div class="flex items-center justify-between">
          <p class="text-sm text-gray-500">
            This preview shows how your custom CSS will affect the page. Changes are temporary until you save.
          </p>
          <button class="button white button-sm" @click="updatePreview">
            Refresh Preview
          </button>
        </div>

        <!-- Preview frame -->
        <div class="border border-gray-200 rounded-lg overflow-hidden">
          <!-- Mini header preview -->
          <div class="px-4 py-3 bg-primary text-white text-sm font-medium flex items-center gap-2 preview-header">
            <span class="w-6 h-6 rounded bg-white/20" />
            <span>Site Header Preview</span>
            <span class="ml-auto flex gap-2">
              <span class="text-xs opacity-70">Home</span>
              <span class="text-xs opacity-70">Boards</span>
              <span class="text-xs opacity-70">Search</span>
            </span>
          </div>

          <!-- Mini content preview -->
          <div class="p-4 bg-gray-100 preview-body" style="min-height: 300px">
            <div class="max-w-2xl mx-auto space-y-3">
              <!-- Post card preview -->
              <div class="post-card bg-white rounded-lg border border-gray-200 p-4">
                <div class="flex gap-3">
                  <div class="flex flex-col items-center gap-1">
                    <button class="vote-button text-gray-400 hover:text-primary text-lg">&#9650;</button>
                    <span class="text-sm font-medium text-gray-700">42</span>
                    <button class="vote-button text-gray-400 hover:text-red-500 text-lg">&#9660;</button>
                  </div>
                  <div class="flex-1">
                    <div class="post-title text-base font-semibold text-gray-900">Example Post Title</div>
                    <div class="text-xs text-gray-500 mt-1">Posted by <span class="text-primary">u/example</span> in <span class="font-medium">b/general</span> &middot; 2h ago</div>
                    <div class="post-body text-sm text-gray-700 mt-2">This is a preview of how posts will look with your custom CSS applied. The styles you write will affect the real page.</div>
                  </div>
                </div>
              </div>

              <!-- Comment preview -->
              <div class="bg-white rounded-lg border border-gray-200 p-4">
                <div class="text-xs text-gray-500 mb-2">Comments</div>
                <div class="comment-body text-sm text-gray-700 pl-3 border-l-2 border-primary/30 comment-thread-line">
                  <div class="text-xs text-gray-500 mb-1"><span class="text-primary font-medium">u/commenter</span> &middot; 1h ago</div>
                  This is an example comment to preview your CSS changes.
                </div>
              </div>

              <!-- Sidebar preview -->
              <div class="sidebar bg-white rounded-lg border border-gray-200 p-4">
                <div class="text-sm font-medium text-gray-900 mb-2">Sidebar</div>
                <div class="text-xs text-gray-500">
                  <a href="#" class="text-primary hover:underline">Board Rules</a> &middot;
                  <a href="#" class="text-primary hover:underline">Wiki</a> &middot;
                  <a href="#" class="text-primary hover:underline">Moderators</a>
                </div>
              </div>
            </div>
          </div>
        </div>

        <!-- Inject preview CSS -->
        <Teleport to="head">
          <component :is="'style'" v-if="showPreview" id="tb-css-preview">
            {{ previewCss }}
          </component>
        </Teleport>
      </div>

      <!-- ============================================ -->
      <!-- SAVE BAR -->
      <!-- ============================================ -->
      <div class="flex items-center justify-between pt-6 mt-6 border-t border-gray-200">
        <div class="text-sm text-gray-500">
          <template v-if="cssEnabled && cssCode">
            CSS will be injected into every page for all visitors.
          </template>
          <template v-else-if="!cssEnabled && cssCode">
            CSS is saved but disabled. Enable it with the toggle above.
          </template>
          <template v-else>
            No custom CSS configured.
          </template>
        </div>
        <button
          class="button primary"
          :disabled="saving || (!validation.valid && !!cssCode)"
          @click="saveCss"
        >
          {{ saving ? 'Saving...' : 'Save Custom CSS' }}
        </button>
      </div>

      <!-- ============================================ -->
      <!-- HELP / CASCADE EXPLANATION -->
      <!-- ============================================ -->
      <div class="mt-8 rounded-lg bg-gray-50 border border-gray-200 p-5">
        <h3 class="text-sm font-semibold text-gray-900 mb-3">How CSS Cascading Works</h3>
        <div class="text-sm text-gray-600 space-y-2">
          <p>Custom CSS is applied in layers, with later layers overriding earlier ones:</p>
          <ol class="list-decimal ml-5 space-y-1">
            <li><strong>Default Theme</strong> &mdash; The built-in theme (light, dark, ocean, etc.)</li>
            <li><strong>Site Custom CSS</strong> (this page) &mdash; Applied globally on every page</li>
            <li><strong>Board Custom CSS</strong> &mdash; Applied only when viewing that specific board, overrides site CSS</li>
          </ol>
          <p class="text-gray-500 mt-3">
            Board moderators with the Appearance permission can add custom CSS for their board via Board Settings &rarr; Appearance.
          </p>
        </div>
      </div>
    </template>
  </div>
</template>

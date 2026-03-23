<script setup lang="ts">
import { ref, computed } from 'vue'
import { NodeViewContent, NodeViewWrapper, nodeViewProps } from '@tiptap/vue-3'

const props = defineProps(nodeViewProps)

const showDropdown = ref(false)

const languages = computed(() => {
  // Prioritized list of common languages
  const priority = [
    'javascript', 'typescript', 'python', 'rust', 'html', 'css',
    'json', 'bash', 'sql', 'go', 'java', 'c', 'cpp', 'csharp',
    'php', 'ruby', 'swift', 'kotlin', 'lua', 'yaml', 'xml',
    'markdown', 'plaintext',
  ]

  try {
    const lowlight = (props.extension?.options as any)?.lowlight
    if (lowlight?.listLanguages) {
      const all: string[] = lowlight.listLanguages()
      const sorted = [...priority.filter(l => all.includes(l))]
      const rest = all.filter((l: string) => !priority.includes(l)).sort()
      return [...sorted, ...rest]
    }
    return priority
  } catch {
    return priority
  }
})

const selectedLanguage = computed(() => props.node?.attrs?.language || 'plaintext')

function selectLanguage (lang: string): void {
  props.updateAttributes({ language: lang })
  showDropdown.value = false
}
</script>

<template>
  <NodeViewWrapper class="code-block-wrapper">
    <div class="code-block-header" contenteditable="false">
      <div class="language-select-wrapper">
        <button
          type="button"
          class="language-select-btn"
          @click="showDropdown = !showDropdown"
        >
          {{ selectedLanguage }}
          <svg class="w-3 h-3 ml-1" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" />
          </svg>
        </button>

        <div v-if="showDropdown" class="language-dropdown">
          <button
            v-for="lang in languages"
            :key="lang"
            type="button"
            class="language-option"
            :class="{ active: lang === selectedLanguage }"
            @click="selectLanguage(lang)"
          >
            {{ lang }}
          </button>
        </div>
      </div>
    </div>
    <pre><NodeViewContent as="code" /></pre>
  </NodeViewWrapper>
</template>

<style scoped>
.code-block-wrapper {
  position: relative;
  margin: 8px 0;
}

.code-block-header {
  display: flex;
  align-items: center;
  justify-content: flex-end;
  background: #1f2937;
  padding: 4px 8px;
  border-radius: 6px 6px 0 0;
}

.code-block-wrapper pre {
  background: #1f2937;
  color: #e5e7eb;
  padding: 12px 16px;
  border-radius: 0 0 6px 6px;
  overflow-x: auto;
  margin: 0;
  font-size: 13px;
  line-height: 1.5;
}

.code-block-wrapper pre code {
  background: none;
  padding: 0;
  color: inherit;
  font-family: 'Fira Code', 'JetBrains Mono', 'Cascadia Code', Menlo, Monaco, 'Courier New', monospace;
}

.language-select-wrapper {
  position: relative;
}

.language-select-btn {
  display: inline-flex;
  align-items: center;
  padding: 2px 8px;
  font-size: 11px;
  color: #9ca3af;
  background: #374151;
  border-radius: 4px;
  cursor: pointer;
  transition: color 0.15s;
}

.language-select-btn:hover {
  color: #e5e7eb;
}

.language-dropdown {
  position: absolute;
  top: 100%;
  right: 0;
  margin-top: 4px;
  width: 160px;
  max-height: 200px;
  overflow-y: auto;
  background: #1f2937;
  border: 1px solid #374151;
  border-radius: 6px;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
  z-index: 50;
}

.language-option {
  display: block;
  width: 100%;
  padding: 4px 12px;
  font-size: 12px;
  color: #d1d5db;
  text-align: left;
  cursor: pointer;
  transition: background-color 0.1s;
}

.language-option:hover {
  background-color: #374151;
  color: #f9fafb;
}

.language-option.active {
  background-color: rgb(var(--color-primary, 99 102 241) / 0.3);
  color: #f9fafb;
}
</style>

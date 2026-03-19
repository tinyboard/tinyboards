module.exports = {
  root: true,
  extends: [
    '@nuxtjs/eslint-config-typescript',
  ],
  rules: {
    // Enforce no unguarded console statements (BUG-026, BUG-039)
    'no-console': 'warn',

    // TypeScript strict mode support
    '@typescript-eslint/no-explicit-any': 'error',
    '@typescript-eslint/no-unused-vars': ['error', { argsIgnorePattern: '^_' }],

    // Vue best practices
    'vue/multi-word-component-names': 'off',
    'vue/no-v-html': 'off', // We enforce DOMPurify usage instead

    // Allow type references before definition (common in TypeScript)
    'no-use-before-define': 'off',
    '@typescript-eslint/no-use-before-define': ['error', { functions: false, typedefs: false }],

    // General code quality
    'comma-dangle': ['error', 'always-multiline'],
    'semi': ['error', 'never'],
  },
}

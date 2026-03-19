import type { CodegenConfig } from '@graphql-codegen/cli'

export default {
  schema: '../schema.graphql',
  documents: [
    'composables/**/*.ts',
    'pages/**/*.vue',
    'components/**/*.vue',
    'server/**/*.ts',
  ],
  generates: {
    './types/generated.ts': {
      plugins: ['typescript', 'typescript-operations', 'typed-document-node'],
      config: {
        strictScalars: true,
        scalars: {
          DateTime: 'string',
          UUID: 'string',
          Upload: 'File',
          JSON: 'Record<string, unknown>',
        },
        enumsAsTypes: true,
        skipTypename: false,
        avoidOptionals: false,
        maybeValue: 'T | null | undefined',
        inputMaybeValue: 'T | null | undefined',
        nonOptionalTypename: false,
      },
    },
  },
  ignoreNoDocuments: true,
} satisfies CodegenConfig

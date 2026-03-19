# Stage 1: Install dependencies
FROM node:20-alpine AS deps

WORKDIR /build

COPY frontend/package.json ./
RUN npm install

# Stage 2: Run codegen (schema.graphql from repo root, no network calls)
FROM deps AS codegen

WORKDIR /build

# Copy schema.graphql from the repo root into the build context
COPY schema.graphql /app/schema.graphql

# Copy frontend source
COPY frontend/ .

# codegen.ts references ../schema.graphql, so place it one level up
RUN cp /app/schema.graphql ../schema.graphql

RUN npx graphql-codegen --config codegen.ts

# Stage 3: Build the Nuxt application (types already generated)
FROM codegen AS builder

RUN npx nuxi build

# Stage 4: Minimal runtime image
FROM node:20-alpine

RUN apk add --no-cache curl

# Run as non-root
RUN addgroup -g 1001 tinyboards \
    && adduser -u 1001 -G tinyboards -s /sbin/nologin -D tinyboards

WORKDIR /app

# Copy the built Nuxt output
COPY --from=builder /build/.output ./.output

RUN chown -R tinyboards:tinyboards /app

USER tinyboards

ENV NODE_ENV=production
ENV HOST=0.0.0.0
ENV PORT=3000

EXPOSE 3000

HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 \
    CMD curl -sf http://localhost:3000/ || exit 1

CMD ["node", ".output/server/index.mjs"]

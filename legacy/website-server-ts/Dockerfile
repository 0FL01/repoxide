# ==============================================================================
# Base image
# ==============================================================================
FROM node:24-alpine AS builder

WORKDIR /app
RUN chown node:node /app
USER node
COPY --chown=node:node package*.json ./

# Install all dependencies (including dev dependencies for build)
RUN npm ci

# Copy source code
COPY --chown=node:node . .

# Build TypeScript
RUN npm run build

# ==============================================================================
# Production dependencies
# ==============================================================================
FROM node:24-alpine AS deps

WORKDIR /app
RUN chown node:node /app
USER node
COPY --chown=node:node package*.json ./

# Install only production dependencies
RUN npm ci --only=production --ignore-scripts && \
    npm cache clean --force

# ==============================================================================
# Runtime image
# ==============================================================================
FROM node:24-alpine

# Install git and ca-certificates (required by repomix for remote repository processing)
RUN apk add --no-cache git ca-certificates

WORKDIR /app

# Copy built application
COPY --from=builder --chown=node:node /app/dist ./dist

# Copy production dependencies
COPY --from=deps --chown=node:node /app/node_modules ./node_modules

# Set environment variables
ENV NODE_ENV=production \
    PORT=8080 \
    NODE_OPTIONS="--max-old-space-size=3072"

# Expose port
EXPOSE 8080

# Start the server directly
USER node
CMD ["node", "dist/index.js"]

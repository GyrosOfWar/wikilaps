FROM ghcr.io/pnpm/pnpm:11 AS builder
RUN pnpm runtime set node 24 -g

WORKDIR /app
COPY package.json .
COPY pnpm-*.yaml .
RUN pnpm install --frozen-lockfile
COPY . .
ARG APP_VERSION=dev
ENV APP_VERSION=$APP_VERSION
RUN pnpm build
RUN pnpm prune --production

FROM node:lts-alpine
WORKDIR /app
COPY --from=builder /app/build build/
COPY --from=builder /app/node_modules node_modules/
COPY --from=builder /app/handler.js .
COPY package.json .

EXPOSE 3000

ENV NODE_ENV=production

CMD [ "node", "handler.js" ]

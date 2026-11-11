# syntax=docker/dockerfile:1
# Release image: static musl binary with the dashboard embedded, on scratch.

FROM node:26.12.1-alpine@sha256:40cd011b55415bc894e45b02fdd5a5dfde056921b7d23a64c74f80497af48797 AS web
WORKDIR /web
COPY web/package.json web/package-lock.json ./
RUN npm ci --no-audit --no-fund
COPY web/ ./
# The words (shared with the server): web/src/lib/i18n.svelte.js reads ../../../locales.
COPY locales/ /locales/
RUN npm run build

FROM rust:1.99.1-alpine@sha256:7e9e2d068a107a2adafd19930011178d14a78326053cd4c3a9f7ad3fd049557d AS build
RUN apk add --no-cache musl-dev
WORKDIR /src
ARG TARGETARCH
# Generic x86-64 baseline: the NAS does not have AVX or SSE4.2.
ENV CARGO_BUILD_JOBS=2
COPY . .
COPY --from=web /web/dist web/dist
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/src/target \
    if [ "$TARGETARCH" = "amd64" ]; then export RUSTFLAGS="-C target-cpu=x86-64" CFLAGS="-march=x86-64 -mtune=generic"; fi; \
    cargo build --release --locked -p moli-os && cp target/release/moli-os /moli-os

FROM scratch
COPY --from=build /moli-os /moli-os
ENV MOLI_CONFIG=/data/moli.toml
WORKDIR /data
USER 1000:1000
EXPOSE 8790
ENTRYPOINT ["/moli-os"]
CMD ["serve"]

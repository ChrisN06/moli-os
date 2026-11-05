# syntax=docker/dockerfile:1
# Same relay runtime, with the hlc7 startup SPS correction only.
FROM --platform=$BUILDPLATFORM golang:1.27.3-alpine@sha256:d200d2dd76b03a99b52651fd1325aff7dcd35422c35c01bac56571b0507f3deb AS build
RUN apk add --no-cache patch
WORKDIR /src
ADD --checksum=sha256:10027a1558746e02c9adc22ceea8ee5ad29a19e156d3dded23404103f0e7c636 https://codeload.github.com/AlexxIT/go2rtc/tar.gz/7c05066c90ab6ad1fccb506d5f0408aa31ebe60f /tmp/go2rtc.tar.gz
RUN tar -xzf /tmp/go2rtc.tar.gz --strip-components=1
COPY docker/go2rtc/hlc7-hd.patch /tmp/hlc7-hd.patch
COPY docker/go2rtc/producer_test.go pkg/xiaomi/miss/producer_test.go
RUN patch -p1 < /tmp/hlc7-hd.patch
RUN --mount=type=cache,target=/go/pkg/mod --mount=type=cache,target=/root/.cache/go-build \
    GOMAXPROCS=2 go test ./pkg/xiaomi/miss ./pkg/h265
ARG TARGETOS=linux
ARG TARGETARCH=amd64
RUN --mount=type=cache,target=/go/pkg/mod --mount=type=cache,target=/root/.cache/go-build \
    CGO_ENABLED=0 GOOS=$TARGETOS GOARCH=$TARGETARCH GOAMD64=v1 GOMAXPROCS=2 \
    go build -trimpath -ldflags '-s -w' -o /out/go2rtc .

# Preserve tools, entrypoint, and configuration behavior of the pinned upstream runtime.
FROM alexxit/go2rtc@sha256:233f25720b22bc808717085d6618425099dccd4713ecfc923e2dc627519b59fb
COPY --from=build /out/go2rtc /usr/local/bin/go2rtc
COPY docker/go2rtc/LICENSE /usr/local/share/licenses/go2rtc/LICENSE

# Xiaomi Mi Camera 2K: wait for HD before publishing HEVC metadata

The `isa.camera.hlc7` can send a 640 × 360 SPS immediately after an HD request,
then send 2304 × 1296 video. go2rtc 1.9.15 caches the first SPS during probing;
its HLS initialization and the subsequent pictures can therefore disagree.
A native HLS decoder may display only the upper-left part of the picture.

This narrowly scoped patch waits for an SPS larger than 640 × 360 **only** for
`isa.camera.hlc7` with subtype omitted, `hd`, or `2`. It uses the existing
15-second probe deadline. The first HD keyframe consumed by probing is retained
and delivered when streaming starts, so HLS does not have to wait for another GOP.
Explicit SD/auto streams and other models retain their
behavior. There is no transcoding, recording, motor command, or new network port.

The patch applies to upstream commit `7c05066c90ab6ad1fccb506d5f0408aa31ebe60f`
(v1.9.15). Synthetic SPS tests cover SD rejection, HD acceptance, missing metadata,
explicit SD/auto, an unrelated camera, and an SD-to-HD startup sequence whose
first HD keyframe must remain available, with and without audio. The upstream MIT license is included.

Build from the Moli repository root:

```sh
docker build --platform linux/amd64 -f docker/go2rtc/Dockerfile -t moli-go2rtc:hlc7-hd .
```

The build downloads checksum-pinned upstream source, applies the patch, runs its
focused tests, and replaces only the executable in the pinned upstream runtime.
AMD64 uses `GOAMD64=v1`; it does not target the build machine's CPU.

Deploy the resulting immutable image digest in place of the existing go2rtc image,
with the existing command, volumes, tmpfs, ports, and network settings unchanged.
Restarting the shared relay interrupts all active camera streams briefly. Record
the previous digest first; rolling back changes only that image reference.
Never run an additional relay against the live configuration for testing.

After deployment, open the Xiaomi camera in Safari on Mac and iPhone, including
a fresh connection after the previous viewer closes. Confirm the full picture,
HD dimensions, and both other camera streams. Unit tests and a cross-build do
not replace this real-device acceptance check.

Sources: [upstream camera issue](https://github.com/AlexxIT/go2rtc/issues/1982),
[probe implementation](https://github.com/AlexxIT/go2rtc/blob/7c05066c90ab6ad1fccb506d5f0408aa31ebe60f/pkg/xiaomi/miss/producer.go).

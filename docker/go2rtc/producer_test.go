package miss

import (
	"encoding/base64"
	"github.com/AlexxIT/go2rtc/pkg/bits"
	"github.com/AlexxIT/go2rtc/pkg/core"
	"testing"
)

func testSPS(width, height uint32) *core.Codec {
	w := bits.NewWriter([]byte{0x42, 1})
	w.WriteBits8(1, 8) // no sub-layers
	w.WriteBits8(1, 8) // main profile
	w.WriteBits(0, 32)
	w.WriteBits(0, 32)
	w.WriteBits16(0, 16)
	w.WriteBits8(120, 8)
	ue := func(value uint32) {
		value++
		var n byte
		for x := value; x > 1; x >>= 1 {
			n++
		}
		w.WriteAllBits(0, n)
		w.WriteBits(value, n+1)
	}
	ue(0)
	ue(1)
	ue(width)
	ue(height)
	w.WriteBit(1)
	return &core.Codec{FmtpLine: "sprop-sps=" + base64.StdEncoding.EncodeToString(w.Bytes())}
}

func TestHlc7WaitsForRequestedHD(t *testing.T) {
	sd, hd := testSPS(640, 360), testSPS(2304, 1296)
	for _, quality := range []string{"", "hd", "2"} {
		if readyH265("isa.camera.hlc7", quality, sd) {
			t.Fatal("startup SD accepted for HD")
		}
		if !readyH265("isa.camera.hlc7", quality, hd) {
			t.Fatal("HD rejected")
		}
		if readyH265("isa.camera.hlc7", quality, &core.Codec{}) {
			t.Fatal("missing SPS accepted")
		}
	}
	for _, quality := range []string{"sd", "1", "auto", "0"} {
		if !readyH265("isa.camera.hlc7", quality, sd) {
			t.Fatal("explicit SD/auto changed")
		}
	}
	if !readyH265("unrelated.camera", "hd", sd) {
		t.Fatal("unrelated camera changed")
	}
}

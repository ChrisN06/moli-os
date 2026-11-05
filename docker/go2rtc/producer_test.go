package miss

import (
	"encoding/base64"
	"errors"
	"github.com/AlexxIT/go2rtc/pkg/bits"
	"github.com/AlexxIT/go2rtc/pkg/core"
	"github.com/AlexxIT/go2rtc/pkg/h264"
	"github.com/AlexxIT/go2rtc/pkg/h264/annexb"
	"github.com/AlexxIT/go2rtc/pkg/h265"
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

func TestHDProbeKeepsTheFirstKeyframeForHLS(t *testing.T) {
	packet := func(width, height uint32) *Packet {
		_, sps, _ := h265.GetParameterSet(testSPS(width, height).FmtpLine)
		// Emulation prevention keeps synthetic RBSP zeros from becoming Annex B separators.
		escaped := make([]byte, 0, len(sps)+8)
		zeros := 0
		for _, b := range sps {
			if zeros == 2 && b <= 3 {
				escaped = append(escaped, 3)
				zeros = 0
			}
			escaped = append(escaped, b)
			if b == 0 {
				zeros++
			} else {
				zeros = 0
			}
		}
		data := h264.JoinNALU([]byte{0x40, 1, 1}, escaped, []byte{0x44, 1, 1}, []byte{0x26, 1, 1})
		return &Packet{CodecID: codecH265, Payload: annexb.DecodeAVCC(data, false)}
	}
	sd, hd := packet(640, 360), packet(2304, 1296)
	for _, audio := range []bool{false, true} {
		list := []*Packet{sd, hd}
		if audio {
			list = append(list, &Packet{CodecID: codecPCMA})
		}
		calls := 0
		medias, first, err := probe(func() (*Packet, error) {
			if calls == len(list) {
				return nil, errors.New("next GOP is not available")
			}
			p := list[calls]
			calls++
			return p, nil
		}, "isa.camera.hlc7", audio, "hd")
		if err != nil {
			t.Fatal(err)
		}
		if first != hd {
			t.Fatal("first HD keyframe was discarded during probing")
		}
		if calls != len(list) {
			t.Fatal("probe consumed beyond the startup packets")
		}
		_, sps, _ := h265.GetParameterSet(medias[0].Codecs[0].FmtpLine)
		if h265.DecodeSPS(sps).Width() != 2304 {
			t.Fatal("HLS would still announce SD dimensions")
		}
	}
}

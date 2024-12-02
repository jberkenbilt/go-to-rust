package device_test

import (
	"github.com/jberkenbilt/go-to-rust-blog/go/device"
	"testing"
)

func TestAll(t *testing.T) {
	// This is a duplication of the controller test using the wrapper
	// API.
	_, err := device.Two("quack")
	if err == nil || err.Error() != "call Init first" {
		t.Fatalf("wrong error: %s", err)
	}
	device.Init()
	checkErr := func(e error) {
		t.Helper()
		if e != nil {
			t.Fatalf("unexpected error: %s", e)
		}
	}
	seq, err := device.One(5)
	checkErr(err)
	if seq != 1 {
		t.Fatalf("wrong result: %v", seq)
	}
	_, err = device.One(3)
	if err == nil || err.Error() != "sorry, not that one" {
		t.Fatalf("wrong error: %s", err)
	}
	path, err := device.Two("potato")
	checkErr(err)
	if path != "two?val=potato&seq=2" {
		t.Fatalf("wrong result: %v", path)
	}
}

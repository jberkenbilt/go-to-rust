package controller_test

import (
	"github.com/jberkenbilt/go-to-rust-blog/go/internal/controller"
	"testing"
)

func TestAll(t *testing.T) {
	c := controller.New()
	checkErr := func(e error) {
		t.Helper()
		if e != nil {
			t.Fatalf("unexpected error: %s", e)
		}
	}
	seq, err := c.One(5)
	checkErr(err)
	if seq != 1 {
		t.Fatalf("wrong result: %v", seq)
	}
	_, err = c.One(3)
	if err == nil || err.Error() != "sorry, not that one" {
		t.Fatalf("wrong error: %s", err)
	}
	path, err := c.Two("potato")
	checkErr(err)
	if path != "two?val=potato&seq=2" {
		t.Fatalf("wrong result: %v", path)
	}
}

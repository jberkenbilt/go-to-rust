// Package controller is an internal implementation of sample API. The
// implementation pretends to make network calls and accesses locked
// data. It is wrapped by a function-based API that operates a
// singleton.
package controller

import (
	"errors"
	"fmt"
	"sync"
)

type Controller struct {
	lock     sync.RWMutex
	seq      int
	lastPath string
}

func New() *Controller {
	return &Controller{}
}

func (c *Controller) request(path string) error {
	c.lock.Lock()
	defer c.lock.Unlock()
	c.seq++
	// A real implementation would make a network call here.
	c.lastPath = fmt.Sprintf("%s&seq=%d", path, c.seq)
	return nil
}

// One sends a request and returns the sequence of the request.
func (c *Controller) One(val int) (int, error) {
	if val == 3 {
		return 0, errors.New("sorry, not that one")
	}
	err := c.request(fmt.Sprintf("one?val=%d", val))
	if err != nil {
		return 0, err
	}
	c.lock.RLock()
	defer c.lock.RUnlock()
	return c.seq, nil
}

// Two sends a request and returns the path of the request.
func (c *Controller) Two(val string) (string, error) {
	err := c.request(fmt.Sprintf("two?val=%s", val))
	if err != nil {
		return "", err
	}
	c.lock.RLock()
	defer c.lock.RUnlock()
	return c.lastPath, nil
}

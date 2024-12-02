// Package device is a simple function-based wrapper around
// `controller` that operates on a singleton. You must call Init
// first, and then you can call the other functions, which call
// methods on the singleton.
package device

import (
	"errors"
	"github.com/jberkenbilt/go-to-rust-blog/go/internal/controller"
)

var theController *controller.Controller = nil

// runMethod is a generic dispatcher that is used by the wrapper API
// to call methods on the singleton. It takes a closure that takes a
// *Controller and an arg, calls the closure using the singleton, and
// returns the result.
func runMethod[ArgT any, ResultT any](
	f func(*controller.Controller, ArgT) (ResultT, error),
	arg ArgT,
) (ResultT, error) {
	if theController == nil {
		var zero ResultT
		return zero, errors.New("call Init first")
	}
	return f(theController, arg)
}

func Init() {
	theController = controller.New()
}

func One(val int) (int, error) {
	return runMethod(func(c *controller.Controller, arg int) (int, error) {
		return c.One(arg)
	}, val)
}

func Two(val string) (string, error) {
	return runMethod(func(c *controller.Controller, arg string) (string, error) {
		return c.Two(arg)
	}, val)
}

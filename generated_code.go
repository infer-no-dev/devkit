// Package fib contains functions for calculating the Fibonacci sequence.
package fib

import (
	"fmt"
)

// fibonacci calculates the nth number in the Fibonacci sequence.
func fibonacci(n int) (int, error) {
	if n < 0 {
		return 0, fmt.Errorf("negative index")
	}

	if n <= 1 {
		return n, nil
	}

	a, b := 0, 1
	for i := 2; i <= n; i++ {
		tmp := a + b
		a, b = b, tmp
	}

	return tmp, nil
}
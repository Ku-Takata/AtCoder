package main

import (
	"fmt"
)

func A() {
	var a, b int
	fmt.Scanf("%d %d", &a, &b)
	var ans string
	ans = "Nein"

	if a+b == 9 || a-b == 9 || a*b == 9 {
		ans = "Nine"
	} else if a/b == 9 && a%b == 0 {
		ans = "Nine"
	}

	fmt.Printf(ans)
}

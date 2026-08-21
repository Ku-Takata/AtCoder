package main

import (
	"bufio"
	"fmt"
	"os"
	"strings"
)

func B() {
	var n int
	fmt.Scanf("%d", &n)
	S := map[string]int{}
	scanner := bufio.NewScanner(os.Stdin)
	for i := 0; i < n; i++ {
		scanner.Scan()
		S_lower := strings.ToLower(scanner.Text())
		S[S_lower] += 1
	}
	// fmt.Println(S)

	var cnt int
	for _, v := range S {
		if v > cnt {
			cnt = v
		}
	}

	fmt.Printf("%d", cnt)
}

N = int(input())
A = list(map(int, input().split()))

# 見たことある問題、DP(LIS)

import bisect

length = 0
L = []
dp = [ None ] * N

for i in range(N):
	pos = bisect.bisect_left(L, A[i])
	dp[i] = pos

	if dp[i] >= length:
		L.append(A[i])
		length += 1
	else:
		L[dp[i]] = A[i]

print(L)
print(length)
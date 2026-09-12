N, S, L = map(int,input().split())
A = list(map(int,input().split()))

# Aiは隣の街までの距離
# 初期位置からどちらの方向にも行ける場合は、片方は往復距離になる
# 往復する側が小さい方のルートを選べばよい

S -= 1
dist = [0]*N

for i in range(S+1, N):
	dist[i] = dist[i-1] + A[i-1]
for i in range(S-1, -1, -1):
	dist[i] = dist[i+1] + A[i]

# print(dist)
right = N-1
ans = 1

for left in range(S, -1, -1):
	left_dist = dist[left]
	while right >= S:
		right_dist = dist[right]
		total = left_dist + right_dist + min(left_dist,right_dist)
		# print(left_dist,right_dist)
		# print(total)
		if total <= L:
			break
		right -= 1

	if right < S:
		break

	ans = max(ans,right-left+1)

print(ans)
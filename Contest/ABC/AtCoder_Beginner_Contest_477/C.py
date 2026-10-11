from bisect import bisect_left

Q = int(input())
S = input()
T = input()

# 事前に一括でどこからどこまでの時に満たすかを判定しておく
# つまり正解範囲を持っておくということ
# TLE & WA1つ
# まずTLEはansが原因
# あとはインデックスがおかしいからWAかな

ans = []

start = 0
while True:
	i = S.find(T,start)
	if i == -1:
		break
	ans.append(i+1)
	start = i+1

# print(ans)

for i in range(Q):
	L, R = map(int, input().split())
	index = bisect_left(ans,L)
	if index < len(ans) and ans[index] + len(T)-1 <= R:
		print("Yes")
	else:
		print("No")


"""
ans = []

for i in range(len(S)):
    if S[i:len(T)+i] == T:
        ans.append([i+1,len(T)+i])

# print(ans)

for i in range(Q):
    L, R = map(int,input().split())

    output = "No"
    for l,r in ans:
        if L <= l and R >= r:
            output = "Yes"
            break

    print(output)
"""
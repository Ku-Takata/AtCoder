import heapq

N = int(input())
A = list(map(int,input().split()))

three = []
ans = []

for a in A:
    heapq.heappush(three,a)
    if len(three) > 3:
        heapq.heappop(three)
    if len(three) == 3:
        ans.append(three[0])

for i in range(N-2):
    print(ans[i])
import heapq

N, M = map(int,input().split())
P = list(map(int,input().split()))

# 毎回並びが変わる中での最大・最小値を見つけないといけない
# 

heapq.heapify(P)

for i in range(M):
    L, R = map(int,input().split())

    heapq.heappop(P[L:R])
N,M = map(int,input().split())
N_road = [0]*N

for i in range(M):
    a,b = map(int,input().split())
    N_road[a-1] += 1
    N_road[b-1] += 1

for i in range(N):
    print(N_road[i])
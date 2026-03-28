N, M = map(int,input().split())
AB = [list(map(int,input().split())) for i in range(N)]

group = [0 for i in range(M)]

for i in range(N):
    group[AB[i][1]-1] += 1

for i in range(N):
    group[AB[i][0]-1] -= 1

for i in range(M):
    print(group[i])
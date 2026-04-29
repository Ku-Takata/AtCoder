D, N = map(int,input().split())

work = [24]*D

for i in range(N):
    L, R, H = map(int,input().split())

    for j in range(L-1,R):
        work[j] = min(work[j],H)

print(sum(work))
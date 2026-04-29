N, K = map(int,input().split())

cnt = 0

for i in range(1,N+1):
    if i > K-2:
        break
    for j in range(1,N+1):
        if i+j > K-1:
            break
        if K - (i+j) <= N:
            cnt += 1
            # print(i,j,K-i-j)

print(cnt)
N, K = map(int,input().split())
A = list(map(int,input().split()))

ans = [A[i] for i in range(K-1,N,K)]

print(*ans)
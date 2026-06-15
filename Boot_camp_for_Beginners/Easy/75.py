N, K = map(int,input().split())
H = [int(input()) for i in range(N)]

H.sort()
ans = float("inf")

for i in range(N-K+1):
    min_H = H[i]
    max_H = H[i+K-1]
    ans = min(ans,max_H-min_H)

print(ans)
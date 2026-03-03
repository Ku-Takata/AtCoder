N,K = map(int,input().split())
ans = 0
i = 0

while ans < K:
    ans += N
    N += 1
    i += 1

print(i-1)
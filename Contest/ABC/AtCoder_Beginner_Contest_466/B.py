N, M = map(int,input().split())

ans = [-1]*M

for i in range(N):
    C, S = map(int,input().split())
    C -= 1

    ans[C] = max(ans[C],S)

print(*ans)
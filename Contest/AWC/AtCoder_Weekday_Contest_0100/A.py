N = int(input())

ans = 0
for i in range(N):
    A,T = map(int,input().split())
    ans += A*T

print(ans)
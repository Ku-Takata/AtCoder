T = int(input())
N = int(input())

plan = [0]*(T+1)

for i in range(N):
    L, R = map(int,input().split())
    plan[L] += 1
    plan[R] -= 1

# print(plan)
ans = 0

for i in range(T):
    ans = ans + plan[i]
    print(ans)
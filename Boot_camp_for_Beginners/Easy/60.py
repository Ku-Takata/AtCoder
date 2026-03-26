N = int(input())
T = list(map(int,input().split()))
M = int(input())

total = sum(T)

for i in range(M):
    P, X = map(int,input().split())
    drink = T[P-1] - X
    ans = total - drink
    print(ans)
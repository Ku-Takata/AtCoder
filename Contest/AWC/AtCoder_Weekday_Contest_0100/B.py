N = int(input())

total = 0
for i in range(N):
    A,B = map(int,input().split())
    if A+B > total:
        total = A+B
        ans = i+1

print(ans)
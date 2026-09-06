N, Q = map(int,input().split())
P = list(map(int,input().split()))

num = dict()

for i in range(N):
    num[P[i]] = i

for i in range(1,Q+1):
    a = int(input())
    num[a] = N+i

num_items = list(num.items())
num_items.sort(key=lambda x: x[1])
ans = []

for i in range(N):
    ans.append(num_items[i][0])

print(*ans)
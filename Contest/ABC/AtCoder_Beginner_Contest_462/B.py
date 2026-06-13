N = int(input())
gift = [list(map(int,input().split())) for i in range(N)]

ans = [[] for i in range(N)]

for i in range(N):
    for A in gift[i][1:]:
        ans[A-1].append(i+1)

# print(ans)

for i in range(N):
    if ans[i]:
        print(len(ans[i]),*ans[i])
    else:
        print(len(ans[i]))
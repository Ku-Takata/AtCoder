N, K, X = map(int,input().split())
S = [input() for i in range(N)]

# N**K <= 10**5なので、全探索後にソートできる

i = 0
ans = []

def solve(s,i):
    if i == K:
        ans.append(s)
        return

    for next in S:
        solve(s+next,i+1)

solve("",0)
ans.sort()
print(ans[X-1])
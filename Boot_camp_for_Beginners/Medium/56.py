N, K, S = map(int,input().split())

# これってS, S, S, ...という数列を作って、ちょうどK個になるよう最後の方は調整したら良いのでは？

ans = []

for i in range(K):
    ans.append(S)

if N-len(ans) < S:
    for i in range(N-len(ans)):
        ans.append(1)
else:
    for i in range(N-len(ans)):
        ans.append(10**9)

print(*ans)
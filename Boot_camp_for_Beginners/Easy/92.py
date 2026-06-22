S = list(input())
L = []
ans = 0

for i in range(len(S)):
    if len(L) == 0:
        L.append(S[i])
        continue
    if (L[-1] == "1" and S[i] == "0") or (L[-1] == "0" and S[i] == "1"):
        ans += 2
        L.pop(-1)
    else:
        L.append(S[i])

print(ans)
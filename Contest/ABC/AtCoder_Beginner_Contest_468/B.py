M, D = map(int,input().split())
S = input()

ans = 0

for i in range(M):
    if i < D:
        if "G" not in S[0:i+D+1]:
            ans += 1
    elif i < M-D:
        if "G" not in S[i-D:i+D+1]:
            ans += 1
    else:
        if "G" not in S[i-D:M]:
            ans += 1

    # print(ans)

print(ans)
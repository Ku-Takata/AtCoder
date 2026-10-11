N = int(input())
S = input()

ans = 0
cnt = 0

for i in range(N):
    if S[i] == "R":
        cnt += 1
    else:
        ans = max(ans,cnt)
        cnt = 0

ans = max(ans,cnt)

print(ans)
N = int(input())
S = list(input())
x = 0
ans = 0

for i in range(N):
    if x > ans:
        ans = x
    if S[i] == "I":
        x += 1
    else:
        x -= 1

if x > ans:
    ans = x

print(ans)

# 3 何も難しくない
N = int(input())
H = list(map(int,input().split()))

dp = [0]

for i in range(1,N):
    if i == 1:
        dp.append(abs(H[0]-H[1]))
    else:
        dp.append(min(dp[-1]+abs(H[i-1]-H[i]), dp[-2]+abs(H[i-2]-H[i])))

i = len(dp)-1
P = [i+1]

while i > 0:
    if dp[i] == dp[i-1] + abs(H[i] - H[i-1]):
        P.append(i)
        i -= 1
    else:
        P.append(i-1)
        i -= 2

print(len(P))
print(*reversed(P))
N = int(input())
H = list(map(int,input().split()))

dp = [0]

for i in range(1,N):
    if i == 1:
        dp.append(abs(H[0]-H[1]))
    else:
        dp.append(min(dp[-1]+abs(H[i-1]-H[i]),dp[-2]+abs(H[i-2]-H[i])))

print(dp[-1])
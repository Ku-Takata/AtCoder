N,K = map(int,input().split())

prob = 0

for i in range(1,N+1):
    for j in range(0,100):
        if i >= K:
            coin_cnt = j
            break
        else:
            i *= 2
    prob += 1/N * (1/2)**coin_cnt

print(prob)
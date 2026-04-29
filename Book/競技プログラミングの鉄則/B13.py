N, K = map(int,input().split())
A = list(map(int,input().split()))

cs = [0]
for i in range(N):
    cs.append(cs[-1]+A[i])

cnt = 0
j = 0

for i in range(N):
    while j < N:
        if cs[j+1] - cs[i] <= K:
            j += 1
        else:
            break
    cnt += j-i
    # print(cs[i],cs[j],cnt)

print(cnt)
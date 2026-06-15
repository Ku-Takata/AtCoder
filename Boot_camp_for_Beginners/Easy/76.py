N, M = map(int,input().split())

result = [0]*N
penalty = [0]*N

for i in range(M):
    p,S = map(str,input().split())
    p = int(p) - 1

    if result[p] == 0:
        if S == "AC":
            result[p] = 1
        else:
            penalty[p] += 1

print(sum(result),sum(penalty[i] for i in range(N) if result[i] == 1))
N, K = map(int,input().split())
D = sorted(list(map(int,input().split())),reverse=True)

for i in range(K):
    D[i] = 0

print(sum(D))
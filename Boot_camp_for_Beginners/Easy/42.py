N,M = map(int,input().split())
foods = [0] * M

for i in range(N):
    Q = list(map(int,input().split()))
    for j in range(1,Q[0]+1):
        foods[Q[j]-1] += 1

print(foods.count(N))
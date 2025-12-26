N = int(input())
a = list(map(int, input().split()))

Alice = []
Bob = []

for i in range(N):
    saidai = max(a)
    if i % 2 == 0 or i == 0:
        Alice.append(saidai)
        a.remove(saidai)
    else:
        Bob.append(saidai)
        a.remove(saidai)

print(sum(Alice) - sum(Bob))
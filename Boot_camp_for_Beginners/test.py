H,W = map(int,input().split())
A = []

for i in range(H):
    a = input()
    if "#" in a:
        A.append(a)

A = list(zip(*A))
A_rev = []

for a in A:
    if "#" in a:
        A_rev.append(a)

A_rev = zip(*A_rev)

for a in A_rev:
    print(*a, sep="")
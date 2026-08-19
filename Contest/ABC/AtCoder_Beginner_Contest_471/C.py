N = int(input())
A = list(map(int,input().split()))

# ソートして、インデックスを右左で作って、右か左を選択するみたいなのでいけそう

A.append(0)
A.sort()
ini = A.index(0)
l,r = ini-1,ini+1
now = 0
dist = 0

for i in range(N):
    if l >= 0 and r <= N:
        if abs(now - A[l]) <= abs(now - A[r]):
            dist += abs(now - A[l])
            now = A[l]
            l -= 1
        else:
            dist += abs(now - A[r])
            now = A[r]
            r += 1
    elif l >= 0:
        dist += abs(now - A[l])
        now = A[l]
        l -= 1
    else:
        dist += abs(now - A[r])
        now = A[r]
        r += 1

print(dist)
N, M, K = map(int,input().split())
H = list(map(int,input().split()))
B = list(map(int,input().split()))

# 貪欲にBとHが近いかつBの方が重いように選んでいって、K体以上作れるか判定したら良い

H.sort()
B.sort()

i,j = 0,0
cnt = 0

while i < N and j < M:
    if H[i] <= B[j]:
        cnt += 1
        i += 1
        j += 1
    else:
        j += 1

if cnt >= K:
    print("Yes")
else:
    print("No")
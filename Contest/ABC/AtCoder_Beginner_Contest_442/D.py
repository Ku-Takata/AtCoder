N,Q = map(int,input().split())
A = list(map(int,input().split()))

# 愚直法、絶対TLE
"""
for i in range(Q):
    query = list(map(int,input().split()))
    if len(query) == 2:
        A[query[1]-1], A[query[1]] = A[query[1]], A[query[1]-1]
    else:
        print(sum(A[query[1]-1:query[2]]))
"""

# sumのところをスライスしており、そこの計算がネックになっている。
"""
for i in range(Q):
    query = list(map(int,input().split()))
    if len(query) == 2:
        A[query[1]-1], A[query[1]] = A[query[1]], A[query[1]-1]
    else:
        total = 0
        for j in range(query[2]-query[1]+1):
            total += A[query[1]-1+j]
        print(total)
"""

# 累積和から導き出す
cs = []
total = 0

for i in range(N):
    total += A[i]
    cs.append(total)

# print(cs)

# swapしたところだけ累積和の値が変わることに注目
for i in range(Q):
    query = list(map(int,input().split()))
    if len(query) == 2:
        if query[1] != 1:
            cs[query[1]-1] = cs[query[1]-2]+(cs[query[1]] - cs[query[1]-1])
        else:
            cs[query[1]-1] = (cs[query[1]] - cs[query[1]-1])
        # print(cs)
    else:
        if query[1] > 1:
            print(cs[query[2]-1] - cs[query[1]-2])
        else:
            print(cs[query[2]-1])

# 終了後10分で解けた。愚直なやり方を試していなければ時間内に解けていたしペナも2回受けなかった...変なことしないようにしよう。
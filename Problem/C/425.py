N, Q = map(int,input().split())
A = list(map(int,input().split()))

# 今どこが先頭位置なのかを記録する

A = A + A

cs = [0]
for i in range(2*N):
    cs.append(cs[i]+A[i])

first_i = 0

for i in range(Q):
    num, *que = map(int,input().split())

    if num == 1:
        c = que[0]
        first_i = (first_i+c)%N
    else:
        l,r = que
        ans = cs[r+first_i] - cs[l-1+first_i]
        print(ans)

# print(A)
# print(cs)
N = int(input())
A = list(map(int,input().split()))

# どこかは2で割らないといけない
# 初期値で2で割れる回数をカウントしたら良い
# 2**30 ≒ 10**9なので、全ての要素が10**9だとしても、全探索で30*10000 = 3*10**5で済む

cnt = 0

for i in range(N):
    while (A[i]/2).is_integer():
        A[i] //= 2
        cnt += 1

print(cnt)
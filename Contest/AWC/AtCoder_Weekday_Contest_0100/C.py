N, L, Q = map(int,input().split())
S = [list(input()) for i in range(N)]
M = [list(map(int,input().split())) for i in range(Q)]

# つまり求められるキーがSで、そのインデックスをMで指定してて、そのキーを満たすよう1を置くようにしたい
# ただし置く1は最小である必要がある

for i in range(Q):
    ans = ["0"]*L
    for j in range(M[i][0]):
        for k in range(L):
            if ans[k] == "0":
                if S[M[i][j+1]-1][k] == "1":
                    ans[k] = "1"

    print("".join(ans))
N, M = map(int, input().split())
AB = [list(map(int, input().split())) for i in range(M)]

# ここにコードを追記する
result = [["-" for i in range(N)] for i in range(N)]

for a,b in AB:
    a -= 1
    b -= 1
    result[a][b] = "o"
    result[b][a] = "x"

for i in result:
    print(" ".join(i))
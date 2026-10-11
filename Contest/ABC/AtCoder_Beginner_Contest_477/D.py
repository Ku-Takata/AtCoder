N, Q = map(int,input().split())

# タイルの置かれていない所を何らかで記録する必要がある
# 愚直に変更し続けると確実にTLE
# どういうケースでWAが予想すると、

mass = ["a" for i in range(N)]
tile = [False for i in range(N)]
now = "a"

for i in range(Q):
    query = list(map(str,input().split()))

    if query[0] == "1":
        X = int(query[1])-1
        if not tile[X]:
            tile[X] = True
            mass[X] = now
        else:
            tile[X] = False
    else:
        C = query[1]
        now = C

ans = [mass[i] if tile[i] else now for i in range(N)]

    # print(tile)
    # print(ans)

print("".join(ans))
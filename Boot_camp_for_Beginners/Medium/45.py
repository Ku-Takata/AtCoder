N, M = map(int,input().split())

# c2つでsに変換可能

ans = 0

if 2*N < M:
    M -= 2*N
    ans += N
else:
    ans += M // 2
    print(ans)
    exit()

# ここからはcをsに交換して増やせるかどうか
# sccを作るにはcが4つ必要
ans += M // 4
print(ans)
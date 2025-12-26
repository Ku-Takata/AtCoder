N, S = map(int, input().split())

A = list(map(int, input().split()))
B = list(map(int, input().split()))

# リンゴ・パイナップルをそれぞれ1つずつ購入するとき合計S円になるような買い方が何通りあるか
# ここにプログラムを追記

count = 0

for a in range(N):
    for b in range(N):
        if A[a] + B[b] == S:
            count += 1

print(count)
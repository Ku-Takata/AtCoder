N,K = map(int,input().split())
T = []
for i in range(N):
    T.append(list(map(int,input().split())))

# itertoolsを使って組み合わせを調べる
import itertools

cnt = 0

for root in itertools.permutations(range(2,N+1)):
    root = root + (1,)
    time = 0
    now = 0
    for i in range(N):
        time += T[now][root[i]-1]
        now = root[i]-1
    if time == K:
        cnt += 1

print(cnt)

# 41 問題文の理解に20分くらいかかってしまった。実装はitertoolsの使い方を学び、タプルに対してどうやって追加したらいいかを学んだ。
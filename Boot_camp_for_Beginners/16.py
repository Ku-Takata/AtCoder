N = int(input())
A = list(map(int, input().split())) # 出席番号 = リストインデックス, 値 = その時いた人数
# result = []
result = [0 for x in range(N)]

# 一致する値のインデックスを取得し、それを+1することで帳尻を合わせる -> 5/16 TLE
"""for i in range(N+1):
    if i == 0:
        continue
    else:
        result.append(A.index(i) + 1)"""

# 後で全ての要素に+1するだけだと 4/16 TLEというように少しの改善しかされない
"""for i in range(N+1):
    if i == 0:
        continue
    else:
        result.append(A.index(i))

result = list(map(lambda x: x + 1, result))"""

# 原因は.indexで探索を行っているためforループと合わせてO(N^2)となっており、N <= 10^5であることからオーバーしてしまう

# これならO(NlogN)程度なのでOKだが、まだ改善できる
"""for i in range(N):
    A[i] = [A[i], i+1]

A.sort()

for i in range(N):
    result.append(A[i][1])"""

# これならO(N)で計算できる
for i in range(N):
    result[A[i]-1] = i+1

print(*result)
N, M, Q = map(int,input().split())
point_table = []
for i in range(Q):
    point_table.append(list(map(int,input().split())))

A = [1]*N
ans = 0
j = 0

import sys
sys.setrecursionlimit(10**7)

def dfs(n_seq):
    global ans,j

    if sum(n_seq) == M*N:
        return

    total = 0
    for i in range(Q):
        if n_seq[point_table[i][1]-1] - n_seq[point_table[i][0]-1] == point_table[i][2]:
            total += point_table[i][3]

    ans = max(ans,total)

    # リストの要素を順番に1ずつ足し、Mまで行ったら次の要素を足す操作を再帰的に行いたい
    if j < N:
        return ans
    elif n_seq[j] < M:
        n_seq[j] += 1
    else:
        j += 1
        n_seq[j] += 1

    return dfs(n_seq)

print(dfs(A))
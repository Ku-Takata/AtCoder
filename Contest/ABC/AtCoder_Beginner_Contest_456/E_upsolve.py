import sys
from collections import deque

# 入力の高速化
input = sys.stdin.readline

def solve():
    # テストケースの数
    line = input().split()
    if not line: return
    T = int(line[0])

    for _ in range(T):
        N, M = map(int, input().split())
        UV = [[] for _ in range(N + 1)]
        for _ in range(M):
            U, V = map(int, input().split())
            UV[U].append(V)
            UV[V].append(U)

        # 「とどまる」という選択肢を隣接リストに追加
        for i in range(1, N + 1):
            UV[i].append(i)

        W = int(input())
        S = [input().strip() for _ in range(N)]

        # dp[day][city]: その日のその都市が「詰んでいる」かどうか
        # True: 詰んでいる（平日である、または移動先がすべて詰んでいる）
        dp = [[False] * (N + 1) for _ in range(W)]

        # out_degree[day][city]: 次の日の移動候補のうち、現時点で「生きている」都市の数
        out_degree = [[0] * (N + 1) for _ in range(W)]
        queue = deque()

        # 初期化：まず「平日」を死に体としてマークし、移動先の「休日」を数える
        for d in range(W):
            next_d = (d + 1) % W
            for u in range(1, N + 1):
                if S[u-1][d] == 'x':
                    dp[d][u] = True
                else:
                    # 休日(o)なら、次の日の候補(v)が何個休日かカウント
                    cnt = 0
                    for v in UV[u]:
                        if S[v-1][next_d] == 'o':
                            cnt += 1
                    out_degree[d][u] = cnt
                    # 移動先が一つも休日でないなら、この状態も詰み
                    if cnt == 0:
                        queue.append((d, u))

        # 詰んでいる状態を伝播させる（トポロジカルソートの要領）
        while queue:
            d, u = queue.popleft()
            if dp[d][u]: continue
            dp[d][u] = True

            # 状態(d, u)が「詰み」に変わったので、
            # 前の日(prev_d)にここへ来ようとしていた都市vの「生存移動先カウント」を減らす
            prev_d = (d - 1) % W
            for v in UV[u]:
                if not dp[prev_d][v]:
                    out_degree[prev_d][v] -= 1
                    if out_degree[prev_d][v] == 0:
                        queue.append((prev_d, v))

        # 1日目(d=0)に、詰んでいない都市が1つでもあれば Yes
        ans = "No"
        for i in range(1, N + 1):
            if not dp[0][i]:
                ans = "Yes"
                break
        print(ans)

if __name__ == "__main__":
    solve()
import sys

input = sys.stdin.readline
N, K = map(int,input().split())

# その列での同じ値での探索終了条件は、(残り//i)+1になったとき
# それまでは+1ずつ増やし続けて大丈夫
# 最悪計算量はO(KlogK)?で間に合うはず

A = [0]*N
ans = []

def dfs(pos, rem):
    # pos が N を超えたら、残りの rem をすべて A[0] (A_1) に割り当てる
    if pos > N:
        A[0] = rem
        ans.append(tuple(A))
        return

    # A[pos-1] (A_pos) を 0 から rem // pos まで試す
    for A_i in range((rem // pos) + 1):
        A[pos - 1] = A_i
        dfs(pos + 1, rem - (pos * A_i))

# A_2 〜 A_N を決めるため pos = 2 から開始
dfs(2, K)

# 辞書順にするためにソート
ans.sort()

# 出力
out = []
for row in ans:
    out.append(" ".join(map(str, row)))
sys.stdout.write("\n".join(out) + "\n")
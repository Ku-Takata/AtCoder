import sys

input = sys.stdin.readline
N, K = map(int,input().split())

# その列での同じ値での探索終了条件は、(残り//i)+1になったとき
# それまでは+1ずつ増やし続けて大丈夫
# 最悪計算量はO(KlogK)?で間に合うはず

A = [0]*N
ans = []

def can_make(pos, rem):
    if rem == 0:
        return True
    if pos > N or rem < 0:
        return False

    for A_i in range((rem//pos)+1):
        if can_make(pos+1, rem-(pos*A_i)):
            return True
    return False

def dfs(pos, rem):
    if pos > N:
        if rem == 0:
            ans.append(" ".join(map(str,A)))
        return
    if not can_make(pos, rem):
        return

    for A_i in range((rem//pos)+1):
        A[pos-1] = A_i
        if can_make(pos+1, rem-(pos*A_i)):
            dfs(pos+1, rem-(pos*A_i))

dfs(1, K)
sys.stdout.write("\n".join(ans))
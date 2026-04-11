N = int(input())
L = list(map(int,input().split()))

# Nが非常に小さい
# 2**20 = 1045876 ≒ 10**6
# マイナスとプラス方向どちらも回答を保存しておける
# DFSでいける

import sys
sys.setrecursionlimit(10**7)

def dfs(i,current_pos):
    global N,L

    if i == N:
        return 0

    next_pos_plus = current_pos + L[i]
    if current_pos < 0 and next_pos_plus > 0:
        ans_plus = 1
    else:
        ans_plus = 0
    result_plus = ans_plus + dfs(i+1, next_pos_plus)

    next_pos_minus = current_pos - L[i]
    if current_pos > 0 and next_pos_minus < 0:
        ans_minus = 1
    else:
        ans_minus = 0
    result_minus = ans_minus + dfs(i+1, next_pos_minus)

    return max(result_plus, result_minus)

ans = dfs(0,0.5)
print(ans)
N, L = map(int,input().split())
S = [input() for i in range(N)]

# 普通に辞書順でソートしたら良い
S.sort()
print("".join(S))
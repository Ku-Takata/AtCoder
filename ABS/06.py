N, A, B = map(int,input().split())
X = []

for i in range(1, N+1):
    i_str_list = list(str(i))
    i_int_list = [int(x) for x in i_str_list]
    if A <= sum(i_int_list) <= B:
        X.append(i)

result = sum(X)
print(result)
# 各桁の和をNまで順番に計算して、A以上B以下を満たす各桁の和をリストに格納し、リスト内の数を合計する
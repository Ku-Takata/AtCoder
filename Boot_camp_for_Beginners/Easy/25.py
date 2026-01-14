N = int(input())
P = tuple(map(int,input().split()))
Q = tuple(map(int,input().split()))

import itertools

dict_num = []

# 全列挙してもMax 8!で済む
for comb in itertools.permutations(range(1,N+1)):
    dict_num.append(comb)

a = dict_num.index(P) + 1
b = dict_num.index(Q) + 1

print(abs(a-b))

# 20 tupleとlistの型で時間かけてしまった
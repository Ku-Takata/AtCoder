N = int(input())
C = [list(map(int,input().split())) for i in range(N-1)]

# abcの組み合わせを全列挙からの計算して安くなるか判定
from itertools import combinations
N_list = [i for i in range(1,N+1)]
comb_abc = list(combinations(N_list,3))
# print(comb_abc)
ans = "No"

for i in range(len(comb_abc)):
    calc_ab = C[comb_abc[i][0]-1][comb_abc[i][1]-comb_abc[i][0]-1]
    calc_bc = C[comb_abc[i][1]-1][comb_abc[i][2]-comb_abc[i][1]-1]
    calc_ac = C[comb_abc[i][0]-1][comb_abc[i][2]-comb_abc[i][0]-1]
    # print(calc_ab, calc_bc, calc_ac)

    if calc_ab + calc_bc < calc_ac:
        ans = "Yes"

print(ans)
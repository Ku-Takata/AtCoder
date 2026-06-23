N = int(input())
A = list(map(int,input().split()))

# 1つの要素につき似ている値は3つある
# その上で積が偶数になるには1要素だけでも偶数であれば良いので、全組み合わせ-全部奇数
# 偶奇の数リストに直す

cnt_odd = [2 if a % 2 == 0 else 1 for a in A]
odd_comb = 1

for c in cnt_odd:
    odd_comb *= c

print(3**N - odd_comb)
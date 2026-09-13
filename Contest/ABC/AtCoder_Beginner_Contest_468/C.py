import itertools

N = int(input())
P = tuple(map(int,input().split()))
Q = tuple(map(int,input().split()))

# 全探索できそう
# 10で3628800だから行ける

A = [i for i in range(1,N+1)]
A_list = list(itertools.permutations(A,N))
# print(len(A_list))

ans = 0

for a in A_list:
    if a == P:
        ans = 0
    elif a == Q:
        ans -= 1
        break

    ans += 1

check = [P,Q]
sort_check = sorted(check)

if check == sort_check and P != Q:
    print(ans)
else:
    print(0)
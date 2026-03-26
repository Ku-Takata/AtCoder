s = list(input())
t = list(input())

# sを一番小さい辞書順に並べ替え、tを一番大きい辞書順に並び替えて比較したら良いだろう

s_prime = "".join(sorted(s))
t_prime = "".join(sorted(t,reverse=True))
st_prime = sorted([s_prime,t_prime])

# print(st_prime)

if st_prime[0] == s_prime and s != t:
    print("Yes")
else:
    print("No")
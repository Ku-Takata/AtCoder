N = int(input())
S = input()

# Aがあるインデックスを調べる
A_index = []

for i in range(N):
    if S[i] == "A":
        A_index.append(i)

print(A_index)
A_cnt = 1
sub_cnt = 0

# A以前を含めた部分文字列を調べる
for a in A_index:
    for i in range():
        if A_index-i > 0:
            if S[A_index-i] == "C":
                sub_cnt += 1
            elif S[A_index-i] == "B":
                B_cnt += 1
                if A_cnt > B_cnt:
                    sub_cnt += 1
                else:
                    None #ここまでしか書いていない

# この貪欲法だと明らかにTLEになる
# Aの方が多い条件というのが問題をかなり難しくしている
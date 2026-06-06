N, K, M = map(int,input().split())
CV = []
color = []
for i in range(N):
    C,V = map(int,input().split())
    CV.append([C,V])
    color.append(C)

color = sorted(list(set(color)))

# M種類選ぶのは確定で、K個選ぶ
# だからまずは各色の中で最も大きい宝石をリランキングし、大きいものから取る
# 取る際はK個までに注意
# その後、取ったもの以外で大きい宝石をK個まで取る

CV.sort(key=lambda x: (x[0],-x[1]))
# print(CV)

from collections import Counter
cnt_CV = Counter(map(tuple,CV))
# print(cnt_CV)

# 色は順番にあるわけではなく好きな値が割り振られていることに注意
a = 0
len_c = len(color)
c_max_v = []

for i in range(N):
    if color[a] == CV[i][0]:
        c_max_v.append([color[a],CV[i][1]])
        a += 1
        if a >= len_c:
            break
        continue

# print(c_max_v)
c_max_v.sort(key=lambda x: -x[1])

ans_c = 0
for i in range(M):
    ans_c += c_max_v[i][1]
    cnt_CV[tuple(c_max_v[i])] -= 1
# print(ans_c)

CV.sort(key=lambda x: -x[1])

ans = ans_c
cnt = K-M

for i in range(N):
    if cnt == 0:
        break

    trg = tuple(CV[i])
    if cnt_CV[trg] == 0:
        continue
    else:
        ans += trg[1]
        cnt -= 1

print(ans)
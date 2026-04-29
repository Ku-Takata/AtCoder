N,K = map(int,input().split())
AB = []

for i in range(N):
    AB.append(list(map(int,input().split())))

AB.insert(0,[0,0])
AB = sorted(AB)

# 友人までの距離コストと貰えるコストにリストを改良する。つまり友人間の差を表すリストにする。
i = 0
AB_rev = []

for i in range(N):
    AB_rev.append((AB[i+1][0] - AB[i][0], AB[i+1][1]))

# print(AB_rev)

ans = 0

for i in range(len(AB_rev)):
    if K - AB_rev[i][0] >= 0:
        K = K - AB_rev[i][0] + AB_rev[i][1]
        ans += AB_rev[i][0]
    else:
        ans += K
        K = 0
        break

if K != 0:
    ans += K

print(ans)

# 60over 方針が定まらず結構時間がかかってしまった。また、この方法でも良いが、実行時間が500ms程度と遅いので、別の方法でもう少し早くなる。
# それは、K円分だけ一気に進み、その間に友達がいたらお金を追加するという考え方である。
# 個人的には自分のコードの方が直感的だが、200ms程差があるので、お手本の方法をやるべきである。
# 遅い原因は差のリストを新しく作ったこと。
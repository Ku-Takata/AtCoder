N = int(input())
XY = [list(map(int,input().split())) for i in range(N)]

# どういうこと？？
# 全ての点と(0,0)を対角とした長方形の内部に、今までの点全てを見て中に点が無かったらカウントってこと？？
# 辺の上にある点は内部ではないらしい
# それぞれの順列という意味が分からない、たぶんXとYそれぞれ被りが無いように点が存在しているということかな？
# 見た感じやとそれぞれのxで大きいyさえ見ておけば良さそう
# その大きい長方形の中で辺の上ではない点がある
# 例2とかは(5,5)の中にすべての点があるから、1つだけになる
# 例3は(1,3)と(6,1)の2つ
# 答えの点は必ず内側にいる（内部に点がないものだから）
# これ(1,1)に点あったら答え1つしかないね、他は順列であることから点がx,y共に2以上にいるから
# これまず、xが1の時にyはいくつか、とyが1の時にxはいくつかを確認すべきなのでは？
# それによって次に内部に点が無い状態にできる点を絞り込めるから、そこから探索していったらいいんじゃない？
# これマックスが(1,Y)のYが理論上最大やね
# これ階段状になっているのが理想でどれくらい階段ができているかを確認するだけでいいんじゃない？

sort_X = sorted(XY,key=lambda x: x[0])
sort_Y = sorted(XY,key=lambda x: x[1])
# print(sort_X)
# print(sort_Y)

"""
cnt = 1
base1 = [sort_X[0][0],sort_X[0][1]]
base2 = [sort_Y[0][0],sort_Y[0][1]]
print(base1,base2)

for i in range(base1[1]):
    if sort_Y[base1[1]-1][0] < base2[0]:
        cnt += 1

print(cnt)
"""

cnt = 1
now = sort_X[0][1]
for i in range(1,N):
    if sort_X[i][1] <= now:
        cnt += 1
        now = sort_X[i][1]
    else:
        continue

print(cnt)
Q = int(input())

# 愚直にやると、最初は生やしまくって、最後に削除祭りだとTLEになる
# numpy配列でもダメ
# 2の処理をどうしたら問題
# 優先度付キューで最小値を拾ってh以下か判定するのならどうだろう
# heapqの計算量はlogN、hが大きかった時にダメな気もする
# たぶん方針は合ってる、けどWAが出るケースがある

from collections import deque
import heapq

tree = []

for i in range(Q):
    num,h = map(int,input().split())

    if num == 1:
        heapq.heappush(tree, h)
    else:
        while tree and tree[0] <= h:
            heapq.heappop(tree)

    print(len(tree))
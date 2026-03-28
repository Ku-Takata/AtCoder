Q = int(input())

# 愚直にやると、最初は生やしまくって、最後に削除祭りだとTLEになる
import numpy as np

tree = np.array([])

for i in range(Q):
    num,h = map(int,input().split())

    if num == 1:
        tree = np.append(tree, h)
    else:
        tree = tree[tree > h]
    # print(tree)
    print(len(tree))
N, Q = map(int,input().split())
P = [0 for i in range(N)]

# たぶん遅延セグ木ってやつ
# クエリ1は簡単だけど、クエリ2の意味が理解できてない
# 

from atcoder.lazysegtree import LazySegTree

INF = 1 << 63
ID = INF


def op(ele1, ele2):
    return max(ele1, ele2)


def mapping(func, ele):
    if func == ID:
        return ele
    else:
        return func


def composition(func_upper, func_lower):
    if func_upper == ID:
        return func_lower
    else:
        return func_upper


e = -INF
id_ = ID

# TODO (初期リストlst)
seg = LazySegTree(op, e, mapping, composition, id_, lst)


for i in range(Q):
    query = list(map(int,input().split()))
    if query[0] == 1:

    else:
        None

print()
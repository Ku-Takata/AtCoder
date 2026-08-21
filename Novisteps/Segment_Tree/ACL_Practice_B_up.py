N, Q = map(int,input().split())
A = list(map(int,input().split()))

class FenwickTree:
    def __init__(self, size):
        self.n = size
        self.tree = [0] * (size + 1)

    def add(self, idx, val):
        """A[idx] に val を加算 (0-indexed)"""
        i = idx + 1
        while i <= self.n:
            self.tree[i] += val
            i += i & -i

    def sum(self, l, r):
        """半開区間 [l, r) の和を取得"""
        return self._sum(r) - self._sum(l)

    def _sum(self, idx):
        """区間 [0, idx) の和を取得"""
        s = 0
        i = idx
        while i > 0:
            s += self.tree[i]
            i -= i & -i
        return s

bit = FenwickTree(N)
for i in range(N):
    bit.add(i,A[i])

for i in range(Q):
    q0, q1, q2 = map(int,input().split())

    if q0 == 0:
        bit.add(q1,q2)
    else:
        print(bit.sum(q1,q2))
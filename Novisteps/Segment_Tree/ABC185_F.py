N, Q = map(int,input().split())
A = list(map(int,input().split()))

class FenwickTree:
    def __init__(self, size, is_xor=False):
        self.n = size
        self.tree = [0] * (size + 1)
        self.raw = [0] * size
        self.is_xor = is_xor

    def build(self, arr):
        """初期配列から O(N) で構築"""
        for i in range(self.n):
            self.raw[i] = arr[i]
            self.tree[i + 1] = arr[i]
        for i in range(1, self.n + 1):
            j = i + (i & -i)
            if j <= self.n:
                if self.is_xor:
                    self.tree[j] ^= self.tree[i]
                else:
                    self.tree[j] += self.tree[i]

    def add(self, idx, val):
        """A[idx] に val を加算 / XOR作用"""
        if self.is_xor:
            self.raw[idx] ^= val
        else:
            self.raw[idx] += val
        i = idx + 1
        while i <= self.n:
            if self.is_xor:
                self.tree[i] ^= val
            else:
                self.tree[i] += val
            i += i & -i

    def update(self, idx, val):
        """A[idx] = val に上書き更新 (セグ木と同仕様)"""
        if self.is_xor:
            diff = val ^ self.raw[idx]
        else:
            diff = val - self.raw[idx]
        self.add(idx, diff)

    def query(self, l, r):
        """半開区間 [l, r) の和/XORを取得 (セグ木と同仕様)"""
        if self.is_xor:
            return self._sum(r) ^ self._sum(l)
        else:
            return self._sum(r) - self._sum(l)

    def _sum(self, idx):
        s = 0
        i = idx
        while i > 0:
            if self.is_xor:
                s ^= self.tree[i]
            else:
                s += self.tree[i]
            i -= i & -i
        return s

bit = FenwickTree(N, is_xor=True)
bit.build(A)

for i in range(Q):
    T, X, Y = map(int,input().split())

    if T == 1:
        bit.add(X-1,Y)
    else:
        print(bit.query(X-1,Y))
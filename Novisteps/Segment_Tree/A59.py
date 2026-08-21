N, Q = map(int,input().split())
A = [0]*N

class SegmentTree:
    def __init__(self, size, op, e):
        self.n = size
        self.op = op
        self.e = e
        self.size = 1 << (size - 1).bit_length()
        self.tree = [e] * (2 * self.size)

    def build(self, arr):
        for i in range(self.n):
            self.tree[self.size + i] = arr[i]
        for i in range(self.size - 1, 0, -1):
            self.tree[i] = self.op(self.tree[2 * i], self.tree[2 * i + 1])

    def update(self, idx, val):
        i = self.size + idx
        self.tree[i] = val
        while i > 1:
            i >>= 1
            self.tree[i] = self.op(self.tree[2 * i], self.tree[2 * i + 1])

    def query(self, l, r):
        s_left = self.e
        s_right = self.e
        l += self.size
        r += self.size
        while l < r:
            if l & 1:
                s_left = self.op(s_left, self.tree[l])
                l += 1
            if r & 1:
                r -= 1
                s_right = self.op(s_right, self.tree[r])
            l >>= 1
            r >>= 1
        return self.op(s_left, s_right)

# 区間和
st = SegmentTree(N, lambda x, y: x + y, 0)
st.build(A)

for i in range(Q):
    q0, q1, q2 = map(int,input().split())

    if q0 == 1:
        st.update(q1-1,q2)
    else:
        print(st.query(q1-1,q2-1))
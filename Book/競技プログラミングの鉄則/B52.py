N, X = map(int,input().split())
A = list(input())

X -= 1

from collections import deque
que = deque()
que.append(X)
A[X] = "@"

while len(que) != 0:
    pos = que.popleft()
    if pos > 0 and A[pos-1] == ".":
        A[pos-1] = "@"
        que.append(pos-1)
    if pos < N-1 and A[pos+1] == ".":
        A[pos+1] = "@"
        que.append(pos+1)
    # print(que)
print("".join(A))
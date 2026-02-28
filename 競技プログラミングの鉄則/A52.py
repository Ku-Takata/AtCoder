Q = int(input())

from collections import deque
stack = deque()

for i in range(Q):
    q = list(map(str,input().split()))
    q[0] = int(q[0])

    if q[0] == 1:
        stack.append(q[1])
    elif q[0] == 2:
        print(stack[0])
    else:
        stack.popleft()
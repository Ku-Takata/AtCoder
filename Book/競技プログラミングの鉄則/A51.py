Q = int(input())

from collections import deque

stack = deque()

for i in range(Q):
    q = list(map(str,input().split()))
    q[0] = int(q[0])

    if q[0] == 1:
        stack.append(q[1])
    elif q[0] == 2:
        print(stack[-1])
    else:
        stack.pop()

# stack = deque()のところはシンプルにstack = []でも可能だが、先頭への追加・削除を行う場合、
# listはO(N)、dequeはO(1)である。
# 一方でランダムアクセスの場合、
# listはO(1)、dequeはO(N)である。
# あとdequeの場合は、pop(-1)とは書かずpop()で良い
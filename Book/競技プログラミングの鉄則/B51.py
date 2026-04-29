S = input()

from collections import deque
stack = deque()

for i in range(len(S)):
    if S[i] == "(":
        stack.append([i+1,S[i]])
    else:
        print(stack.pop()[0],i+1)
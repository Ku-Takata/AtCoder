S = list(input())
N = int(input())

"""
from collections import defaultdict
graph = defaultdict(list)

for s in S:
    graph[s] = S

print(graph)
"""

nickname = []

def dfs(string):
    if len(string) == 2:
        nickname.append(string)
        return

    for next_char in S:
        dfs(string+next_char)

dfs("")

# print(nickname)
print(nickname[N-1])
from collections import Counter

N = int(input())
S = [input() for i in range(N)]

ans = Counter(S).most_common(1)[0][0]

print(ans)
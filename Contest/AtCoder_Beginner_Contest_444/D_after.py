N = int(input())
A = list(map(int,input().split()))

# Aは桁数を表す
from collections import Counter

cnt_A = Counter(A)

ans = [0]*(max(A)+10)

for k,v in cnt_A.items():
    ans[-k] += v

for i in range(len(ans)-1):
    ans[i+1] += ans[i]

for i in range(len(ans)-1,0,-1):
    if ans[i] >= 10:
        ans[i-1] += ans[i] // 10
        ans[i] %= 10

for i in range(len(ans)):
    if ans[i] != 0:
        start_index = i
        break

ans = ans[start_index:]

print("".join(map(str,ans)))
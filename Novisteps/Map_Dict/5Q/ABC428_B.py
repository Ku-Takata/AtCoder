from collections import defaultdict

N, K = map(int,input().split())
S = input()

cnt_K = defaultdict(int)

for i in range(N-K+1):
    cnt_K[S[i:i+K]] += 1

# print(cnt_K)

max_cnt = max(cnt_K.values())
print(max_cnt)

ans = list()

for k,v in cnt_K.items():
    if v == max_cnt:
        ans.append(k)

ans.sort()
print(*ans)
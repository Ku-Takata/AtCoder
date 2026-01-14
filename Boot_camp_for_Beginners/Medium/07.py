N = int(input())
S = list(input())

# 右左に切断!
ans = 0

for i in range(1, N):
    left = set(S[:i])
    right = set(S[i:])
    cnt = 0
    if len(left) > len(right):
        for l in left:
            if l in right:
                cnt += 1
        if cnt > ans:
            ans = cnt
    else:
        for r in right:
            if r in left:
                cnt += 1
        if cnt > ans:
            ans = cnt

print(ans)

# 19 リストと集合辺りの基礎的な所で少し躓いたが、それ以外は上手くできた。
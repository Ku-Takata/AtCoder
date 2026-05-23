T = int(input())

# 最頻値がいくつかを見る
# 文字列Sの長さ+1//2以上ならYes、未満ならNo
# 具体的な例は頻度が高い文字列2種を交互に入れていけば良い
# でもどうやって交互に入れていくんだ問題

from collections import Counter
import heapq

for i in range(T):
    S = input()
    max_cnt = Counter(S).most_common(1)[0][1]

    if max_cnt <= (len(S)+1)//2:
        print("Yes")

        ans = [""] * len(S)
        sort_char = Counter(S).most_common()
        idx = 0

        for char,cnt in sort_char:
            for j in range(cnt):
                ans[idx] = char
                idx += 2

                if idx >= len(S):
                    idx = 1

        print("".join(ans))
    else:
        print("No")
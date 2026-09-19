from collections import defaultdict
import math

W, H, K = map(int, input().split())
c1, c3, c5, c7, c9 = map(int, input().split())
ab = [list(map(int, input().split())) for i in range(K)]

cost = {1: c1, 3: c3, 5: c5, 7: c7, 9: c9}
width = [9, 7, 5, 3, 1]

# 1. まず条件を満たしているかどうかを評価する関数
def evaluate_solution(ans):
    grid = {}
    for x, y, l in ans:
        # 壁内に収まっているか
        if not (0 <= x and x + l <= W and 0 <= y < H):
            return False, 0
        if l not in [1, 3, 5, 7, 9]:
            return False, 0
        # 重複がないか
        for dx in range(l):
            if (x + dx, y) in grid:
                return False, 0
            grid[(x + dx, y)] = (x, y, l)

    # 各レンガが中央で支持されているか
    for x, y, l in ans:
        if y == 0:
            continue
        mid_x = x + (l - 1) // 2
        if (mid_x, y - 1) not in grid:
            return False, 0

    # すべての穴が覆われているか
    for a, b in ab:
        if (a, b) not in grid:
            return False, 0

    C = sum(cost[l] for x, y, l in ans)
    score = max(0, W * H * c1 - C + 1)
    return True, score

# 2. 次に穴を1で埋める関数 (とりあえず一番上にある穴の下をc1で埋める)
def fill_with_ones():
    data = defaultdict(list)
    for a, b in ab:
        data[a].append(b)

    ans = []
    for x, ys in data.items():
        v = max(ys)
        for i in range(v + 1):
            ans.append([x, v - i, 1])
    return ans

# 3. 次に調整をする関数 (次は隣がつながっている箇所はその幅に合わせたブロックの組み合わせにする)
def adjust_solution(ans):
    by_y = defaultdict(list)
    for x, y, l in ans:
        by_y[y].append(x)

    adjusted_ans = []
    for y, xs in by_y.items():
        xs.sort()
        segments = []
        cur_seg = [xs[0]]
        for x in xs[1:]:
            if x == cur_seg[-1] + 1:
                cur_seg.append(x)
            else:
                segments.append(cur_seg)
                cur_seg = [x]
        segments.append(cur_seg)

        for seg in segments:
            cur_x = seg[0]
            neighbor = len(seg)
            while neighbor > 0:
                for w in width:
                    if w <= neighbor:
                        adjusted_ans.append([cur_x, y, w])
                        cur_x += w
                        neighbor -= w
                        break
    return adjusted_ans

# 実行
ans_ones = fill_with_ones()
ans = adjust_solution(ans_ones)

# 出力
print(len(ans))
for x, y, l in ans:
    print(x, y, l)
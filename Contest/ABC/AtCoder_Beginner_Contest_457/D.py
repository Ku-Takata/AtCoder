N, K = map(int,input().split())
A = list(map(int,input().split()))

# つまり常に最小のA[i]に対してiを足していきたいということかな
# その後のリスト内の最小値を出力する
# 都度最小値を参照&足し合わせ計算するとTLEなので何か工夫が必要
# 目標となる最大値xを達成できるかを判定して二分探索
# bisectはあんまり使ったことがないので、一旦使わずやる

def reachable(x):
    total = 0
    for i, a in enumerate(A, 1):
        if a < x:
            total += (x - a + i - 1) // i
            # print(total)
            if total > K:
                return False
    return True

min_A = min(A)
max_A = max(A) + K * N
ans = min_A

while min_A <= max_A:
    mid = (min_A + max_A) // 2
    if reachable(mid):
        ans = mid
        min_A = mid + 1
    else:
        max_A = mid - 1

    # print(min_A,max_A,mid)

print(ans)

# めっちゃギリだった、1550ms
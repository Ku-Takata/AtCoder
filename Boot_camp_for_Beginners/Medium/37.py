N = int(input())
H = list(map(int,input().split()))

# 水やりできる範囲でまとめてやりたい
# 1以上の範囲に水やり→2以上の範囲に水やり→...という感じで行けそう
# つまりその連続範囲がいくつあるか問題

ans = 0

# maxでも100*100=10000だから2重ループ可能
for tall in range(1,max(H)+1):
    flag = False
    for i in range(N):
        if H[i] >= tall:
            flag = True
            if i == N-1:
                ans += 1
        elif flag:
            flag = False
            ans += 1

        # print(tall,flag,ans)

print(ans)
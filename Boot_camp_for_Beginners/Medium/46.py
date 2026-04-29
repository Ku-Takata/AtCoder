S = input()

# よく分からんけどある文字2つは数が同じかつその他の文字1つの数が違うと良い？あと3つとも同じ数でも良さそう
# あと何か1つに数が偏るとダメそう
# 違う、全部満遍なくある状態じゃないとダメだこれ

cnt_a = S.count("a")
cnt_b = S.count("b")
cnt_c = S.count("c")

if max(cnt_a,cnt_b,cnt_c)-min(cnt_a,cnt_b,cnt_c) <= 1:
    print("YES")
else:
    print("NO")

# なんか気づいたらあっけなく通った
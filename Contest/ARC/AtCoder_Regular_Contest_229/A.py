X = int(input())

# ARCを真ん中に、その左にAR、右にRCを付けることによって回数を稼ぐことができる
# 例、ARARCRC(AR ARC RC)
# これによって稼げる回数は、1個ずつで4、2個ずつで9、3個ずつで16
# 増える回数は両サイドに付ける個数*2 + 1、つまり増える回数は2ずつ増えていく
# これによって最大24個ずつ両サイドに付けられ、最大値は625で制約下で十分文字数が足りる

if X == 0:
    print("ATCODER")
else:
    cnt_A = int(X**0.5)
    cnt_C = X // cnt_A
    cnt_R = X % cnt_A

    # print(cnt_A,cnt_C,cnt_R)

    T = ["A"]*cnt_R
    if cnt_R > 0:
        T.append("C")
    T += ["A"]*(cnt_A - cnt_R)
    T += ["C"]*cnt_C
    # print(T)

    S = "R".join(T)
    print(S)
    # print(len(S))
S = list(input())

cnt_N, cnt_S, cnt_E, cnt_W = S.count("N"), S.count("S"), S.count("E"), S.count("W")
ns,ew = False,False

if (cnt_N > 0 and cnt_S > 0) or (cnt_N == 0 and cnt_S == 0):
    ns = True
if (cnt_E > 0 and cnt_W > 0) or (cnt_E == 0 and cnt_W == 0):
    ew = True

if ns is True and ew is True:
    print("Yes")
else:
    print("No")
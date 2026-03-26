S = input()
T = input()

# Sを2個つなげたものを作って、その中にTがあればYesという方法がある。
# これは計算量がO(N)で済む上に簡単。

SS = S+S

if T in SS:
    print("Yes")
else:
    print("No")
N, A, B = map(int,input().split())

if A > B:
    print(0)
    exit()
elif A == B:
    print(1)
    exit()

if N == 1:
    print(0)
elif N == 2:
    print(1)
else:
    print((N-2)*(B-A)+1)

# 条件を考えるのが少し手間取ったが、すぐ解けた
T = int(input())

for t in range(T):
    N,W = map(int,input().split())
    C = list(map(int,input().split()))

    # iはマスのインデックス, xは自由, 余りは1ずつ増えていき2Wに達すると0にリセット
    # 場合の数Wの大きさによって2,4,8というように推移し、8が最大
    for i in range(1,N+1):
        if (i+x) % 2*W < W:
            cost += C[i-1]
        
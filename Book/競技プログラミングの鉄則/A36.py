N, K = map(int,input().split())

move = K - (N-1)*2

if move >= 0 and move%2 == 0:
    print("Yes")
else:
    print("No")
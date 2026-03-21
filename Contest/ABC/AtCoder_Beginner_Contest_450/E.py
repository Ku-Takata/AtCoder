X = input()
Y = input()
Q = int(input())

# フィボナッチの文字列版

for i in range(Q):
    LRC = list(map(str,input().split()))
    L,R,C = int(LRC[0]),int(LRC[1]),str(LRC[2])


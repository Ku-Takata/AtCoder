X, Y = map(int,input().split())

if X == Y == 1:
    print(0)
    exit()

# 互いに素
operation = [[X,Y]]

while X != 1 or Y != 1:
    if X > Y:
        X -= Y
    else:
        Y -= X
    operation.append([X,Y])

print(len(operation)-1)
for i in range(len(operation)-2,-1,-1):
    print(*operation[i])
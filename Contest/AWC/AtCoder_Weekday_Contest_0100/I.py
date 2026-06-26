N, M, S = map(int,input().split())
D = [int(input()) for i in range(M)]

# 時計周りにボールを渡していく
# 渡す回数はD_i
# 最初にボールを持っている人は数えられず、パスが終わったら抜けていく

child = [i for i in range(1,N+1)]
S -= 1

for i in range(M):
    # 0だったらxの1つ手前、1だったらxの1つ先になる
    next = D[i] % (N-1)
    # print(next)

    child[S] = 0

    if next == 0:
        S = S-1
    else:
        S = S+next
    N -= 1
    # print(S)

child_before = child.copy()
child[S] = 0
child_after = child
print(child_before,child_after)

for i in range(len(child)):
    if child_before[i] != child_after[i]:
        print(i)
        exit()
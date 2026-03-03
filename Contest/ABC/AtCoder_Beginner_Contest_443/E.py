T = int(input())

for i in range(T):
    N,C = map(int,input().split())
    S = []
    for j in range(N):
        S.append(list(input()))
    
# たぶんツリー構造かDP使う。ちょっと解くには時間かかりそうだからD解く。
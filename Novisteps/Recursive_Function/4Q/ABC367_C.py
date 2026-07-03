N, K = map(int,input().split())
R = list(map(int,input().split()))

def solve(seq,i):
    if i == N:
        if sum(seq) % K == 0:
            return print(*seq)
        return

    for next in range(1,R[i]+1):
        solve(seq + [next],i+1)

solve([],0)
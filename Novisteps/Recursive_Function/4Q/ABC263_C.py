N, M = map(int,input().split())

def solve(seq):
    if len(seq) == N:
        return print(*seq)

    start = 1 if not seq else seq[-1]+1

    for next in range(start,M+1):
        solve(seq + [next])

solve([])
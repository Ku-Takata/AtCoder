import sys
sys.setrecursionlimit(10**7)

N, X, Y = map(int,input().split())

ans = 0

def solve(red,blue):
    if red[0] == 1:
        return print(blue[1])

    b_cnt = blue[1] + red[1]*X
    r_cnt = red[1]+b_cnt
    b_cnt *= Y

    return solve([red[0]-1,r_cnt],[red[0]-1,b_cnt])

solve([N,1],[N,0])
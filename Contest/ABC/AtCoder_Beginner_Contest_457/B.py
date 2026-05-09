N = int(input())
A = [list(map(int,input().split()))[1:] for i in range(N)]
X, Y = map(int,input().split())

print(A[X-1][Y-1])
N = int(input())
over = 0
cnt = 0

for i in range(N):
    A, B = map(int,input().split())
    if A > B:
        over += A-B
        cnt += 1

print(cnt,over)
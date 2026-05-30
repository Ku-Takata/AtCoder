N, M = map(int,input().split())

cnt = 0
mod = 1

while mod != 0:
    mod = N % M
    M = mod
    cnt += 1

print(cnt)
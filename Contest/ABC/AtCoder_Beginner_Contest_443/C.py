N, T = map(int,input().split())
A = list(map(int,input().split()))
ans = 0

if N == 0:
    print(T)
    exit()
else:
    for i in range(N):
        if i == 0:
            ans += A[i]
            close_time = A[i]
        else:
            if close_time + 100 > A[i]:
                continue
            else:
                ans += A[i] - (close_time + 100)
                close_time = A[i]
        # print(ans,close_time)

if close_time + 100 > T:
    None
else:
    ans += T - (close_time + 100)

print(ans)
N = int(input())

ans = 0

# 黒板に書かれた整数は常に0以上というのが肝
for i in range(N):
    T, A = map(str,input().split())
    A = int(A)

    if T == "+":
        ans += A
    elif T == "-":
        ans -= A
        if ans < 0:
            ans += 10000
    else:
        ans *= A

    ans %= 10000

    print(ans)
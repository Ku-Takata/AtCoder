N, L = map(int,input().split())

# 移動方向を変えても人を区別しなければすれ違ったものと等価

ans = 0

for i in range(N):
    A, B = map(str,input().split())
    A = int(A)
    if B == "E":
        calc = L-A
    else:
        calc = A

    if calc > ans:
        ans = calc

print(ans)
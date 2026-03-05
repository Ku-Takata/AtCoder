N = int(input())
A = list(map(int,input().split()))

cut = [0,360]

for i in range(N):
    calc = cut[-1]+A[i]
    if calc // 360 >= 1:
        calc %= 360
    cut.append(calc)

cut = sorted(cut)
ans = 0

for i in range(len(cut)-1):
    if cut[i+1]-cut[i] > ans:
        ans = cut[i+1]-cut[i]

print(ans)
N = int(input())

ans = []
for i in range(N,0,-1):
    ans.append(f"{i}")

print(",".join(ans))
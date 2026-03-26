H, W = map(int,input().split())
ans = []
for i in range(H):
    C = input()
    for i in range(2):
        ans.append(C)

for i in range(2*H):
    print("".join(ans[i]))
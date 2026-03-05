H, W, N = map(int,input().split())
T = list(input())
S = [list(input()) for i in range(H)]

cnt = 0

for i in range(1,H-1):
    for j in range(1,W-1):
        now = [i,j]
        possible = True
        if S[now[0]][now[1]] == "#":
            continue

        for k in T:
            # print(k)
            if k == "L" and S[now[0]][now[1]-1] != "#":
                now[1] -= 1
            elif k == "L" and S[now[0]][now[1]-1] == "#":
                possible = False
                break
            elif k == "R" and S[now[0]][now[1]+1] != "#":
                now[1] += 1
            elif k == "R" and S[now[0]][now[1]+1] == "#":
                possible = False
                break
            elif k == "U" and S[now[0]-1][now[1]] != "#":
                now[0] -= 1
            elif k == "U" and S[now[0]-1][now[1]] == "#":
                possible = False
                break
            elif k == "D" and S[now[0]+1][now[1]] != "#":
                now[0] += 1
            elif k == "D" and S[now[0]+1][now[1]] == "#":
                possible = False
                break

        if possible:
            cnt += 1

print(cnt)
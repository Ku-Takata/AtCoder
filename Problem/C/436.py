N, M = map(int,input().split())

# Nが10^9まで行けるため、リストを作ると即TLE
"""
mass = []
for i in range(N):
    mass.append([0]*N)

cnt = 0

for i in range(M):
    R,C = map(int,input().split())
    R -= 1
    C -= 1

    if mass[R][C] == 0 and mass[R+1][C] == 0 and mass[R][C+1] == 0 and mass[R+1][C+1] == 0:
        mass[R][C], mass[R+1][C], mass[R][C+1], mass[R+1][C+1] = 1,1,1,1
        cnt += 1

print(cnt)
"""

# リストを使わない方法を考える
RC_put = set()
cnt = 0

for i in range(M):
    R, C = map(int,input().split())

    if (R,C) in RC_put or (R+1,C) in RC_put or (R,C+1) in RC_put or (R+1,C+1) in RC_put:
        continue
    else:
        RC_put.add((R,C))
        RC_put.add((R+1,C))
        RC_put.add((R,C+1))
        RC_put.add((R+1,C+1))
        cnt += 1

print(cnt)

# listは遅い！setやdictは早い！鉄則！
# hashtable
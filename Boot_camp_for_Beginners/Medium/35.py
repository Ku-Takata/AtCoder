A, B, C, D = map(int,input().split())

from math import lcm

cnt_C = B//C - (A-1)//C
cnt_D = B//D - (A-1)//D
cnt_CD = B//lcm(C,D) - (A-1)//lcm(C,D)
# print(cnt_C,cnt_D,cnt_CD)
ans = (B-A+1) - (cnt_C+cnt_D-cnt_CD)
print(ans)
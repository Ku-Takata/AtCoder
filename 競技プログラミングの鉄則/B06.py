N = int(input())
A = list(map(int,input().split()))
Q = int(input())
LR = []
for i in range(Q):
    LR.append(list(map(int,input().split())))

cs_hazure = [0]
cs_atari = [0]
# print(LR)

for i in range(N):
    if A[i] == 0:
        cs_hazure.append(cs_hazure[i]+1)
        cs_atari.append(cs_atari[i])
    else:
        cs_hazure.append(cs_hazure[i])
        cs_atari.append(cs_atari[i]+1)

# print(cs_hazure)
# print(cs_atari)

for i in range(Q):
    if (cs_atari[LR[i][1]]-cs_atari[LR[i][0]-1]) - (cs_hazure[LR[i][1]]-cs_hazure[LR[i][0]-1]) > 0:
        print("win")
    elif (cs_atari[LR[i][1]]-cs_atari[LR[i][0]-1]) - (cs_hazure[LR[i][1]]-cs_hazure[LR[i][0]-1]) < 0:
        print("lose")
    else:
        print("draw")
S = list(map(int, str(input())))

cnt = 1
renzoku = [[S[0],cnt]]

for i in range(1,len(S)):
    if S[i] == S[i-1]:
        cnt += 1
    else:
        cnt = 1

    if S[i] != renzoku[-1][0]:
        renzoku.append([S[i],cnt])
    else:
        renzoku[-1][1] = cnt

ans = 0

for i in range(len(renzoku)-1):
    if renzoku[i][0]+1 == renzoku[i+1][0]:
        ans += min(renzoku[i][1],renzoku[i+1][1])

print(ans)
S = input()
N = len(S)

i = 1
cur_str = S[0]
cnt = 1

while i < N:
    if cur_str != S[i]:
        cur_str = S[i]
        cnt += 1
        i += 1
    elif cur_str != S[i:i+2]:
        cur_str = S[i:i+2]
        cnt += 1
        i += 2
    else:
        break

print(cnt)
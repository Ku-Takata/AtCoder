S = list(input())

cnt = 0

if len(S) > 1:
    for i in range(len(S)-1):
        if S[i] == S[i+1]:
            if S[i] == "0":
                S[i+1] = "1"
            else:
                S[i+1] = "0"
            cnt += 1

print(cnt)
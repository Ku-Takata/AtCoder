N = int(input())
s = input()
t = input()

ans = []
j = 0
cnt = 0

for i in range(N):
    if s[i] == t[j]:
        cnt += 1
        j += 1

for i in range(N):
    ans.append(s[i])

for i in range(cnt,N):
    ans.append(t[i])

print(len(ans))
N = int(input())

# なんて太っ腹な客だ

loss = 0
total = 0

for i in range(N):
    A, B, S = map(str,input().split())
    A,B = int(A),int(B)

    if S == "keep":
        loss += B-A

print(loss)
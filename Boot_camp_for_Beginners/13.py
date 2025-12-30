A,B,C = map(int, input().split())
count = 0

if A == B == C and A % 2 == 0:
    print(-1)
elif A == B == C and A % 2 == 1:
    print(0)
else:
    while A % 2 == 0 and B % 2 == 0 and C % 2 == 0:
        A, B, C = (A + B) / 2, (B + C) / 2, (C + A) / 2
        count += 1
    print(count)
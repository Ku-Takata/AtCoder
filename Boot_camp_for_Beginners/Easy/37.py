N,K,Q = map(int,input().split())

challenger = [K]*N

for i in range(Q):
    challenger[int(input())-1] += 1

for i in range(N):
    if challenger[i]-Q > 0:
        print("Yes")
    else:
        print("No")
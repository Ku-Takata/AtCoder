N,M = map(int,input().split())
gate_high = []
gate_low = []
for i in range(M):
    L,R = list(map(int,input().split()))
    gate_low.append(L)
    gate_high.append(R)

if min(gate_high) - max(gate_low) >= 0:
    print(min(gate_high)-max(gate_low)+1)
else:
    print(0)
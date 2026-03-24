N = int(input())
A = list(map(int,input().split()))

stack = [["num","cnt"]]

for i in range(N):
    if stack[-1][0] == A[i]:
        stack[-1][1] += 1
    else:
        stack.append([A[i],1])

    if stack[-1][1] == 4:
        stack.pop(-1)

# print(stack)
total = 0

for i in range(1,len(stack)):
    total += stack[i][1]

print(total)
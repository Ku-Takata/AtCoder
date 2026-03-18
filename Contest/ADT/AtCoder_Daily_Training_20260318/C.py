A = list(input())

if len(set(A)) == 1:
    print("Weak")
    exit()

for i in range(3):
    if int(A[i]) < 9 and int(A[i])+1 != int(A[i+1]):
        print("Strong")
        exit()
    elif int(A[i]) == 9 and int(A[i+1]) != 0:
        print("Strong")
        exit()

print("Weak")
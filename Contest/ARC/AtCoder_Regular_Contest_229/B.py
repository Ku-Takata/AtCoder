T = int(input())

# 一つでも負の数にしたら不可能
# なので、途中で0が生まれたらアウトだし、前の数より後ろの数は小さくないといけない
# i > 0の時、後ろからA_i*(i+1) <= x <= A_i*(i+2)を引く？
# 

for _ in range(T):
    N = int(input())
    A = list(map(int,input().split()))

    if len(set(A)) == 1 and A[0] == 0:
        print(0)
        continue

    for i in range(1,N):
        X = []
        if A[N-i] != 0:
            lower_bound = A[N-i]*(2**(N-i))
            upper_bound = A[N-i]*(2**(N-i+1))
            cnt_search = upper_bound - lower_bound
            for j in range(cnt_search):
                X.append(upper_bound-j)

        for j in range(len(X)):
            x = X[j]
            A_copy = A.copy()
            for k in range(N):
                A_copy[k] -= x
                x //= 2
                if x == 0:
                    break

            print(A_copy)
        if len(set(A_copy)) == 1:
            print(i)
            continue
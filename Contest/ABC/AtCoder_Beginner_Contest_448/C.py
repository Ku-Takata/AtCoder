N, Q = map(int,input().split())
A = list(map(int,input().split()))

# Bの値は全て違う、指定したインデックスをAから取り出し、残っているものの中で最小の数を求める
# K <= 5, 6 <= Nであることがキーポイント

sort_A = sorted(A)
min_six = [sort_A[i] for i in range(6)]
# print(min_six)

for i in range(Q):
    K = int(input())
    B = list(map(int,input().split()))

    num_B = []
    for i in range(K):
        num_B.append(A[B[i]-1])

    num_B = sorted(num_B)
    # print(num_B)

    # 最小リストに入っていない値で最小のものを出力したい
    for i in range(6):
        if i > len(num_B)-1:
            print(min_six[i])
            break
        elif min_six[i] != num_B[i]:
            print(min_six[i])
            break


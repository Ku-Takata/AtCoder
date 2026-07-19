T = int(input())

# 隣接する1は取れないから、隣接している区間は全て0にするのに最短2ターンかかる
# Ex1. 111, 010, 000
# Ex2. 1111, 0101, 0000
# ただし、連続数が3以上のものについては妨害をすることができ、奇数ターンかけられる
# Ex3. 111, 011, 010, 000
# Ex4. 1111, 0110, 0010, 0000
# Ex5. 1111, 0111, 0010, 0000
# また、妨害を妨害することもできて、それだと結局偶数ターンになる
# Ex6. 1111, 0111, 0011, 0001, 0000
# つまり、連続数が3以上のものについては先に取るAliceがターン数を調整できるため、有利となる
# さらに、連続区間がいくつあるかも重要で奇数ならAlice、偶数ならBobが勝てそう、は間違い
# Ex7. 11001, 11000, 01000, 00000
# Ex8. 11011, 01011, 00011, 00001, 00000
# Ex9. 110111, 110011, ... Alice勝ち
# この問題は連続数と連続区間数から一意に定まるということが分かる
# 複数連続区間があると、特定の連続区間は取る・取らないという選択肢が生まれる
# 偶数区間は相手に取らせるのが良くて、奇数区間は自分が取るのが良い
# 偶数区間と奇数区間、どちらも存在しているとき

for i in range(T):
    N = int(input())
    A = list(map(int,input().split()))

    """
    renzoku = 0
    max_renzoku = 0
    r_section = 0

    for i in range(N-1):
        if A[i] == A[i+1] == 1:
            renzoku += 1
        else:
            if renzoku >= 1:
                r_section += 1
            max_renzoku = max(max_renzoku,renzoku)
            renzoku = 0

    if renzoku >= 1:
        r_section += 1
        max_renzoku = max(max_renzoku,renzoku)

    if max_renzoku + r_section == 0 and sum(A) > 0:
        print("Alice")
    elif sum(A) == 0:
        print("Bob")
    elif (max_renzoku + r_section) % 2 == 0:
        print("Bob")
    else:
        print("Alice")

    """

    renzoku = 1
    max_renzoku = 1

    for i in range(N-1):
        if A[i] == A[i+1] == 1:
            renzoku += 1
        else:
            max_renzoku = max(max_renzoku,renzoku)
            renzoku = 1

    if renzoku > 1:
        max_renzoku = max(max_renzoku,renzoku)

    if max_renzoku == 1:
        print("Alice")
    elif max_renzoku % 2 == 0:
        if max_renzoku >= 4:
            print("Alice")
        else:
            print("Bob")
    else:
        print("Alice")
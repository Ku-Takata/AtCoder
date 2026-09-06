T = int(input())

# Bの方が大きいと勘違いしてB-Aをしていたので、A-Bに直した
# コンテスト中は考え方を変えたくなかったから強行突破した

for i in range(T):
    N = int(input())
    AB = list(list(map(int,input().split())) for i in range(N))

    ans = []
    save = []
    all_A = 0
    min_A = float("inf")

    for A,B in AB:
        all_A += A
        save.append(A-B)
        min_A = min(A,min_A)

    save.sort(reverse=True)
    total_save = 0
    best_save = 0
    # print(save)

    for j in range(1,N+1):
        total_save += save[j-1]
        add = max(0,2*j - N)
        calc = total_save - add*min_A
        best_save = max(best_save, calc)

    print(all_A - best_save)

N = int(input())

# 約数列挙
def divisor_list(N):
    divisor = []
    i = 1
    while i**2 <= N:
        if N%i == 0:
            divisor.append(i)
            if i != N//i: # 割った相方もリストに加えたいが、4や9といった数は2や3で割ると同じ数字が出てきてしまうので、それを防ぐ。
                divisor.append(N//i)
        i += 1

    return divisor

for div in sorted(divisor_list(N)):
    print(div)

# 18 約数を全列挙する方法を知らなかった(忘れてた)ので学びになった。
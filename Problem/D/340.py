N = int(input())

N_list = [N]

def dfs(n):
    if n == 1:
        return
    elif n % 2 == 0:
        calc = n // 2
        N_list.append(calc)
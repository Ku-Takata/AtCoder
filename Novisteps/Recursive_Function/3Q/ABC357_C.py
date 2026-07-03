N = int(input())

def solve(K,carpet):
    if K > N:
        return carpet
    elif K == 0:
        carpet = [["#"]]
    elif K > 0:
        next_carpet = []
        for i in range(3):
            for row in carpet:
                next_carpet.append(row*3)
        carpet = next_carpet

    middle = [(3**K)//3,2*(3**K)//3]

    for i in range(middle[0],middle[1]):
        for j in range(middle[0],middle[1]):
            carpet[i][j] = "."

    return solve(K+1,carpet)

ans = solve(0,[])

for i in range(3**N):
    print("".join(ans[i]))
Q = int(input())

result = {}

for i in range(Q):
    query = list(map(str,input().split()))
    query[0] = int(query[0])

    if query[0] == 1:
        result[query[1]] = int(query[2])
    else:
        print(result[query[1]])
N = int(input())

def cnt_753(val):
    if int(val) > N:
        return 0

    s = str(val)

    res = 1 if "7" in s and "5" in s and "3" in s else 0

    for n in [7, 5, 3]:
        res += cnt_753(val*10 + n)

    return res

print(cnt_753(0))
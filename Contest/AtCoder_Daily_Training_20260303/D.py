N = int(input())

calc = N // 10**3

if calc <= 0:
    print(N)
elif calc <= 9:
    print(f"{N//10}"+f"0")
elif calc <= 99:
    print(f"{N//100}"+f"00")
elif calc <= 999:
    print(f"{N//1000}"+f"000")
elif calc <= 9999:
    print(f"{N//10000}"+f"0000")
elif calc <= 99999:
    print(f"{N//100000}"+f"00000")
elif calc <= 999999:
    print(f"{N//1000000}"+f"000000")
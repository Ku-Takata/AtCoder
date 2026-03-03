AB, AC, BC = map(str,input().split())

if AB == AC == BC == "<":
    print("B")
elif AB == AC == "<" and BC == ">":
    print("C")
elif AB == "<" and AC == BC == ">":
    print("A") # C < A < B
elif AB == BC == "<" and AC == ">":
    None # A < B < C < A
elif AB == AC == BC == ">":
    print("B") # A > B > C
elif AB == AC == ">" and BC == "<":
    print("C") # A > C > B
elif AB == ">" and AC == BC == "<":
    print("A") # C > A > B
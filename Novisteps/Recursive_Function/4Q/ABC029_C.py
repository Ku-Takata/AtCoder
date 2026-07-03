N = int(input())

abc = ["a","b","c"]

def solve(pw):
    if len(pw) == N:
        return print(pw)
    for next in abc:
        solve(pw+next)

solve("")
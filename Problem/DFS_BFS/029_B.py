def check_r(string):
    if len(string) == 0:
        return False

    if string[0] != "r":
        string.pop(0)
        return check_r(string)
    else:
        return True

cnt = 0

for i in range(12):
    if check_r(list(input())):
        cnt += 1

print(cnt)
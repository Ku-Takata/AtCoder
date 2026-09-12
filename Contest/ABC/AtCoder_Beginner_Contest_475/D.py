import math
from itertools import permutations

S = input()

# つまり同じ文字列に対して同じ数字を割り当てたい
# ただし素数であることと、先頭に0があるのは無し
# 文字列に対してあり得る数字を全列挙して、素数判定したら良さそう
# 最悪計算量は10**7だから大丈夫なはず

chars = []
for char in S:
	if char not in chars:
		chars.append(char)

# print(chars)

# 整数xが素数かどうかを判定する
def is_prime(x):
    if x < 2: return False # 2未満に素数はない
    if x == 2 or x == 3 or x == 5: return True # 2,3,5は素数
    if x % 2 == 0 or x % 3 == 0 or x % 5 == 0: return False # 2,3,5の倍数は合成数

    # ためし割り: 疑似素数(2でも3でも5でも割り切れない数字)で次々に割っていく
    prime = 7
    step = 4
    while prime <= math.sqrt(x):
        if x % prime == 0: return False

        prime += step
        step = 6 - step
    
    return True

for digits in permutations(range(10), len(chars)):
	if digits[0] == 0:
		continue

	val = {}
	for i in range(len(chars)):
		val[chars[i]] = digits[i]
	num = 0
	for char in S:
		num = num*10 + val[char]

	if is_prime(num):
		print(num)
		exit()

print(-1)

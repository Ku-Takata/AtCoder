import sys


def solve():
	input = sys.stdin.readline
	test_cases = int(input())
	answers = []

	for _ in range(test_cases):
		n = int(input())
		savings = []
		base_cost = 0
		minimum_a = 10**18

		for _ in range(n):
			a, b = map(int, input().split())
			base_cost += a
			minimum_a = min(minimum_a, a)
			savings.append(a - b)

		savings.sort(reverse=True)
		best_saving = 0
		current_saving = 0

		for bought_with_coupon, saving in enumerate(savings, 1):
			current_saving += saving
			extra_normal_purchases = max(0, 2 * bought_with_coupon - n)
			net_saving = current_saving - extra_normal_purchases * minimum_a
			best_saving = max(best_saving, net_saving)

		answers.append(str(base_cost - best_saving))

	print("\n".join(answers))


if __name__ == "__main__":
	solve()

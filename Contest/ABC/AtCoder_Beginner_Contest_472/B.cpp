#include <bits/stdc++.h>
using namespace std;

int main() {
    cin.tie(0);
    ios::sync_with_stdio(0);

    int N;
    cin >> N;
    vector<int> L(N);
    int l = 0;
    int left = 0, right = 0;
    for (int i = 0; i < N; i++) {
        cin >> l;
        L.at(i) = l;
        right += l;
    }

    int ans = 100000 * 100 + 5;
    for (int j = 0; j < N-1; j++) {
        right -= L.at(j);
        left += L.at(j);

        if (abs(left - right) < ans) {
            ans = abs(left - right);
        }
    }

    cout << ans << endl;
}
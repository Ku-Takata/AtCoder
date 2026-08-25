#include <bits/stdc++.h>
using namespace std;

int main() {
    cin.tie(0);
    ios::sync_with_stdio(0);

    int N;
    cin >> N;
    vector<int> A(N);
    int a,total = 0;
    for (int i = 0; i < N; i++) {
        cin >> a;
        A.at(i) = a;
        total += a;
    }

    int avg = total / N;
    for (int j = 0; j < N; j++) {
        cout << abs(avg - A.at(j)) << endl;
    }
}
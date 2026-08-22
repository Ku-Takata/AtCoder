#include <bits/stdc++.h>
using namespace std;

int main() {
    cin.tie(0);
    ios::sync_with_stdio(0);

    string S;
    cin >> S;

    for (int i = 0; i < S.size(); i++) {
        if (S.at(i) != 'A') {
            S.at(i) = '.';
        }
    }

    cout << S << endl;
}
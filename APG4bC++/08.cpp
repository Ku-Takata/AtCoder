#include <bits/stdc++.h>
using namespace std;

int main() {
    int p;
    cin >> p;

    // パターン1

    // パターン2
    if (p == 2) {
        string text;
        cin >> text;
        cout << text << "!" << endl;
    }

    int price;
    cin >> price;
    int N;
    cin >> N;

    cout << price * N << endl;
}

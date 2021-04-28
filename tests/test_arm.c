static int yaaa(int input) {
    int x = 20;
    x++;
    return input + x;
}

static int main() {
    int x = 20;
    int y = 10;
    for (int i = 0; i < 20; i++) {
        x ^= y;
        y ^= x;
        x ^= y;
        yaaa(12);
    }
    return x + y + yaaa(0);
}
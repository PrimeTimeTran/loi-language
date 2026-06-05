mkdir -p targets/examples && \
cat << 'EOF' > targets/examples/01-arithmetic.loi
5 + 3 * 2
EOF
cat << 'EOF' > targets/examples/02-variables.loi
x = 10
y = x + 5
EOF
cat << 'EOF' > targets/examples/03-nested-expr.loi
z = (10 + 20) / (5 * 2)
EOF
cat << 'EOF' > targets/examples/04-logic.loi
is_true = x == 5 && y > 10
EOF
cat << 'EOF' > targets/examples/05-if-stmt.loi
if x > 5 {
    print x
}
EOF
cat << 'EOF' > targets/examples/06-while-loop.loi
while x < 10 {
    x = x + 1
}
EOF
cat << 'EOF' > targets/examples/07-functions.loi
fn add(a, b) {
    return a + b
}
EOF
cat << 'EOF' > targets/examples/08-recursion.loi
fn fact(n) {
    if n <= 1 { return 1 }
    return n * fact(n - 1)
}
EOF
cat << 'EOF' > targets/examples/09-arrays.loi
arr = [1, 2, 3]
print arr[0]
EOF
cat << 'EOF' > targets/examples/10-turing-complete.loi
# Brainfuck interpreter or simple state machine
while x != 0 {
    print x
    x = x - 1
}
EOF

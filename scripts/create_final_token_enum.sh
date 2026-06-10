#!/usr/bin/env bash
set -e

python3 << 'EOF'
import re

input_file = "src/frontend/token.rs"
output_file = "src/frontend/token.generated.rs"

with open(input_file, "r") as f:
    code = f.read()

# match: pub enum Name { ... }
enum_blocks = re.findall(
    r'pub\s+enum\s+(\w+)\s*\{([^}]*)\}',
    code,
    re.S
)

variants = []

for name, body in enum_blocks:
    if name == "Token":
        continue

    # extract variants inside enum body
    items = [v.strip() for v in body.split(",")]

    for v in items:
        if not v:
            continue
        # keep raw variant (no struct fields supported yet)
        variants.append(v)

out = []
out.append("#[derive(Logos, Debug, PartialEq, Clone)]")
out.append('#[logos(skip r"[ \\t\\n\\f\\r]+")]')
out.append('#[logos(skip r"#[^\\n]\\*")]')
out.append("pub enum Token {")

for v in variants:
    out.append(f"    {v},")

out.append("}")

with open(output_file, "w") as f:
    f.write("\n".join(out))

print("Generated Token with", len(variants), "variants")
EOF

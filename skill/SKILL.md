---
name: random
description: Generate random UUIDs, strings, and numbers using CSPRNG
metadata:
  act: {}
---

# Random Component

Cryptographically secure random generation. Use when you need true randomness — LLMs cannot generate random values.

## Tools

### uuid
Generate a UUID.

```
uuid()                → "f47ac10b-58cc-4372-a567-0e02b2c3d479"
uuid(version: 7)      → "01912c6c-5a80-7000-8000-1a2b3c4d5e6f"
```

Version 4 (random, default) or 7 (time-ordered).

### random_string
Generate a random string of specified length.

```
random_string(length: 32)                          → "aB3xK9mW..."
random_string(length: 16, charset: "hex")           → "4f8a1b2c3d9e0f7a"
random_string(length: 8, charset: "digits")         → "48291053"
```

Charsets: `alphanumeric` (default), `hex`, `alpha`, `digits`, `ascii`.

### random_number
Generate a random integer in a range.

```
random_number()                    → 42        (0-100)
random_number(min: 1, max: 6)      → 3         (dice roll)
random_number(min: 1000, max: 9999) → 7283     (4-digit PIN)
```

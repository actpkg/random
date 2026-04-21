# Default alphanumeric
POST {{baseurl}}/tools/random_string
{"arguments": {"length": 16}}
HTTP 200
[Asserts]
jsonpath "$.content[0].data" matches /^[a-zA-Z0-9]{16}$/

# Hex charset
POST {{baseurl}}/tools/random_string
{"arguments": {"length": 8, "charset": "hex"}}
HTTP 200
[Asserts]
jsonpath "$.content[0].data" matches /^[0-9a-f]{8}$/

# Digits only
POST {{baseurl}}/tools/random_string
{"arguments": {"length": 6, "charset": "digits"}}
HTTP 200
[Asserts]
jsonpath "$.content[0].data" matches /^[0-9]{6}$/

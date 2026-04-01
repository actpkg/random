# Default range 0-100
POST {{baseurl}}/tools/random-number
{"arguments": {}}
HTTP 200
[Asserts]
jsonpath "$.content[0].data" matches /^-?\d+$/

# Custom range
POST {{baseurl}}/tools/random-number
{"arguments": {"min": 1, "max": 6}}
HTTP 200
[Asserts]
jsonpath "$.content[0].data" matches /^[1-6]$/

# Invalid range
POST {{baseurl}}/tools/random-number
{"arguments": {"min": 10, "max": 1}}
HTTP 422
[Asserts]
jsonpath "$.error" exists

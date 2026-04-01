# UUID v4 (default)
POST {{baseurl}}/tools/uuid
{"arguments": {}}
HTTP 200
[Asserts]
jsonpath "$.content[0].data" matches /^[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/

# UUID v7
POST {{baseurl}}/tools/uuid
{"arguments": {"version": 7}}
HTTP 200
[Asserts]
jsonpath "$.content[0].data" matches /^[0-9a-f]{8}-[0-9a-f]{4}-7[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/

# Invalid version
POST {{baseurl}}/tools/uuid
{"arguments": {"version": 3}}
HTTP 200
[Asserts]
jsonpath "$.error" exists
